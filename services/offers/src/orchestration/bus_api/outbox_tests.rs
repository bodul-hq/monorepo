use crate::app;
use crate::app_with_reporting_token;
use crate::orchestration::customer_facing_publisher::CustomerFacingPublisher;
use crate::start_mulac;
use crate::test_pool;
use crate::tests::assert_batch_response;
use crate::tests::assert_error_id;
use crate::tests::assert_error_response;
use outbox::prelude::poem::OutboxAdapter;
use outbox::prelude::{AsyncOutboxService, testing};
use outbox::prelude::{OutboundMessageEnvelope, OutboxError, OutboxPublisherPort};
use poem::Route;
use poem::{http::StatusCode, test::TestClient};
use serde_json::{Value, json};
use std::sync::Arc;
use testing::invalid_run_requests;
use testing::panicking_consumer;
use testing::partially_failing_consumer;
use testing::publisher_consumer;
use testing::valid_run_requests;
use tokio::runtime::Handle;
use tokio::test as tokio_test;

struct UnusedPublisher;
impl OutboxPublisherPort for UnusedPublisher {
    fn publish(&self, _: OutboundMessageEnvelope) -> Result<(), OutboxError> {
        panic!("storage failure must prevent publication")
    }
}

fn client_with(outbox: OutboxAdapter) -> TestClient<Route> {
    let (_, mut state) = start_mulac(test_pool()).unwrap();
    state.outbox = outbox;
    TestClient::new(app_with_reporting_token(state, "test".into()))
}

#[tokio_test]
async fn configured_outbox_reports_storage_failure() {
    let client = client_with(OutboxAdapter::new(AsyncOutboxService::from_postgres(
        test_pool(),
        Arc::new(UnusedPublisher),
    )));
    for limit in [1, 1000] {
        let response = client
            .post("/api/outbox/run")
            .body_json(&json!({"limit":limit}))
            .send()
            .await;
        response.assert_status_is_ok();
        let body: Value = response.json().await.value().deserialize();
        assert!(body["processed"].is_null());
        let errors = body["errors"].as_array().unwrap();
        assert_eq!(errors.len(), 1);
        assert_error_id(errors[0].as_str().unwrap());
    }
}

#[tokio_test]
async fn outbox_run_validates_the_shared_run_request_table() {
    let client = TestClient::new(app(test_pool()));
    for body in valid_run_requests() {
        // Validation passed when the unconfigured publisher is reached.
        let response = client.post("/api/outbox/run").body_json(&body).send().await;
        response.assert_status(StatusCode::SERVICE_UNAVAILABLE);
        response
            .assert_text("outbox publisher is not configured")
            .await;
    }
    for body in invalid_run_requests() {
        let response = client.post("/api/outbox/run").body_json(&body).send().await;
        assert_eq!(response.0.status(), StatusCode::BAD_REQUEST, "{body}");
    }
    // Poem's RunRequest wrapper validates while parsing, so it prefixes the message.
    let response = client
        .post("/api/outbox/run")
        .body_json(&json!({"limit":0}))
        .send()
        .await;
    response.assert_status(StatusCode::BAD_REQUEST);
    response
        .assert_text(
            "parse request payload error: failed to parse \"RunRequest\": limit must be between 1 and 1000",
        )
        .await;
}

#[tokio_test]
async fn outbox_run_requires_json_content_type() {
    let client = TestClient::new(app(test_pool()));
    for content_type in [None, Some("text/plain")] {
        let mut request = client.post("/api/outbox/run").body(r#"{"limit":1}"#);
        if let Some(content_type) = content_type {
            request = request.content_type(content_type);
        }
        request
            .send()
            .await
            .assert_status(StatusCode::UNSUPPORTED_MEDIA_TYPE);
    }
}

#[tokio_test]
async fn outbox_run_publishes_the_shared_run_schemas() {
    let client = TestClient::new(app(test_pool()));
    let spec: Value = client
        .get("/api/openapi.json")
        .send()
        .await
        .json()
        .await
        .value()
        .deserialize();
    let run = &spec["paths"]["/outbox/run"]["post"];
    assert_eq!(
        run["summary"],
        "Publish one batch to customer-facing's inbox; returns 503 when delivery is not configured."
    );
    assert_eq!(
        run["requestBody"]["content"]["application/json; charset=utf-8"]["schema"]["$ref"],
        "#/components/schemas/RunRequest"
    );
    assert_eq!(
        run["responses"]["200"]["content"]["application/json; charset=utf-8"]["schema"]["$ref"],
        "#/components/schemas/RunResponse"
    );
    assert_eq!(
        run["responses"]
            .as_object()
            .unwrap()
            .keys()
            .collect::<Vec<_>>(),
        vec!["200"]
    );
}

#[tokio_test]
async fn outbox_run_keeps_published_items_of_a_partial_batch() {
    let (service, outcomes) = partially_failing_consumer();
    let response = client_with(OutboxAdapter::new(service))
        .post("/api/outbox/run")
        .body_json(&json!({"limit":5}))
        .send()
        .await;
    response.assert_status_is_ok();
    assert_batch_response(response, 1).await;
    assert_eq!(
        *outcomes.lock().unwrap(),
        vec!["completed", "failed", "completed"]
    );
}

#[tokio_test]
async fn outbox_run_sanitizes_blocking_task_failures() {
    let response = client_with(OutboxAdapter::new(panicking_consumer()))
        .post("/api/outbox/run")
        .body_json(&json!({"limit":1}))
        .send()
        .await;
    response.assert_status(StatusCode::INTERNAL_SERVER_ERROR);
    assert_error_response(response).await;
}

#[tokio_test(flavor = "multi_thread")]
async fn outbox_run_reports_distinct_ids_for_customer_facing_publisher_errors() {
    // Mulac's test rows are not routed to customer_facing, so each fails
    // before any request is sent.
    let publisher = CustomerFacingPublisher::new("http://127.0.0.1:9", Handle::current()).unwrap();
    let service = publisher_consumer(2, Arc::new(publisher));
    let response = client_with(OutboxAdapter::new(service))
        .post("/api/outbox/run")
        .body_json(&json!({"limit":5}))
        .send()
        .await;
    response.assert_status_is_ok();
    assert_batch_response(response, 2).await;
}
