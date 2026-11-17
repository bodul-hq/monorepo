use crate::app;
use crate::offer_read::tests::offers as tests_offers;
use crate::reporting::tests::empty;
use crate::tests::assert_batch_response;
use crate::tests::assert_error_response;
use ::commanding::prelude::poem::CommandingAdapter;
use ::commanding::prelude::testing as commanding;
use ::eventing::prelude::poem::EventingAdapter;
use ::eventing::prelude::testing as eventing;
use ::inbox::prelude::poem::InboxAdapter;
use ::inbox::prelude::testing as inbox;
use commanding::empty_consumer as commanding_empty_consumer;
use commanding::failing_reservation_consumer as commanding_failing_reservation_consumer;
use commanding::panicking_consumer as commanding_panicking_consumer;
use commanding::queued_consumer as commanding_queued_consumer;
use eventing::empty_consumer as eventing_empty_consumer;
use eventing::failing_reservation_consumer as eventing_failing_reservation_consumer;
use eventing::panicking_consumer as eventing_panicking_consumer;
use eventing::partially_failing_consumer;
use eventing::queued_consumer as eventing_queued_consumer;
use inbox::failing_storage_parts;
use inbox::invalid_run_requests;
use inbox::invalid_submissions;
use inbox::panicking_parts;
use inbox::recording_parts;
use inbox::valid_run_requests;
use inbox::valid_submissions;
use outbox::prelude::poem::OutboxAdapter;
use outbox::prelude::testing::panicking_consumer as testing_panicking_consumer;
use poem::Route;
use poem::{http::StatusCode, test::TestClient};
use serde_json::{Value, json};
use tokio::test as tokio_test;

use crate::orchestration::AppState;

pub fn state() -> AppState {
    AppState {
        reporting: empty(),
        offers: tests_offers(Vec::new()),
        inbox: InboxAdapter::new(recording_parts().0),
        commands: CommandingAdapter::new(commanding_empty_consumer()),
        events: EventingAdapter::new(eventing_empty_consumer()),
        outbox: OutboxAdapter::unconfigured(),
    }
}

fn client(state: AppState) -> TestClient<Route> {
    TestClient::new(app(state))
}

#[tokio_test]
async fn runners_validate_requests_and_preserve_empty_batch_contracts() {
    let client = client(state());
    for queue in ["inbox", "commands", "events", "outbox"] {
        let path = format!("/api/{queue}/run");
        for body in invalid_run_requests() {
            let response = client.post(&path).body_json(&body).send().await;
            assert_eq!(
                response.0.status(),
                StatusCode::BAD_REQUEST,
                "{queue}: {body}"
            );
        }
        for body in valid_run_requests() {
            let response = client.post(&path).body_json(&body).send().await;
            if queue == "outbox" {
                response.assert_status(StatusCode::SERVICE_UNAVAILABLE);
                response
                    .assert_text("outbox publisher is not configured")
                    .await;
            } else {
                response.assert_status_is_ok();
                response
                    .assert_json(json!({"processed":0,"errors":[]}))
                    .await;
            }
        }
    }
}

#[tokio_test]
async fn command_and_event_batches_stop_at_the_requested_limit() {
    let mut state = state();
    state.commands = CommandingAdapter::new(commanding_queued_consumer(3));
    state.events = EventingAdapter::new(eventing_queued_consumer(3));
    let client = client(state);
    for queue in ["commands", "events"] {
        for processed in [2, 1, 0] {
            let response = client
                .post(format!("/api/{queue}/run"))
                .body_json(&json!({"limit":2}))
                .send()
                .await;
            response.assert_status_is_ok();
            response
                .assert_json(json!({"processed":processed,"errors":[]}))
                .await;
        }
    }
}

