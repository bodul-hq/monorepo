use bootstrap::database_pool;
use env::VarError as EnvVarError;
use env::var as env_var;
use io::Error as IoError;
use io::ErrorKind as IoErrorKind;
use io::Result as IoResult;
use kernel::boot;
#[cfg(not(unix))]
use std::future::pending;
use std::io::stderr;
use tokio::main as tokio_main;
#[cfg(unix)]
use tokio::select;
use tokio::signal::ctrl_c;
#[cfg(unix)]
use tokio::signal::unix::SignalKind;
#[cfg(unix)]
use tokio::signal::unix::signal as unix_signal;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::fmt;
mod app;
use app::app;
use reporting::DEFAULT_REPORTING_TOKEN;
mod bootstrap;
pub mod infra;
mod monitoring;
mod orchestration;
mod reporting;
use commanding::prelude::{AsyncCommandingService, poem::CommandingAdapter};
#[cfg(test)]
use diesel::{PgConnection, r2d2::ConnectionManager};
use eventing::prelude::{AsyncEventingService, poem::EventingAdapter};
use inbox::prelude::{AsyncInboxService, poem::InboxAdapter};
use infra::DbPool;
use kernel::{KernelConfig, KernelError, PersistentKernelHandle};
use outbox::prelude::poem::OutboxAdapter;
#[cfg(test)]
use poem::Route;
use poem::{Server, listener::TcpListener};
#[cfg(test)]
use std::time::Duration;
use std::{env, io, sync::Arc};
#[tokio_main]
async fn main() -> IoResult<()> {
    fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| "warn".into()))
        .with_writer(stderr)
        .init();
    let port = parse_port(env_var("PORT"))?;
    let pool = database_pool()?;
    let shutdown = shutdown_signal()?;

    // Start persistent queue infrastructure; domain registrations come with future features.
    let (kernel, state) = start_mulac(pool.clone()).map_err(err_kernel_could_not_start)?;
    let token =
        env_var("PRODUCTS_REPORTING_TOKEN").unwrap_or_else(|_| DEFAULT_REPORTING_TOKEN.to_owned());
    let result = Server::new(TcpListener::bind(format!("0.0.0.0:{port}")))
        .run_with_graceful_shutdown(app(state, token), shutdown, None)
        .await;
    kernel.shutdown();
    result
}

pub fn start_mulac(pool: DbPool) -> Result<(Arc<PersistentKernelHandle>, AppState), KernelError> {
    let kernel = boot(KernelConfig::default()).start_persistent(pool.clone(), 1)?;
    let inbox = InboxAdapter::new(AsyncInboxService::from_postgres(pool.clone()));
    let commanding = CommandingAdapter::new(AsyncCommandingService::new(kernel.command_consumer()));
    // No domain subscribers are registered by this foundation.
    let events = EventingAdapter::new(AsyncEventingService::new(kernel.event_consumer()));
    Ok((
        Arc::new(kernel),
        AppState {
            pool,
            inbox,
            commanding,
            events,
            // No delivery destination is defined yet.
            outbox: OutboxAdapter::unconfigured(),
        },
    ))
}

#[derive(Clone)]
pub struct AppState {
    pub pool: DbPool,
    pub inbox: InboxAdapter,
    pub commanding: CommandingAdapter,
    pub events: EventingAdapter,
    pub outbox: OutboxAdapter,
}

fn err_kernel_could_not_start(error: KernelError) -> IoError {
    IoError::other(format!("Mulac kernel could not start: {error}"))
}

// A concrete, lazy pool for tests that do not need successful database access.
// The nonexistent socket directory makes storage failures deterministic.
#[cfg(test)]
fn test_pool() -> DbPool {
    DbPool::builder()
        .max_size(1)
        .min_idle(Some(0))
        .connection_timeout(Duration::from_millis(50))
        .build_unchecked(ConnectionManager::<PgConnection>::new(
            "host=/dev/null/products-test dbname=products_test connect_timeout=1",
        ))
}

