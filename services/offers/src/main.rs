//! Retailer offers: domain, application, persistence, and HTTP boundaries.

pub mod app;
pub mod bootstrap;
pub mod infra;
pub mod monitoring;
pub mod offers;
pub mod orchestration;
pub mod read_offer;
pub mod read_offers_identifiers;
pub mod reporting;
pub mod subscribers;

#[cfg(test)]
mod postgres_tests;

use std::{env, io, sync::Arc};

use async_graphql::{EmptyMutation, EmptySubscription, Object, Schema, http::GraphiQLSource};
use async_graphql_poem::GraphQL;

use kernel::{KernelConfig, KernelError, PersistentKernelHandle};

use poem::{Route, Server, get, handler, listener::TcpListener, web::Html};
use poem_openapi::OpenApiService;
use testimony_infra::prelude::DbPool;

use crate::infra::repository::offer_handler;
use crate::monitoring::health::HealthApi;
use crate::offers::{
    command::CREATE_FROM_OFFER_DETAILS_COMMAND,
    event::{OFFER_CREATED_EVENT, OFFER_PRICE_CHANGED_EVENT, OFFER_TITLE_CHANGED_EVENT},
    http::rest::OffersApi,
};
use crate::orchestration::customer_facing_publisher::CustomerFacingPublisher;
use crate::orchestration::{CommandingApi, EventingApi, InboxApi, OutboxApi};
use crate::read_offer::{
    projector::RetailerOfferProjector, repository::PostgresRetailerOfferRepository,
};
use crate::read_offers_identifiers::{
    projector::RetailerOfferIdentifiersProjector,
    repository::PostgresRetailerOfferIdentifiersRepository,
};
use crate::reporting::{DEFAULT_REPORTING_TOKEN, ReportingApi};
use commanding_api::io::CommandingService;
use eventing_api::io::EventingService;
use inbox_api::io::InboxService;
use mulac_poem::io::{CommandingAdapter, EventingAdapter, InboxAdapter, OutboxAdapter};
use outbox_api::io::OutboxService;

use crate::subscribers::customer_facing::CustomerFacingSubscriber;

#[cfg(test)]
use diesel::{PgConnection, r2d2::ConnectionManager};
#[cfg(test)]
use std::time::Duration;

#[tokio::main]
async fn main() -> io::Result<()> {
    let port = parse_port(env::var("PORT"))?;
    // Checked before connecting; customer-facing may be down at startup.
    let publisher = CustomerFacingPublisher::from_env(tokio::runtime::Handle::current())?;
    let pool = bootstrap::database_pool()?.expect("DATABASE_URL is required");
    let shutdown = shutdown_signal()?;

    // Boot the in-process kernel and register all command handlers and event subscribers.
    let (kernel, mut state) = start_mulac(pool.clone()).map_err(err_kernel_could_not_start)?;
    if let Some(publisher) = publisher {
        state.outbox = OutboxAdapter::new(OutboxService::from_postgres(
            pool.clone(),
            Arc::new(publisher),
        ));
    }
    let token =
        env::var("OFFERS_REPORTING_TOKEN").unwrap_or_else(|_| DEFAULT_REPORTING_TOKEN.to_owned());
    let result = Server::new(TcpListener::bind(format!("0.0.0.0:{port}")))
        .run_with_graceful_shutdown(app_with_reporting_token(state, token), shutdown, None)
        .await;
    kernel.shutdown();
    result
}

