use super::*;
use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use http_body_util::BodyExt;
use outbox_api::io::testing;
use serde_json::{Value, json};
use tower::ServiceExt;

pub(crate) async fn run(state: AppState, body: Value) -> (StatusCode, Value) {
    let request = Request::post("/api/outbox/run")
        .header("content-type", "application/json")
        .body(Body::from(body.to_string()))
        .unwrap();
    let response = crate::app(state).oneshot(request).await.unwrap();
    let status = response.status();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let body =
        serde_json::from_slice(&bytes).unwrap_or_else(|_| json!(String::from_utf8_lossy(&bytes)));
    (status, body)
}

#[tokio::test]
async fn validates_before_reporting_missing_publisher() {
    for body in testing::invalid_run_requests() {
        assert_eq!(
            run(AppState::default(), body).await.0,
            StatusCode::BAD_REQUEST
        );
    }
    for body in testing::valid_run_requests() {
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
            crate::app(AppState::default())
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
        crate::app(AppState::default())
            .oneshot(request)
            .await
            .unwrap()
            .status(),
        StatusCode::PAYLOAD_TOO_LARGE
    );
}

#[tokio::test]
async fn delegates_batch_results_and_sanitizes_panics() {
    for (service, status, result) in [
        (
            testing::empty_service(),
            StatusCode::OK,
            json!({"processed":0,"errors":[]}),
        ),
        (
            testing::queued_service(3),
            StatusCode::OK,
            json!({"processed":2,"errors":[]}),
        ),
        (
            testing::failing_reservation_service(),
            StatusCode::OK,
            json!({"processed":null,"errors":[testing::RESERVATION_FAILURE]}),
        ),
        (
            testing::panicking_service(),
            StatusCode::INTERNAL_SERVER_ERROR,
            json!("queue processing task failed"),
        ),
    ] {
        let state = AppState {
            outbox: Some(OutboxAdapter::new(service)),
            ..Default::default()
        };
        assert_eq!(run(state, json!({"limit":2})).await, (status, result));
    }
}

#[tokio::test]
async fn documents_shared_schemas_with_bounds_and_errors() {
    let response = crate::app(AppState::default())
        .oneshot(
            Request::get("/api/openapi.json")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let spec: Value = serde_json::from_slice(&bytes).unwrap();
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
