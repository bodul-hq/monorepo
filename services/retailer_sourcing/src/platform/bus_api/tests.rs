use std::time::Duration;

use axum::Router;
use axum::body::Body;
use axum::extract::DefaultBodyLimit;
use axum::http::{Request, StatusCode, header};
use commanding::prelude::axum::CommandingAdapter;
use commanding::prelude::{AsyncCommandingService, testing as commands};
use diesel::PgConnection;
use diesel::r2d2::ConnectionManager;
use eventing::prelude::axum::EventingAdapter;
use eventing::prelude::{AsyncEventingService, testing as events};
use http_body_util::BodyExt;
use serde_json::{Value, json};
use tower::ServiceExt;

use crate::platform::bus::Bus;
use crate::platform::db::DbPool;
use crate::platform::state::AppState;

const PATHS: [&str; 2] = ["/api/commands/run", "/api/events/run"];

fn configured(
    commands: impl Into<AsyncCommandingService>,
    events: impl Into<AsyncEventingService>,
) -> Router {
    let pool = DbPool::builder()
        .max_size(1)
        .min_idle(Some(0))
        .connection_timeout(Duration::from_millis(50))
        .build_unchecked(ConnectionManager::<PgConnection>::new(
            "host=/dev/null/retailer-sourcing-test dbname=test connect_timeout=1",
        ));
    let mut bus = Bus::default().start(pool).unwrap();
    bus.commands = CommandingAdapter::new(commands);
    bus.events = EventingAdapter::new(events);
    crate::app(AppState {
        bus: Some(bus),
        ..AppState::default()
    })
}

pub(super) async fn send(
    app: &Router,
    path: &str,
    content_type: Option<&str>,
    body: impl Into<Body>,
) -> (StatusCode, String) {
    let mut request = Request::post(path);
    if let Some(content_type) = content_type {
        request = request.header(header::CONTENT_TYPE, content_type);
    }
    let response = app
        .clone()
        .oneshot(request.body(body.into()).unwrap())
        .await
        .unwrap();
    let status = response.status();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    (status, String::from_utf8(bytes.to_vec()).unwrap())
}

pub(super) async fn run(app: &Router, path: &str, limit: usize) -> Value {
    let body = json!({"limit":limit}).to_string();
    let (status, text) = send(app, path, Some("application/json"), body).await;
    assert_eq!(status, StatusCode::OK, "{path}: {text}");
    serde_json::from_str(&text).unwrap()
}

#[tokio::test]
async fn adapters_validate_before_checking_configuration() {
    let available = configured(commands::empty_consumer(), events::empty_consumer());
    let unavailable = crate::app(AppState::default());
    for path in PATHS {
        for app in [&available, &unavailable] {
            for body in commands::invalid_run_requests() {
                let (status, _) = send(app, path, Some("application/json"), body.to_string()).await;
                assert_eq!(status, StatusCode::BAD_REQUEST, "{path}: {body}");
            }
            for (body, expected) in commands::run_request_error_messages() {
                let (status, text) =
                    send(app, path, Some("application/json"), body.to_string()).await;
                assert_eq!(status, StatusCode::BAD_REQUEST);
                assert_eq!(text, expected);
            }
        }
        for body in commands::valid_run_requests() {
            let (status, text) =
                send(&available, path, Some("application/json"), body.to_string()).await;
            assert_eq!(status, StatusCode::OK);
            assert_eq!(
                serde_json::from_str::<Value>(&text).unwrap(),
                json!({"processed":0,"errors":[]})
            );
            let (status, text) = send(
                &unavailable,
                path,
                Some("application/json"),
                body.to_string(),
            )
            .await;
            assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
            assert_eq!(
                serde_json::from_str::<Value>(&text).unwrap(),
                json!({
                    "error":"the command bus is not running; start the service with DATABASE_URL"
                })
            );
        }
    }
}

#[tokio::test]
async fn adapters_preserve_body_rejections() {
    let app = crate::app(AppState::default());
    let limited = app.clone().layer(DefaultBodyLimit::max(8));
    for path in PATHS {
        assert_eq!(
            send(&app, path, Some("application/json"), "{").await.0,
            StatusCode::BAD_REQUEST
        );
        for content_type in [None, Some("text/plain")] {
            assert_eq!(
                send(&app, path, content_type, r#"{"limit":1}"#).await.0,
                StatusCode::UNSUPPORTED_MEDIA_TYPE
            );
        }
        assert_eq!(
            send(&limited, path, Some("application/json"), r#"{"limit":1}"#)
                .await
                .0,
            StatusCode::PAYLOAD_TOO_LARGE
        );
    }
}

#[tokio::test]
async fn each_adapter_processes_only_its_requested_batch() {
    let app = configured(commands::queued_consumer(3), events::queued_consumer(3));
    for path in PATHS {
        for processed in [2, 1, 0] {
            assert_eq!(
                run(&app, path, 2).await,
                json!({"processed":processed,"errors":[]})
            );
        }
    }
}

#[tokio::test]
async fn consumer_errors_remain_batch_results() {
    let app = configured(
        commands::failing_reservation_consumer(),
        events::failing_reservation_consumer(),
    );
    for path in PATHS {
        crate::tests::assert_batch_errors(&run(&app, path, 1).await, 1);
    }
    let (event_service, completed) = events::partially_failing_consumer();
    let app = configured(commands::failing_handler_consumer(), event_service);
    crate::tests::assert_batch_errors(&run(&app, PATHS[0], 1).await, 1);
    crate::tests::assert_batch_errors(&run(&app, PATHS[1], 10).await, 1);
    assert_eq!(*completed.lock().unwrap(), 2);
}

#[tokio::test]
async fn adapter_task_failures_are_sanitized() {
    let app = configured(commands::panicking_consumer(), events::panicking_consumer());
    for path in PATHS {
        let (status, text) = send(&app, path, Some("application/json"), r#"{"limit":1}"#).await;
        assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
        crate::tests::assert_error_id(&text);
    }
}

#[tokio::test]
async fn openapi_uses_shared_schemas_and_documents_error_formats() {
    let response = crate::app(AppState::default())
        .oneshot(
            Request::get("/api/openapi.json")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let spec: Value = serde_json::from_slice(&body).unwrap();
    for path in ["/commands/run", "/events/run"] {
        let operation = &spec["paths"][path]["post"];
        assert_eq!(
            operation["requestBody"]["content"]["application/json"]["schema"]["$ref"],
            "#/components/schemas/RunRequest"
        );
        assert_eq!(
            operation["responses"]["200"]["content"]["application/json"]["schema"]["$ref"],
            "#/components/schemas/RunResponse"
        );
        for status in ["400", "413", "415", "500"] {
            assert_eq!(
                operation["responses"][status]["content"]["text/plain"]["schema"]["type"],
                "string"
            );
        }
        assert_eq!(
            operation["responses"]["503"]["content"]["application/json"]["schema"]["$ref"],
            "#/components/schemas/ErrorBody"
        );
    }
    let schemas = spec["components"]["schemas"].as_object().unwrap();
    let run_names: Vec<_> = schemas
        .keys()
        .filter(|name| name.starts_with("Run"))
        .map(String::as_str)
        .collect();
    assert_eq!(run_names, ["RunRequest", "RunResponse"]);
    assert_eq!(schemas["RunRequest"]["properties"]["limit"]["minimum"], 1);
    assert_eq!(
        schemas["RunRequest"]["properties"]["limit"]["maximum"],
        1000
    );
}