pub fn start_mulac(pool: DbPool) -> Result<(Arc<PersistentKernelHandle>, AppState), KernelError> {
    let handler = Arc::new(offer_handler(pool.clone()));
    let offer_projector = Arc::new(RetailerOfferProjector::new(
        PostgresRetailerOfferRepository::new(pool.clone()),
    ));
    let identifiers_projector = Arc::new(RetailerOfferIdentifiersProjector::new(
        PostgresRetailerOfferIdentifiersRepository::new(pool.clone()),
    ));
    let customer_facing = Arc::new(CustomerFacingSubscriber::new(pool.clone()));
    let kernel = kernel::boot(KernelConfig::default())
        .command_handler(CREATE_FROM_OFFER_DETAILS_COMMAND, handler)
        .event_subscriber(OFFER_CREATED_EVENT, "read_offer", offer_projector.clone())
        .event_subscriber(
            OFFER_TITLE_CHANGED_EVENT,
            "read_offer",
            offer_projector.clone(),
        )
        .event_subscriber(OFFER_PRICE_CHANGED_EVENT, "read_offer", offer_projector)
        .event_subscriber(
            OFFER_CREATED_EVENT,
            "read_offers_identifiers",
            identifiers_projector,
        )
        .event_subscriber(
            OFFER_CREATED_EVENT,
            "customer_facing",
            customer_facing.clone(),
        )
        .event_subscriber(
            OFFER_TITLE_CHANGED_EVENT,
            "customer_facing",
            customer_facing.clone(),
        )
        .event_subscriber(
            OFFER_PRICE_CHANGED_EVENT,
            "customer_facing",
            customer_facing,
        )
        .start_persistent(pool.clone(), 1)?;

    let inbox = InboxAdapter::new(InboxService::from_postgres(pool.clone()));
    let commanding = CommandingAdapter::new(CommandingService::new(kernel.command_consumer()));
    // The kernel's consumer dispatches to the subscribers registered above.
    let events = EventingAdapter::new(EventingService::new(kernel.event_consumer()));
    Ok((
        Arc::new(kernel),
        AppState {
            pool,
            inbox,
            commanding,
            events,
            // Delivery stays off until `main` configures a publisher.
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

fn err_kernel_could_not_start(error: KernelError) -> io::Error {
    io::Error::other(format!("Mulac kernel could not start: {error}"))
}

struct Query;

#[Object]
impl Query {
    async fn health(&self) -> &'static str {
        shared::health()
    }
}

#[handler]
async fn graphiql() -> Html<String> {
    Html(GraphiQLSource::build().endpoint("/graphql").finish())
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
            "host=/dev/null/offers-test dbname=offers_test connect_timeout=1",
        ))
}

#[cfg(test)]
fn app(pool: DbPool) -> Route {
    let (_, state) = start_mulac(pool).expect("start test kernel");
    app_with_reporting_token(state, DEFAULT_REPORTING_TOKEN.to_owned())
}

fn app_with_reporting_token(state: AppState, token: String) -> Route {
    let api = OpenApiService::new(
        (
            HealthApi,
            OffersApi::new(state.inbox.clone()),
            InboxApi::new(state.inbox),
            CommandingApi::new(state.commanding),
            EventingApi::new(state.events),
            OutboxApi::new(state.outbox),
            ReportingApi::new(state.pool, token),
        ),
        "Offers",
        env!("CARGO_PKG_VERSION"),
    )
    .server("/api");
    let schema = Schema::new(Query, EmptyMutation, EmptySubscription);

    let mut route = Route::new()
        .nest("/api/docs", api.swagger_ui())
        .at("/api/openapi.json", api.spec_endpoint())
        .nest("/api", api)
        .at("/graphql", GraphQL::new(schema));
    if cfg!(debug_assertions) {
        route = route.at("/graphql/ide", get(graphiql));
    }
    route
}

/// Parses the `PORT` environment variable as returned by `env::var`.
/// An unset variable selects the default port, 3000.
fn parse_port(value: Result<String, env::VarError>) -> io::Result<u16> {
    match value {
        Ok(value) => value.parse().map_err(|_| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                "PORT must be an integer between 0 and 65535",
            )
        }),
        Err(env::VarError::NotPresent) => Ok(3000),
        Err(error) => Err(io::Error::new(io::ErrorKind::InvalidInput, error)),
    }
}

/// Resolves on Ctrl+C or, on Unix, SIGTERM, so in-flight requests can finish
/// before the server exits.
fn shutdown_signal() -> io::Result<impl Future<Output = ()> + Send + 'static> {
    #[cfg(unix)]
    let mut terminate = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())?;
    Ok(async move {
        #[cfg(unix)]
        tokio::select! {
            Ok(()) = tokio::signal::ctrl_c() => {}
            _ = terminate.recv() => {}
        }
        #[cfg(not(unix))]
        if tokio::signal::ctrl_c().await.is_err() {
            std::future::pending::<()>().await;
        }
    })
}

#[cfg(test)]
mod tests {
    use std::{env::VarError, io::ErrorKind};

    use poem::{http::StatusCode, test::TestClient};
    use serde_json::{Value, json};

    use super::{app, parse_port, test_pool};

    #[tokio::test]
    async fn health_returns_ok() {
        let response = TestClient::new(app(test_pool()))
            .get("/api/health")
            .send()
            .await;
        response.assert_status_is_ok();
        response.assert_text("ok").await;
    }

    #[tokio::test]
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

    #[tokio::test]
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

    #[tokio::test]
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

    #[tokio::test]
    async fn openapi_spec_matches_routes() {
        let client = TestClient::new(app(test_pool()));
        let response = client.get("/api/openapi.json").send().await;
        response.assert_status_is_ok();
        let spec: Value = response.json().await.value().deserialize();

        assert!(
            spec["paths"]["/health"]["get"].is_object(),
            "OpenAPI must document GET /health",
        );
        let import = &spec["paths"]["/tmp-import"]["post"];
        assert_eq!(
            import["requestBody"]["content"]["application/json; charset=utf-8"]["schema"]["$ref"],
            "#/components/schemas/ImportOfferRequest"
        );
        assert_eq!(
            import["responses"]["202"]["content"]["application/json; charset=utf-8"]["schema"]["$ref"],
            "#/components/schemas/SubmitInboxResponse"
        );
        assert!(import["responses"]["200"].is_null());
        assert!(spec["components"]["schemas"]["ImportOfferResponse"].is_null());
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

    #[tokio::test]
    async fn swagger_ui_is_served() {
        let response = TestClient::new(app(test_pool()))
            .get("/api/docs/")
            .send()
            .await;
        response.assert_status_is_ok();
    }

    #[tokio::test]
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

    #[tokio::test]
    async fn tmp_import_reports_unavailable_storage() {
        let response = TestClient::new(app(test_pool()))
            .post("/api/tmp-import")
            .body_json(&json!({"retailer_code":"ankerau", "title":"Charger", "price_amount":1999, "currency":"AUD", "url":"https://example.com/charger"}))
            .send().await;
        response.assert_status(StatusCode::INTERNAL_SERVER_ERROR);
        response
            .assert_text("inbox message could not be stored")
            .await;
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