#[cfg(test)]
fn test_app(pool: DbPool) -> Route {
    let (_, state) = start_mulac(pool).expect("start test kernel");
    app(state, DEFAULT_REPORTING_TOKEN.to_owned())
}

/// Parses the `PORT` environment variable as returned by `env::var`.
/// An unset variable selects the default port, 3000.
fn parse_port(value: Result<String, EnvVarError>) -> IoResult<u16> {
    match value {
        Ok(value) => value.parse().map_err(|_| {
            IoError::new(
                IoErrorKind::InvalidInput,
                "PORT must be an integer between 0 and 65535",
            )
        }),
        Err(EnvVarError::NotPresent) => Ok(3000),
        Err(error) => Err(IoError::new(IoErrorKind::InvalidInput, error)),
    }
}

/// Resolves on Ctrl+C or, on Unix, SIGTERM, so in-flight requests can finish
/// before the server exits.
fn shutdown_signal() -> IoResult<impl Future<Output = ()> + Send + 'static> {
    #[cfg(unix)]
    let mut terminate = unix_signal(SignalKind::terminate())?;
    Ok(async move {
        #[cfg(unix)]
        select! {
            Ok(()) = ctrl_c() => {}
            _ = terminate.recv() => {}
        }
        #[cfg(not(unix))]
        if ctrl_c().await.is_err() {
            pending::<()>().await;
        }
    })
}

#[cfg(test)]
mod tests {
    use poem::test::TestResponse;
    use tokio::test as tokio_test;
    use uuid::Uuid;

    pub(crate) async fn assert_error_response(response: TestResponse) {
        let body = response.0.into_body().into_string().await.unwrap();
        assert_eq!(Uuid::parse_str(&body).unwrap().get_version_num(), 7);
    }

    use std::{env::VarError, io::ErrorKind};

    use poem::{http::StatusCode, test::TestClient};
    use serde_json::{Value, json};

    use super::{parse_port, test_app as app, test_pool};

    #[tokio_test]
    async fn health_returns_ok() {
        let response = TestClient::new(app(test_pool()))
            .get("/api/health")
            .send()
            .await;
        response.assert_status_is_ok();
        response.assert_text("ok").await;
    }

    #[tokio_test]
    async fn graphql_health_returns_ok() {
        let response = TestClient::new(app(test_pool()))
            .post("/graphql")
            .body_json(&json!({ "query": "{ health }" }))
            .send()
            .await;
        response.assert_status_is_ok();
        response
            .assert_json(json!({ "data": { "health": "ok" } }))
            .await;
    }

    #[tokio_test]
    async fn graphql_get_query_returns_ok() {
        let response = TestClient::new(app(test_pool()))
            .get("/graphql")
            .query("query", &"{ health }")
            .send()
            .await;
        response.assert_status_is_ok();
        response
            .assert_json(json!({ "data": { "health": "ok" } }))
            .await;
    }

    #[tokio_test]
    async fn graphiql_is_served_only_in_debug_builds() {
        let response = TestClient::new(app(test_pool()))
            .get("/graphql/ide")
            .send()
            .await;
        if cfg!(debug_assertions) {
            response.assert_status_is_ok();
        } else {
            response.assert_status(StatusCode::NOT_FOUND);
        }
    }

