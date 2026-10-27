use commanding_api::io::testing as commanding;
use eventing_api::io::testing as eventing;
use inbox_api::io::testing as inbox;
use mulac_poem::io::{CommandingAdapter, EventingAdapter, InboxAdapter, OutboxAdapter};
use poem::{http::StatusCode, test::TestClient};
use serde_json::{Value, json};

use crate::orchestration::AppState;

pub fn state() -> AppState {
    AppState {
        inbox: InboxAdapter::new(inbox::recording_service().0),
        commands: CommandingAdapter::new(commanding::empty_service()),
        events: EventingAdapter::new(eventing::empty_service()),
        outbox: OutboxAdapter::unconfigured(),
    }
}

fn client(state: AppState) -> TestClient<poem::Route> {
    TestClient::new(crate::app(state))
}

#[tokio::test]
async fn runners_validate_requests_and_preserve_empty_batch_contracts() {
    let client = client(state());
    for queue in ["inbox", "commands", "events", "outbox"] {
        let path = format!("/api/{queue}/run");
        for body in inbox::invalid_run_requests() {
            let response = client.post(&path).body_json(&body).send().await;
            assert_eq!(
                response.0.status(),
                StatusCode::BAD_REQUEST,
                "{queue}: {body}"
            );
        }
        for body in inbox::valid_run_requests() {
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

#[tokio::test]
async fn command_and_event_batches_stop_at_the_requested_limit() {
    let mut state = state();
    state.commands = CommandingAdapter::new(commanding::queued_service(3));
    state.events = EventingAdapter::new(eventing::queued_service(3));
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

#[tokio::test]
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

#[tokio::test]
async fn inbox_validates_envelopes_and_returns_the_durably_stored_id() {
    let (service, store) = inbox::recording_service();
    let mut state = state();
    state.inbox = InboxAdapter::new(service);
    let client = client(state);
    for body in inbox::invalid_submissions() {
        client
            .post("/api/inbox")
            .body_json(&body)
            .send()
            .await
            .assert_status(StatusCode::BAD_REQUEST);
    }
    assert!(store.stored_ids().is_empty());
    for body in inbox::valid_submissions() {
        let response = client.post("/api/inbox").body_json(&body).send().await;
        response.assert_status(StatusCode::ACCEPTED);
        let body: Value = response.json().await.value().deserialize();
        assert_eq!(body["message_id"], *store.stored_ids().last().unwrap());
    }
}

#[tokio::test]
async fn inbox_submission_sanitizes_storage_and_task_failures() {
    for (service, expected) in [
        (
            inbox::failing_storage_service(),
            "inbox message could not be stored",
        ),
        (inbox::panicking_service(), "inbox submission task failed"),
    ] {
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
        response.assert_text(expected).await;
    }
}

#[tokio::test]
async fn runner_storage_failures_are_batch_reports() {
    let mut state = state();
    state.inbox = InboxAdapter::new(inbox::failing_storage_service());
    state.commands = CommandingAdapter::new(commanding::failing_reservation_service());
    state.events = EventingAdapter::new(eventing::failing_reservation_service());
    let client = client(state);
    for (queue, error) in [
        ("inbox", inbox::RESERVATION_FAILURE),
        ("commands", commanding::RESERVATION_FAILURE),
        ("events", eventing::RESERVATION_FAILURE),
    ] {
        let response = client
            .post(format!("/api/{queue}/run"))
            .body_json(&json!({"limit":1}))
            .send()
            .await;
        response.assert_status_is_ok();
        response
            .assert_json(json!({"processed":null,"errors":[error]}))
            .await;
    }
}

#[tokio::test]
async fn task_panics_are_sanitized_for_every_runner() {
    let mut state = state();
    state.inbox = InboxAdapter::new(inbox::panicking_service());
    state.commands = CommandingAdapter::new(commanding::panicking_service());
    state.events = EventingAdapter::new(eventing::panicking_service());
    state.outbox = OutboxAdapter::new(outbox_api::io::testing::panicking_service());
    let client = client(state);
    for queue in ["inbox", "commands", "events", "outbox"] {
        let response = client
            .post(format!("/api/{queue}/run"))
            .body_json(&json!({"limit":1}))
            .send()
            .await;
        response.assert_status(StatusCode::INTERNAL_SERVER_ERROR);
        response.assert_text("queue processing task failed").await;
    }
}

#[tokio::test]
async fn event_partial_failures_retain_successful_items() {
    let (service, completed) = eventing::partially_failing_service();
    let mut state = state();
    state.events = EventingAdapter::new(service);
    let response = client(state)
        .post("/api/events/run")
        .body_json(&json!({"limit":10}))
        .send()
        .await;
    response.assert_status_is_ok();
    response
        .assert_json(json!({"processed":null,"errors":[eventing::SUBSCRIBER_FAILURE]}))
        .await;
    assert_eq!(*completed.lock().unwrap(), 2);
}

#[tokio::test]
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
