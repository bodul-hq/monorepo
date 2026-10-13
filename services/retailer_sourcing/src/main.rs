mod platform;
mod sitemap_discovery;

use std::env::VarError;
use std::io;

use async_graphql::{EmptyMutation, EmptySubscription, Object, Schema, http::GraphiQLSource};
use async_graphql_axum::GraphQL;
use axum::{Router, response::Html, routing::get};
use utoipa::OpenApi;
use utoipa_axum::{router::OpenApiRouter, routes};
use utoipa_swagger_ui::SwaggerUi;

use platform::Platform;
use platform::bus::{Bus, Running};

#[derive(OpenApi)]
#[openapi(info(title = "Retailer Sourcing"), servers((url = "/api")))]
struct ApiDoc;

#[utoipa::path(get, path = "/health", responses((status = 200, body = String)))]
async fn health() -> &'static str {
    shared::health()
}

struct Query;

#[Object]
impl Query {
    async fn health(&self) -> &'static str {
        shared::health()
    }
}

async fn graphiql() -> Html<String> {
    Html(GraphiQLSource::build().endpoint("/graphql").finish())
}

fn app() -> Router {
    let (api, openapi) = OpenApiRouter::with_openapi(ApiDoc::openapi())
        .routes(routes!(health))
        .split_for_parts();
    let schema = Schema::new(Query, EmptyMutation, EmptySubscription);

    let mut router = Router::new()
        .nest("/api", api)
        .merge(SwaggerUi::new("/api/docs").url("/api/openapi.json", openapi))
        .route_service("/graphql", GraphQL::new(schema));
    if cfg!(debug_assertions) {
        router = router.route("/graphql/ide", get(graphiql));
    }
    router
}

/// Parses the `PORT` environment variable as returned by `std::env::var`.
/// An unset variable selects the default port, 3000.
fn parse_port(value: Result<String, VarError>) -> io::Result<u16> {
    match value {
        Ok(value) => value.parse().map_err(|_| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                "PORT must be an integer between 0 and 65535",
            )
        }),
        Err(VarError::NotPresent) => Ok(3000),
        Err(error) => Err(io::Error::new(io::ErrorKind::InvalidInput, error)),
    }
}

/// Starts the command and event workers when `DATABASE_URL` is set, after
/// applying pending migrations. Without it the service only serves HTTP.
async fn start_features(database_url: Result<String, VarError>) -> io::Result<Option<Running>> {
    let database_url = match database_url {
        Ok(database_url) => database_url,
        Err(VarError::NotPresent) => return Ok(None),
        Err(error) => return Err(io::Error::new(io::ErrorKind::InvalidInput, error)),
    };
    let platform = tokio::task::spawn_blocking(move || Platform::connect(&database_url))
        .await
        .map_err(io::Error::other)?
        .map_err(io::Error::other)?;

    let mut bus = Bus::default();
    sitemap_discovery::prelude::register(&mut bus, &platform);

    bus.start(platform.db).map(Some).map_err(io::Error::other)
}

#[tokio::main]
async fn main() -> io::Result<()> {
    let port = parse_port(std::env::var("PORT"))?;
    let features = start_features(std::env::var("DATABASE_URL")).await?;

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{port}")).await?;
    axum::serve(listener, app())
        .with_graceful_shutdown(shutdown_signal()?)
        .await?;

    if let Some(features) = features {
        features.stop().await;
    }
    Ok(())
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

    use axum::{
        body::Body,
        http::{Request, StatusCode, header},
        response::Response,
    };
    use http_body_util::BodyExt;
    use serde_json::{Value, json};
    use tower::ServiceExt;

    use super::{app, parse_port, start_features};

    async fn get(uri: &str) -> Response {
        app()
            .oneshot(Request::get(uri).body(Body::empty()).unwrap())
            .await
            .unwrap()
    }

    async fn body(response: Response) -> Vec<u8> {
        response
            .into_body()
            .collect()
            .await
            .unwrap()
            .to_bytes()
            .to_vec()
    }

    #[tokio::test]
    async fn health_returns_ok() {
        let response = get("/api/health").await;
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(body(response).await, b"ok");
    }

    #[tokio::test]
    async fn graphql_health_returns_ok() {
        let request = Request::post("/graphql")
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(json!({ "query": "{ health }" }).to_string()))
            .unwrap();
        let response = app().oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let json: Value = serde_json::from_slice(&body(response).await).unwrap();
        assert_eq!(json, json!({ "data": { "health": "ok" } }));
    }

    #[tokio::test]
    async fn graphql_get_query_returns_ok() {
        let response = get("/graphql?query=%7B%20health%20%7D").await;
        assert_eq!(response.status(), StatusCode::OK);
        let json: Value = serde_json::from_slice(&body(response).await).unwrap();
        assert_eq!(json, json!({ "data": { "health": "ok" } }));
    }

    #[tokio::test]
    async fn graphiql_is_served_only_in_debug_builds() {
        let response = get("/graphql/ide").await;
        let expected = if cfg!(debug_assertions) {
            StatusCode::OK
        } else {
            StatusCode::NOT_FOUND
        };
        assert_eq!(response.status(), expected);
    }

    #[tokio::test]
    async fn openapi_spec_matches_routes() {
        let response = get("/api/openapi.json").await;
        assert_eq!(response.status(), StatusCode::OK);
        let spec: Value = serde_json::from_slice(&body(response).await).unwrap();

        assert!(
            spec["paths"]["/health"]["get"].is_object(),
            "OpenAPI must document GET /health",
        );
        assert_eq!(spec["servers"][0]["url"], "/api");

        let server = spec["servers"][0]["url"].as_str().unwrap_or("");
        for path in spec["paths"].as_object().unwrap().keys() {
            let response = get(&format!("{server}{path}")).await;
            assert_ne!(response.status(), StatusCode::NOT_FOUND, "{server}{path}");
        }
    }

    #[tokio::test]
    async fn swagger_ui_is_served() {
        let response = get("/api/docs/").await;
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn features_stay_off_without_a_database() {
        assert!(
            start_features(Err(VarError::NotPresent))
                .await
                .unwrap()
                .is_none()
        );
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
