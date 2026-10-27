mod bootstrap;
mod orchestration;
#[cfg(test)]
mod postgres_tests;
#[cfg(test)]
mod queue_tests;

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
        PlainText(shared::health())
    }
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
    Html(
        async_graphql::http::GraphiQLSource::build()
            .endpoint("/graphql")
            .finish(),
    )
}

fn app(state: AppState) -> Route {
    let api = OpenApiService::new(
        (Api, QueueApi(state)),
        "Customer Facing",
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

/// Parses the `PORT` environment variable as returned by `std::env::var`.
/// An unset variable selects the default port, 3000.
fn parse_port(value: Result<String, std::env::VarError>) -> std::io::Result<u16> {
    match value {
        Ok(value) => value.parse().map_err(|_| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "PORT must be an integer between 0 and 65535",
            )
        }),
        Err(std::env::VarError::NotPresent) => Ok(3000),
        Err(error) => Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, error)),
    }
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let port = parse_port(std::env::var("PORT"))?;
    let pool = bootstrap::database_pool()?;
    let shutdown = shutdown_signal()?;
    let (kernel, state) = orchestration::start_mulac(pool)
        .map_err(|_| std::io::Error::other("Mulac kernel could not start"))?;
    let result = Server::new(TcpListener::bind(format!("0.0.0.0:{port}")))
        .run_with_graceful_shutdown(app(state), shutdown, None)
        .await;
    kernel.shutdown();
    result
}

/// Resolves on Ctrl+C or, on Unix, SIGTERM, so in-flight requests can finish
/// before the server exits.
fn shutdown_signal() -> std::io::Result<impl Future<Output = ()> + Send + 'static> {
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

    use super::parse_port;

    fn app() -> poem::Route {
        super::app(crate::queue_tests::state())
    }

    #[tokio::test]
    async fn health_returns_ok() {
        let response = TestClient::new(app()).get("/api/health").send().await;
        response.assert_status_is_ok();
        response.assert_text("ok").await;
    }

    #[tokio::test]
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

    #[tokio::test]
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

    #[tokio::test]
    async fn graphiql_is_served_only_in_debug_builds() {
        let response = TestClient::new(app()).get("/graphql/ide").send().await;
        if cfg!(debug_assertions) {
            response.assert_status_is_ok();
        } else {
            response.assert_status(StatusCode::NOT_FOUND);
        }
    }

    #[tokio::test]
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

    #[tokio::test]
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
