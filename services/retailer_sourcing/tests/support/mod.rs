#![allow(dead_code)]
use axum::{
    Json, Router,
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
};
use retailer_sourcing::object_storage::ObjectStorage;
use serde_json::{Value, json};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};
use tokio::runtime::Runtime;
use uuid::Uuid;

pub const TOKEN: &str = "sitemap-object-test-token";
#[derive(Default)]
pub struct Data {
    pub uploads: Vec<(String, Value)>,
    pub records: HashMap<Uuid, Value>,
    pub fail_create_at: Option<usize>,
    pub get_response: Option<(u16, String)>,
    pub reads: Vec<(String, Uuid)>,
    /// Gate one bucket's reads to exercise concurrent initial preparation.
    pub read_barrier: Option<(String, Arc<tokio::sync::Barrier>)>,
}
pub struct Server {
    pub runtime: Runtime,
    pub url: String,
    pub data: Arc<Mutex<Data>>,
}
impl Server {
    pub fn new() -> Self {
        let runtime = Runtime::new().unwrap();
        let data = Arc::new(Mutex::new(Data::default()));
        let router = Router::new()
            .route("/api/buckets/{bucket}/records", post(create))
            .route("/api/buckets/{bucket}/records/{id}", get(retrieve))
            .with_state(data.clone());
        let listener = runtime
            .block_on(tokio::net::TcpListener::bind("127.0.0.1:0"))
            .unwrap();
        let url = format!("http://{}", listener.local_addr().unwrap());
        runtime.spawn(async move {
            axum::serve(listener, router).await.unwrap();
        });
        Self { runtime, url, data }
    }
    pub fn client(&self) -> ObjectStorage {
        ObjectStorage::new(&self.url, TOKEN, self.runtime.handle().clone()).unwrap()
    }
}
fn authorized(headers: &HeaderMap) -> bool {
    headers
        .get("authorization")
        .is_some_and(|v| v == format!("Bearer {TOKEN}").as_str())
}
async fn create(
    State(data): State<Arc<Mutex<Data>>>,
    Path(bucket): Path<String>,
    headers: HeaderMap,
    Json(input): Json<Value>,
) -> Response {
    if !authorized(&headers) {
        return StatusCode::UNAUTHORIZED.into_response();
    }
    let mut data = data.lock().unwrap();
    data.uploads.push((bucket.clone(), input.clone()));
    if data.fail_create_at == Some(data.uploads.len()) {
        return StatusCode::SERVICE_UNAVAILABLE.into_response();
    }
    if let Some(existing) = data
        .records
        .values()
        .find(|r| r["bucket"] == bucket && r["idempotency_key"] == input["idempotency_key"])
    {
        if existing["payload"] != input["payload"] || existing["metadata"] != input["metadata"] {
            return StatusCode::CONFLICT.into_response();
        }
        return Json(json!({"id": existing["id"]})).into_response();
    }
    let id = Uuid::now_v7();
    data.records.insert(id, json!({"id":id,"bucket":bucket,"idempotency_key":input["idempotency_key"],"payload":input["payload"],"metadata":input["metadata"],"created_at":"2026-10-15T00:00:00Z"}));
    (StatusCode::CREATED, Json(json!({"id":id}))).into_response()
}
async fn retrieve(
    State(data): State<Arc<Mutex<Data>>>,
    Path((bucket, id)): Path<(String, Uuid)>,
    headers: HeaderMap,
) -> Response {
    if !authorized(&headers) {
        return StatusCode::UNAUTHORIZED.into_response();
    }
    let barrier = data
        .lock()
        .unwrap()
        .read_barrier
        .as_ref()
        .filter(|(expected, _)| expected == &bucket)
        .map(|(_, barrier)| barrier.clone());
    if let Some(barrier) = barrier {
        barrier.wait().await;
    }
    let mut data = data.lock().unwrap();
    data.reads.push((bucket.clone(), id));
    if let Some((status, body)) = &data.get_response {
        return (
            StatusCode::from_u16(*status).unwrap(),
            [("content-type", "application/json")],
            body.clone(),
        )
            .into_response();
    }
    match data.records.get(&id).filter(|r| r["bucket"] == bucket) {
        Some(record) => Json(record.clone()).into_response(),
        None => StatusCode::NOT_FOUND.into_response(),
    }
}
