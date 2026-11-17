use async_graphql::Error as AsyncGraphqlError;
use async_graphql::Result as AsyncGraphqlResult;
use async_graphql::http::GraphiQLSource;
use bootstrap::database_pool;
use offer_read::OfferPage;
use offer_read::Offers;
use offer_read::OffersApi;
use orchestration::start_mulac;
use reporting::DEFAULT_REPORTING_TOKEN;
use reporting::ReportingApi;
use shared::health as shared_health;
use std::env::VarError as EnvVarError;
use std::env::var as env_var;
#[cfg(not(unix))]
use std::future::pending;
use std::io::Error as IoError;
use std::io::ErrorKind as IoErrorKind;
use std::io::Result as IoResult;
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
mod bootstrap;
mod identifier;
mod offer_projection;
mod offer_read;
mod orchestration;
#[cfg(test)]
mod postgres_tests;
#[cfg(test)]
mod queue_tests;
mod reporting;
mod stored_content;

use async_graphql::{EmptyMutation, EmptySubscription, Object, Schema};
use async_graphql_poem::GraphQL;
use orchestration::{AppState, QueueApi};
use poem::{Route, Server, get, handler, listener::TcpListener, web::Html};
use poem_openapi::{OpenApi, OpenApiService, payload::PlainText};

struct Api;

#[OpenApi]
impl Api {
    #[oai(path = "/health", method = "get")]
    async fn health(&self) -> PlainText<&'static str> {
        PlainText(shared_health())
    }
}

struct Query(Offers);

#[Object]
impl Query {
    async fn health(&self) -> &'static str {
        shared_health()
    }

    async fn search_offers(
        &self,
        q: String,
        #[graphql(default = 30)] limit: i32,
        #[graphql(default = 0)] offset: i32,
    ) -> AsyncGraphqlResult<OfferPage> {
        self.0
            .search(&q, limit, offset)
            .await
            .map_err(|error| AsyncGraphqlError::new(error.message()))
    }

    async fn offers(
        &self,
        #[graphql(default = 30)] limit: i32,
        #[graphql(default = 0)] offset: i32,
    ) -> AsyncGraphqlResult<OfferPage> {
        self.0
            .list(limit, offset)
            .await
            .map_err(|error| AsyncGraphqlError::new(error.message()))
    }
}

#[handler]
async fn graphiql() -> Html<String> {
    Html(GraphiQLSource::build().endpoint("/graphql").finish())
}

#[cfg(test)]
fn app(state: AppState) -> Route {
    app_with_reporting_token(state, DEFAULT_REPORTING_TOKEN.into())
}

