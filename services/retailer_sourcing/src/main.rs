mod development;
mod offer_discovery;
mod platform;
mod sitemap_discovery;

use std::env::VarError;
use std::io;

use async_graphql::{EmptyMutation, EmptySubscription, Object, Schema, http::GraphiQLSource};
use async_graphql_axum::GraphQL;
use axum::{Router, response::Html, routing::get};
use tower_http::trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer};
use tracing::Level;
use utoipa::openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme};
use utoipa::{Modify, OpenApi};
use utoipa_axum::{router::OpenApiRouter, routes};
use utoipa_swagger_ui::SwaggerUi;

use platform::Platform;
use platform::bus::Bus;
use platform::state::AppState;

#[derive(OpenApi)]
#[openapi(
    info(title = "Retailer Sourcing"),
    servers((url = "/api")),
    modifiers(&BearerScheme)
)]
struct ApiDoc;

/// Documents the bearer token some routes need, so Swagger UI can send it.
struct BearerScheme;

impl Modify for BearerScheme {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        let scheme = HttpBuilder::new().scheme(HttpAuthScheme::Bearer).build();
        openapi
            .components
            .get_or_insert_with(Default::default)
            .add_security_scheme(
                platform::auth::SECURITY_SCHEME,
                SecurityScheme::Http(scheme),
            );
    }
}

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

