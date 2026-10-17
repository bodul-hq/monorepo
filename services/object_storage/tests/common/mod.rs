use axum::{
    Router,
    body::Body,
    http::{HeaderMap, Request, StatusCode, header},
};
use http_body_util::BodyExt;
use object_storage::BEARER_TOKEN;
use serde_json::{Value, json};
use tower::ServiceExt;

pub const CREATE_QUERY: &str =
    "mutation($input: CreateRecordInput!) { createRecord(input: $input) { id } }";

pub fn input() -> Value {
    json!({"idempotency_key": uuid::Uuid::now_v7().to_string(), "payload":"Živjo 🌍", "metadata":{"source":"test","ordered":[1,2]}})
}
pub fn gql_input(bucket: &str, input: &Value) -> Value {
    json!({"query":CREATE_QUERY, "variables":{"input":{"bucket":bucket,"idempotencyKey":input["idempotency_key"],"payload":input["payload"],"metadata":input["metadata"]}}})
}

pub async fn raw(
    app: &Router,
    method: &str,
    path: &str,
    token: Option<&str>,
    content_type: Option<&str>,
    body: impl Into<Body>,
) -> (StatusCode, HeaderMap, Vec<u8>) {
    let mut builder = Request::builder().method(method).uri(path);
    if let Some(token) = token {
        builder = builder.header(header::AUTHORIZATION, format!("Bearer {token}"));
    }
    if let Some(content_type) = content_type {
        builder = builder.header(header::CONTENT_TYPE, content_type);
    }
    let response = app
        .clone()
        .oneshot(builder.body(body.into()).unwrap())
        .await
        .unwrap();
    let (parts, body) = response.into_parts();
    (
        parts.status,
        parts.headers,
        body.collect().await.unwrap().to_bytes().to_vec(),
    )
}
pub async fn request(
    app: &Router,
    method: &str,
    path: &str,
    body: Value,
) -> (StatusCode, HeaderMap, Value) {
    let (status, headers, body) = raw(
        app,
        method,
        path,
        Some(BEARER_TOKEN),
        Some("application/json"),
        body.to_string(),
    )
    .await;
    let body = serde_json::from_slice(&body).unwrap_or_else(|_| {
        panic!(
            "Non-JSON response ({status}): {}",
            String::from_utf8_lossy(&body)
        )
    });
    (status, headers, body)
}
pub fn problem(status: StatusCode, headers: &HeaderMap, body: &Value) {
    assert_eq!(headers[header::CONTENT_TYPE], "application/problem+json");
    assert_eq!(body["type"], "about:blank");
    assert_eq!(body["title"], status.canonical_reason().unwrap());
    assert_eq!(body["status"], status.as_u16());
    assert!(body["detail"].is_string());
    assert_eq!(body.as_object().unwrap().len(), 4);
}