fn app_with_reporting_token(state: AppState, token: String) -> Route {
    let reports = ReportingApi::new(state.reporting.clone(), token);
    let offers = state.offers.clone();
    let api = OpenApiService::new(
        (Api, QueueApi(state), OffersApi(offers.clone()), reports),
        "Customer Facing",
        env!("CARGO_PKG_VERSION"),
    )
    .server("/api");
    let schema = Schema::new(Query(offers), EmptyMutation, EmptySubscription);

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

/// Parses the `PORT` environment variable as returned by `std::env::var`.
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

#[tokio_main]
async fn main() -> IoResult<()> {
    fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| "warn".into()))
        .with_writer(stderr)
        .init();
    let port = parse_port(env_var("PORT"))?;
    let token = match env_var("CUSTOMER_FACING_REPORTING_TOKEN") {
        Ok(token) => token,
        Err(EnvVarError::NotPresent) => DEFAULT_REPORTING_TOKEN.into(),
        Err(_) => {
            return Err(IoError::new(
                IoErrorKind::InvalidInput,
                "CUSTOMER_FACING_REPORTING_TOKEN must be valid Unicode",
            ));
        }
    };
    let pool = database_pool()?;
    let shutdown = shutdown_signal()?;
    let (kernel, state) =
        start_mulac(pool).map_err(|_| IoError::other("Mulac kernel could not start"))?;
    let result = Server::new(TcpListener::bind(format!("0.0.0.0:{port}")))
        .run_with_graceful_shutdown(app_with_reporting_token(state, token), shutdown, None)
        .await;
    kernel.shutdown();
    result
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
    use super::app as super_app;
    use crate::queue_tests::state as queue_tests_state;
    use poem::Route as PoemRoute;
    use poem::test::TestResponse;
    use serde_json::Value as SerdeJsonValue;
    use std::collections::HashSet;
    use tokio::test as tokio_test;
    use uuid::Uuid;

    pub(crate) fn assert_error_id(error: &str) {
        let id = Uuid::parse_str(error).expect("error response must contain only a UUID");
        assert_eq!(id.get_version_num(), 7);
    }

    pub(crate) fn assert_batch_errors(body: &SerdeJsonValue, count: usize) {
        assert!(body["processed"].is_null(), "{body}");
        let errors = body["errors"].as_array().expect("batch errors array");
        assert_eq!(errors.len(), count, "{body}");
        let mut ids = HashSet::new();
        for error in errors {
            let error = error.as_str().expect("error ID string");
            assert_error_id(error);
            assert!(ids.insert(error), "each failure must have its own ID");
        }
    }

    pub(crate) async fn assert_error_response(response: TestResponse) {
        let body = response.0.into_body().into_string().await.unwrap();
        assert_error_id(&body);
    }

    pub(crate) async fn assert_batch_response(response: TestResponse, count: usize) {
        let body = response.json().await.value().deserialize();
        assert_batch_errors(&body, count);
    }

    use std::{env::VarError, io::ErrorKind};

    use poem::{http::StatusCode, test::TestClient};
    use serde_json::{Value, json};

    use super::parse_port;

    fn app() -> PoemRoute {
        super_app(queue_tests_state())
    }

    #[tokio_test]
    async fn health_returns_ok() {
        let response = TestClient::new(app()).get("/api/health").send().await;
        response.assert_status_is_ok();
        response.assert_text("ok").await;
    }

    #[tokio_test]
    async fn graphql_health_returns_ok() {
        let response = TestClient::new(app())
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
        let response = TestClient::new(app())
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
        let response = TestClient::new(app()).get("/graphql/ide").send().await;
        if cfg!(debug_assertions) {
            response.assert_status_is_ok();
        } else {
            response.assert_status(StatusCode::NOT_FOUND);
        }
    }

    #[tokio_test]
    async fn openapi_spec_matches_routes() {
        let client = TestClient::new(app());
        let response = client.get("/api/openapi.json").send().await;
        response.assert_status_is_ok();
        let spec: Value = response.json().await.value().deserialize();

        assert!(
            spec["paths"]["/health"]["get"].is_object(),
            "OpenAPI must document GET /health",
        );
        assert_eq!(spec["servers"][0]["url"], "/api");

        let server = spec["servers"][0]["url"].as_str().unwrap_or("");
        for (path, methods) in spec["paths"].as_object().unwrap() {
            for method in methods.as_object().unwrap().keys() {
                let uri = format!("{server}{path}");
                let response = match method.as_str() {
                    "get" => client.get(&uri).send().await,
                    "post" => client.post(&uri).body_json(&json!({})).send().await,
                    _ => panic!("unexpected documented method: {method}"),
                };
                assert_ne!(response.0.status(), StatusCode::NOT_FOUND, "{method} {uri}");
                assert_ne!(
                    response.0.status(),
                    StatusCode::METHOD_NOT_ALLOWED,
                    "{method} {uri}"
                );
            }
        }
    }

    #[tokio_test]
    async fn swagger_ui_is_served() {
        let response = TestClient::new(app()).get("/api/docs/").send().await;
        response.assert_status_is_ok();
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
mod region_market_tests;