/// The service's routes, sharing `state`.
fn app(state: AppState) -> Router {
    let (api, openapi) = OpenApiRouter::with_openapi(ApiDoc::openapi())
        .routes(routes!(health))
        .merge(platform::bus_api::routes())
        .merge(sitemap_discovery::prelude::routes())
        .merge(development::prelude::routes())
        .split_for_parts();
    let api = api.with_state(state);
    let schema = Schema::new(Query, EmptyMutation, EmptySubscription);

    let mut router = Router::new()
        .nest("/api", api)
        .merge(SwaggerUi::new("/api/docs").url("/api/openapi.json", openapi))
        .route_service("/graphql", GraphQL::new(schema));
    if cfg!(debug_assertions) {
        router = router.route("/graphql/ide", get(graphiql));
    }
    // One log line per request, with its status and latency.
    router.layer(
        TraceLayer::new_for_http()
            .make_span_with(DefaultMakeSpan::new().level(Level::INFO))
            .on_response(DefaultOnResponse::new().level(Level::INFO)),
    )
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

/// Connects to the database and starts the command and event bus when
/// `DATABASE_URL` is set, after applying pending migrations. Without it the
/// service only serves HTTP.
async fn start_features(database_url: Result<String, VarError>) -> io::Result<AppState> {
    let database_url = match database_url {
        Ok(database_url) if database_url.is_empty() => return Ok(AppState::default()),
        Ok(database_url) => database_url,
        Err(VarError::NotPresent) => return Ok(AppState::default()),
        Err(error) => return Err(io::Error::new(io::ErrorKind::InvalidInput, error)),
    };
    let object_storage =
        platform::object_storage::ObjectStorage::from_env(tokio::runtime::Handle::current())
            .map_err(io::Error::other)?;
    let platform =
        tokio::task::spawn_blocking(move || Platform::connect(&database_url, object_storage))
            .await
            .map_err(io::Error::other)?
            .map_err(io::Error::other)?;

    let mut bus = Bus::default();
    sitemap_discovery::prelude::register(&mut bus, &platform);
    offer_discovery::prelude::register(&mut bus, &platform);
    development::prelude::register(&mut bus, &platform);

    let db = platform.db.clone();
    let object_storage = platform.object_storage.clone();
    let bus = bus.start(platform.db).map_err(io::Error::other)?;
    Ok(AppState {
        bus: Some(bus),
        db: Some(db),
        object_storage: Some(object_storage),
    })
}

#[tokio::main]
async fn main() -> io::Result<()> {
    let _logs = platform::logging::init();
    let port = parse_port(std::env::var("PORT"))?;
    let features = start_features(std::env::var("DATABASE_URL")).await?;

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{port}")).await?;
    axum::serve(listener, app(features))
        .with_graceful_shutdown(shutdown_signal()?)
        .await
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

    use super::platform::auth::BEARER_TOKEN;
    use super::{AppState, app, parse_port, start_features};

    async fn get(uri: &str) -> Response {
        app(AppState::default())
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
        let response = app(AppState::default()).oneshot(request).await.unwrap();
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
        for path in ["/sitemap-retrievals", "/commands/run", "/events/run"] {
            assert!(
                spec["paths"][path]["post"].is_object(),
                "OpenAPI must document POST {path}",
            );
        }
        for (path, _) in DEVELOPMENT_SOURCES {
            assert!(
                spec["paths"][path]["get"].is_object(),
                "OpenAPI must document GET {path}",
            );
            assert_eq!(
                spec["paths"][path]["get"]["security"],
                json!([{ "bearer": [] }]),
                "{path}",
            );
        }
        assert_eq!(
            spec["components"]["securitySchemes"]["bearer"]["scheme"],
            "bearer"
        );
        assert_eq!(spec["servers"][0]["url"], "/api");

        let server = spec["servers"][0]["url"].as_str().unwrap_or("");
        for path in spec["paths"].as_object().unwrap().keys() {
            let response = get(&format!("{server}{path}")).await;
            assert_ne!(response.status(), StatusCode::NOT_FOUND, "{server}{path}");
        }
    }

    async fn post_json(uri: &str, body: Value) -> Response {
        let request = Request::post(uri)
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(body.to_string()))
            .unwrap();
        app(AppState::default()).oneshot(request).await.unwrap()
    }

    #[tokio::test]
    async fn sitemap_retrievals_reject_unknown_retailers() {
        let response = post_json("/api/sitemap-retrievals", json!({ "retailers": ["nope"] })).await;
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        let json: Value = serde_json::from_slice(&body(response).await).unwrap();
        assert_eq!(json, json!({ "error": "unknown retailer code: nope" }));
    }

    #[tokio::test]
    async fn sitemap_retrievals_need_the_workers() {
        let response = post_json(
            "/api/sitemap-retrievals",
            json!({ "retailers": ["ugreenca"] }),
        )
        .await;
        assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
    }

    #[tokio::test]
    async fn runs_reject_a_limit_out_of_range() {
        for uri in ["/api/commands/run", "/api/events/run"] {
            for limit in [0, 1001] {
                let response = post_json(uri, json!({ "limit": limit })).await;
                assert_eq!(response.status(), StatusCode::BAD_REQUEST, "{uri} {limit}");
            }
        }
    }

    #[tokio::test]
    async fn runs_need_the_bus() {
        for uri in ["/api/commands/run", "/api/events/run"] {
            let response = post_json(uri, json!({ "limit": 1 })).await;
            assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE, "{uri}");
        }
    }

    async fn get_with_token(uri: &str, token: &str) -> Response {
        let request = Request::get(uri)
            .header(header::AUTHORIZATION, format!("Bearer {token}"))
            .body(Body::empty())
            .unwrap();
        app(AppState::default()).oneshot(request).await.unwrap()
    }

    /// The development source routes relative to `/api`, with their maximum page sizes.
    const DEVELOPMENT_SOURCES: [(&str, u32); 2] = [
        ("/development/sitemaps/sources", 100),
        ("/development/offer-details/sources", 1000),
    ];

    #[tokio::test]
    async fn development_sources_need_the_bearer_token() {
        for (path, _) in DEVELOPMENT_SOURCES {
            let uri = format!("/api{path}");
            let response = get(&uri).await;
            assert_eq!(response.status(), StatusCode::UNAUTHORIZED, "{uri}");
            assert_eq!(response.headers()[header::WWW_AUTHENTICATE], "Bearer");

            let response = get_with_token(&uri, "wrong").await;
            assert_eq!(response.status(), StatusCode::UNAUTHORIZED, "{uri}");
        }
    }

    #[tokio::test]
    async fn development_sources_reject_a_limit_out_of_range() {
        for (path, max_limit) in DEVELOPMENT_SOURCES {
            for limit in [0, max_limit + 1] {
                let uri = format!("/api{path}?limit={limit}");
                let response = get_with_token(&uri, BEARER_TOKEN).await;
                assert_eq!(response.status(), StatusCode::BAD_REQUEST, "{uri}");
            }
        }
    }

    #[tokio::test]
    async fn development_sources_need_the_database() {
        for (path, max_limit) in DEVELOPMENT_SOURCES {
            for query in [
                String::new(),
                "?limit=1".to_owned(),
                format!("?limit={max_limit}&offset=40"),
            ] {
                let uri = format!("/api{path}{query}");
                let response = get_with_token(&uri, BEARER_TOKEN).await;
                assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE, "{uri}");
            }
        }
    }

    #[tokio::test]
    async fn swagger_ui_is_served() {
        let response = get("/api/docs/").await;
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn features_stay_off_without_a_database() {
        let state = start_features(Err(VarError::NotPresent)).await.unwrap();
        assert!(state.bus.is_none());
        assert!(state.db.is_none());
    }

    #[tokio::test]
    async fn features_stay_off_with_an_empty_database_url() {
        let state = start_features(Ok(String::new())).await.unwrap();
        assert!(state.bus.is_none());
        assert!(state.db.is_none());
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