#[tokio_test]
async fn all_queue_routes_require_well_formed_json() {
    let client = client(state());
    for path in [
        "/api/inbox",
        "/api/inbox/run",
        "/api/commands/run",
        "/api/events/run",
        "/api/outbox/run",
    ] {
        for content_type in [None, Some("text/plain")] {
            let mut request = client.post(path).body(r#"{"limit":1}"#);
            if let Some(content_type) = content_type {
                request = request.content_type(content_type);
            }
            request
                .send()
                .await
                .assert_status(StatusCode::UNSUPPORTED_MEDIA_TYPE);
        }
        client
            .post(path)
            .content_type("application/json")
            .body("{")
            .send()
            .await
            .assert_status(StatusCode::BAD_REQUEST);
    }
}

#[tokio_test]
async fn inbox_validates_envelopes_and_returns_the_durably_stored_id() {
    let (service, store) = recording_parts();
    let mut state = state();
    state.inbox = InboxAdapter::new(service);
    let client = client(state);
    for body in invalid_submissions() {
        client
            .post("/api/inbox")
            .body_json(&body)
            .send()
            .await
            .assert_status(StatusCode::BAD_REQUEST);
    }
    assert!(store.stored_ids().is_empty());
    for body in valid_submissions() {
        let response = client.post("/api/inbox").body_json(&body).send().await;
        response.assert_status(StatusCode::ACCEPTED);
        let body: Value = response.json().await.value().deserialize();
        assert_eq!(body["message_id"], *store.stored_ids().last().unwrap());
    }
}

#[tokio_test]
async fn inbox_submission_sanitizes_storage_and_task_failures() {
    for service in [failing_storage_parts(), panicking_parts()] {
        let mut state = state();
        state.inbox = InboxAdapter::new(service);
        let response = client(state)
            .post("/api/inbox")
            .body_json(
                &json!({"payload":{"value":1},"routing_key":"DevelopmentProbe","source":"test"}),
            )
            .send()
            .await;
        response.assert_status(StatusCode::INTERNAL_SERVER_ERROR);
        assert_error_response(response).await;
    }
}

#[tokio_test]
async fn runner_storage_failures_are_batch_reports() {
    let mut state = state();
    state.inbox = InboxAdapter::new(failing_storage_parts());
    state.commands = CommandingAdapter::new(commanding_failing_reservation_consumer());
    state.events = EventingAdapter::new(eventing_failing_reservation_consumer());
    let client = client(state);
    for queue in ["inbox", "commands", "events"] {
        let response = client
            .post(format!("/api/{queue}/run"))
            .body_json(&json!({"limit":1}))
            .send()
            .await;
        response.assert_status_is_ok();
        assert_batch_response(response, 1).await;
    }
}

#[tokio_test]
async fn task_panics_are_sanitized_for_every_runner() {
    let mut state = state();
    state.inbox = InboxAdapter::new(panicking_parts());
    state.commands = CommandingAdapter::new(commanding_panicking_consumer());
    state.events = EventingAdapter::new(eventing_panicking_consumer());
    state.outbox = OutboxAdapter::new(testing_panicking_consumer());
    let client = client(state);
    for queue in ["inbox", "commands", "events", "outbox"] {
        let response = client
            .post(format!("/api/{queue}/run"))
            .body_json(&json!({"limit":1}))
            .send()
            .await;
        response.assert_status(StatusCode::INTERNAL_SERVER_ERROR);
        assert_error_response(response).await;
    }
}

#[tokio_test]
async fn event_partial_failures_retain_successful_items() {
    let (service, completed) = partially_failing_consumer();
    let mut state = state();
    state.events = EventingAdapter::new(service);
    let response = client(state)
        .post("/api/events/run")
        .body_json(&json!({"limit":10}))
        .send()
        .await;
    response.assert_status_is_ok();
    assert_batch_response(response, 1).await;
    assert_eq!(*completed.lock().unwrap(), 2);
}

#[tokio_test]
async fn openapi_describes_all_queue_routes_with_shared_schemas() {
    let client = client(state());
    let spec: Value = client
        .get("/api/openapi.json")
        .send()
        .await
        .json()
        .await
        .value()
        .deserialize();
    assert!(spec["paths"]["/inbox"]["post"]["responses"]["202"].is_object());
    assert_eq!(
        spec["components"]["schemas"]["SubmitInboxRequest"]["required"],
        json!(["payload", "routing_key", "source"])
    );
    for queue in ["inbox", "commands", "events", "outbox"] {
        let run = &spec["paths"][format!("/{queue}/run")]["post"];
        assert_eq!(
            run["requestBody"]["content"]["application/json; charset=utf-8"]["schema"]["$ref"],
            "#/components/schemas/RunRequest"
        );
        assert_eq!(
            run["responses"]["200"]["content"]["application/json; charset=utf-8"]["schema"]["$ref"],
            "#/components/schemas/RunResponse"
        );
    }
    let limit = &spec["components"]["schemas"]["RunRequest"]["properties"]["limit"];
    assert_eq!(limit["minimum"], 1.0);
    assert_eq!(limit["maximum"], 1000.0);
    let run_schemas: Vec<_> = spec["components"]["schemas"]
        .as_object()
        .unwrap()
        .keys()
        .filter(|name| name.starts_with("Run"))
        .collect();
    assert_eq!(run_schemas, ["RunRequest", "RunResponse"]);
}
