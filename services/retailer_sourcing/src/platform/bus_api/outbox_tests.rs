use super::*;
use crate::app;
use crate::tests::assert_batch_errors;
use crate::tests::assert_error_id;
use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use http_body_util::BodyExt;
use outbox::prelude::testing;
use serde_json::from_slice;
use serde_json::{Value, json};
use testing::RESERVATION_FAILURE;
use testing::empty_consumer;
use testing::failing_reservation_consumer;
use testing::invalid_run_requests;
use testing::panicking_consumer;
use testing::queued_consumer;
use testing::valid_run_requests;
use tokio::test as tokio_test;
use tower::ServiceExt;

pub(crate) async fn run(state: AppState, body: Value) -> (StatusCode, Value) {
    let request = Request::post("/api/outbox/run")
        .header("content-type", "application/json")
        .body(Body::from(body.to_string()))
        .unwrap();
    let response = app(state).oneshot(request).await.unwrap();
    let status = response.status();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let body = from_slice(&bytes).unwrap_or_else(|_| json!(String::from_utf8_lossy(&bytes)));
    (status, body)
}

#[tokio_test]
async fn validates_before_reporting_missing_publisher() {
    for body in invalid_run_requests() {
        assert_eq!(
            run(AppState::default(), body).await.0,
            StatusCode::BAD_REQUEST
        );
    }
    for body in valid_run_requests() {
        assert_eq!(
            run(AppState::default(), body).await,
            (
                StatusCode::SERVICE_UNAVAILABLE,
                json!("outbox publisher is not configured")
            )
        );
    }
    for (content_type, body, status) in [
        (None, "{\"limit\":1}", StatusCode::UNSUPPORTED_MEDIA_TYPE),
        (
            Some("text/plain"),
            "{\"limit\":1}",
            StatusCode::UNSUPPORTED_MEDIA_TYPE,
        ),
        (Some("application/json"), "{", StatusCode::BAD_REQUEST),
    ] {
        let mut request = Request::post("/api/outbox/run");
        if let Some(value) = content_type {
            request = request.header("content-type", value);
        }
        assert_eq!(
            app(AppState::default())
                .oneshot(request.body(Body::from(body)).unwrap())
                .await
                .unwrap()
                .status(),
            status
        );
    }
    let request = Request::post("/api/outbox/run")
        .header("content-type", "application/json")
        .body(Body::from(" ".repeat(2 * 1024 * 1024 + 1)))
        .unwrap();
    assert_eq!(
        app(AppState::default())
            .oneshot(request)
            .await
            .unwrap()
            .status(),
        StatusCode::PAYLOAD_TOO_LARGE
    );
}

#[tokio_test]
async fn delegates_batch_results_and_sanitizes_panics() {
    for (service, status, result) in [
        (
            empty_consumer(),
            StatusCode::OK,
            json!({"processed":0,"errors":[]}),
        ),
        (
            queued_consumer(3),
            StatusCode::OK,
            json!({"processed":2,"errors":[]}),
        ),
        (
            failing_reservation_consumer(),
            StatusCode::OK,
            json!({"processed":null,"errors":[RESERVATION_FAILURE]}),
        ),
        (
            panicking_consumer(),
            StatusCode::INTERNAL_SERVER_ERROR,
            json!("queue processing task failed"),
        ),
    ] {
        let state = AppState {
            outbox: Some(OutboxAdapter::new(service)),
            ..Default::default()
        };
        let (actual_status, body) = run(state, json!({"limit":2})).await;
        assert_eq!(actual_status, status);
        if status == StatusCode::INTERNAL_SERVER_ERROR {
            assert_error_id(body.as_str().unwrap());
        } else if result["processed"].is_null() {
            assert_batch_errors(&body, 1);
        } else {
            assert_eq!(body, result);
        }
    }
}

#[tokio_test]
async fn documents_shared_schemas_with_bounds_and_errors() {
    let response = app(AppState::default())
        .oneshot(
            Request::get("/api/openapi.json")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let spec: Value = from_slice(&bytes).unwrap();
    let operation = &spec["paths"]["/outbox/run"]["post"];
    assert_eq!(
        operation["requestBody"]["content"]["application/json"]["schema"]["$ref"],
        "#/components/schemas/RunRequest"
    );
    assert_eq!(
        operation["responses"]["200"]["content"]["application/json"]["schema"]["$ref"],
        "#/components/schemas/RunResponse"
    );
    assert_eq!(
        spec["components"]["schemas"]["RunRequest"]["properties"]["limit"]["minimum"],
        1
    );
    assert_eq!(
        spec["components"]["schemas"]["RunRequest"]["properties"]["limit"]["maximum"],
        1000
    );
    for status in ["400", "413", "415", "500", "503"] {
        assert!(operation["responses"][status].is_object());
    }
}