    #[tokio_test]
    async fn openapi_spec_matches_routes() {
        let client = TestClient::new(app(test_pool()));
        let response = client.get("/api/openapi.json").send().await;
        response.assert_status_is_ok();
        let spec: Value = response.json().await.value().deserialize();

        assert!(
            spec["paths"]["/health"]["get"].is_object(),
            "OpenAPI must document GET /health",
        );
        let submission = &spec["paths"]["/inbox"]["post"];
        assert!(submission["responses"]["202"].is_object());
        assert_eq!(
            submission["requestBody"]["content"]["application/json; charset=utf-8"]["schema"]["$ref"],
            "#/components/schemas/SubmitInboxRequest"
        );
        assert_eq!(
            submission["responses"]["202"]["content"]["application/json; charset=utf-8"]["schema"]
                ["$ref"],
            "#/components/schemas/SubmitInboxResponse"
        );
        let request = &spec["components"]["schemas"]["SubmitInboxRequest"];
        assert_eq!(
            request["required"],
            json!(["payload", "routing_key", "source"])
        );
        for field in [
            "payload",
            "routing_key",
            "message_id",
            "correlation_id",
            "source",
        ] {
            assert!(request["properties"][field].is_object());
        }
        for queue in ["inbox", "commands", "events", "outbox"] {
            assert!(spec["paths"][format!("/{queue}/run")]["post"].is_object());
        }
        let limit = &spec["components"]["schemas"]["RunRequest"]["properties"]["limit"];
        assert_eq!(limit["minimum"], 1.0);
        assert_eq!(limit["maximum"], 1000.0);
        assert_eq!(spec["servers"][0]["url"], "/api");

        let server = spec["servers"][0]["url"].as_str().unwrap_or("");
        for path in spec["paths"].as_object().unwrap().keys() {
            let response = client.get(format!("{server}{path}")).send().await;
            assert_ne!(response.0.status(), StatusCode::NOT_FOUND, "{server}{path}");
        }
    }

    #[tokio_test]
    async fn swagger_ui_is_served() {
        let response = TestClient::new(app(test_pool()))
            .get("/api/docs/")
            .send()
            .await;
        response.assert_status_is_ok();
    }

    #[tokio_test]
    async fn queue_routes_validate_requests_and_report_failures() {
        let client = TestClient::new(app(test_pool()));
        for queue in ["inbox", "commands", "events", "outbox"] {
            let path = format!("/api/{queue}/run");
            for body in [
                json!({}),
                json!({"limit":0}),
                json!({"limit":1001}),
                json!({"limit":-1}),
                json!({"limit":1.5}),
                json!({"limit":"1"}),
            ] {
                client
                    .post(&path)
                    .body_json(&body)
                    .send()
                    .await
                    .assert_status(StatusCode::BAD_REQUEST);
            }
            client
                .post(&path)
                .content_type("application/json")
                .body("{")
                .send()
                .await
                .assert_status(StatusCode::BAD_REQUEST);
            for limit in [1, 1000] {
                let response = client
                    .post(&path)
                    .body_json(&json!({"limit":limit}))
                    .send()
                    .await;
                if queue == "outbox" {
                    response.assert_status(StatusCode::SERVICE_UNAVAILABLE);
                    response
                        .assert_text("outbox publisher is not configured")
                        .await;
                } else {
                    response.assert_status_is_ok();
                    let body: Value = response.json().await.value().deserialize();
                    assert_eq!(body["processed"], Value::Null);
                    assert!(!body["errors"].as_array().unwrap().is_empty());
                }
            }
        }
    }

    #[test]
    fn port_defaults_when_unset() {
        assert_eq!(parse_port(Err(VarError::NotPresent)).unwrap(), 3000);
    }

    #[test]
    fn port_accepts_the_full_range() {
        for (value, port) in [("0", 0), ("80", 80), ("3001", 3001), ("65535", 65535)] {
            assert_eq!(parse_port(Ok(value.to_owned())).unwrap(), port, "{value:?}");
        }
    }

    #[test]
    fn port_rejects_invalid_values() {
        // Non-numeric, out of range, padded, and non-ASCII Unicode digits.
        for value in [
            "",
            "abc",
            "80a",
            "3000.0",
            "-1",
            "65536",
            "99999999999",
            " 3000",
            "3000\n",
            "３０００",
            "٣٠٠٠",
        ] {
            let error = parse_port(Ok(value.to_owned())).unwrap_err();
            assert_eq!(error.kind(), ErrorKind::InvalidInput, "{value:?}");
        }
    }

    #[cfg(unix)]
    #[test]
    fn port_rejects_non_unicode_values() {
        use std::{ffi::OsString, os::unix::ffi::OsStringExt};

        let value = OsString::from_vec(vec![0xff]);
        let error = parse_port(Err(VarError::NotUnicode(value))).unwrap_err();
        assert_eq!(error.kind(), ErrorKind::InvalidInput);
    }
}

#[cfg(test)]
mod postgres_tests;
