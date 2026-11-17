mod development;
mod monitoring;
mod offer_delivery;
mod offer_discovery;
mod platform;
mod reporting;
mod saga;
mod sitemap_discovery;
mod steps;
mod v2;

use axum::serve;
use development::prelude::register as development_register;
use development::prelude::routes as development_routes;
use io::Error;
use io::ErrorKind as IoErrorKind;
use io::Result as IoResult;
use monitoring::prelude::register as monitoring_register;
use offer_delivery::prelude::register as offer_delivery_register;
use offer_discovery::prelude::register as offer_discovery_register;
use outbox::prelude::AsyncOutboxService;
use outbox::prelude::axum::OutboxAdapter;
use platform::auth::SECURITY_SCHEME;
use platform::bus_api::routes as platform_bus_api_routes;
use platform::http::OfferClient;
use platform::logging::init;
use platform::object_storage::ObjectStorage;
use platform::offers_publisher::OffersPublisher;
use reporting::prelude::register as reporting_register;
use reporting::prelude::routes as reporting_routes;
use shared::health as shared_health;
use sitemap_discovery::prelude::register;
use sitemap_discovery::prelude::routes as sitemap_discovery_routes;
use std::env::var;
#[cfg(not(unix))]
use std::future::pending as std_future_pending;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::runtime::Handle;
use tokio::select;
#[cfg(not(unix))]
use tokio::signal::ctrl_c;
use tokio::signal::unix::SignalKind;
use tokio::signal::unix::signal as signal_unix_signal;
use tokio::task::spawn_blocking;
use tracing::info as tracing_info;
use utoipa::openapi::OpenApi as UtoipaOpenapiOpenApi;
use v2::prelude::register as v2_register;
use v2::prelude::routes as v2_routes;

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
    fn modify(&self, openapi: &mut UtoipaOpenapiOpenApi) {
        let scheme = HttpBuilder::new().scheme(HttpAuthScheme::Bearer).build();
        openapi
            .components
            .get_or_insert_with(Default::default)
            .add_security_scheme(SECURITY_SCHEME, SecurityScheme::Http(scheme));
    }
}

#[utoipa::path(get, path = "/health", responses((status = 200, body = String)))]
async fn health() -> &'static str {
    shared_health()
}

struct Query;

#[Object]
impl Query {
    async fn health(&self) -> &'static str {
        shared_health()
    }
}

async fn graphiql() -> Html<String> {
    Html(GraphiQLSource::build().endpoint("/graphql").finish())
}

/// The service's routes, sharing `state`.
fn app(state: AppState) -> Router {
    let (api, openapi) = OpenApiRouter::with_openapi(ApiDoc::openapi())
        .routes(routes!(health))
        .merge(platform_bus_api_routes())
        .merge(sitemap_discovery_routes())
        .merge(v2_routes())
        .merge(development_routes())
        .merge(reporting_routes())
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
fn parse_port(value: Result<String, VarError>) -> IoResult<u16> {
    match value {
        Ok(value) => value.parse().map_err(|_| {
            Error::new(
                IoErrorKind::InvalidInput,
                "PORT must be an integer between 0 and 65535",
            )
        }),
        Err(VarError::NotPresent) => Ok(3000),
        Err(error) => Err(Error::new(IoErrorKind::InvalidInput, error)),
    }
}

/// Parses the number of concurrent command loops; zero keeps command runs manual.
fn parse_command_workers(value: Result<String, VarError>) -> IoResult<usize> {
    let invalid = || {
        Error::new(
            IoErrorKind::InvalidInput,
            "COMMAND_WORKERS must be a non-negative integer that fits usize",
        )
    };
    match value {
        Err(VarError::NotPresent) => Ok(4),
        Ok(value) if !value.is_empty() && value.bytes().all(|byte| byte.is_ascii_digit()) => {
            value.parse().map_err(|_| invalid())
        }
        _ => Err(invalid()),
    }
}

/// Connects to the database and starts the command and event bus when
/// `DATABASE_URL` is set, after applying pending migrations. Without it the
/// service only serves HTTP.
async fn start_features(database_url: Result<String, VarError>) -> IoResult<AppState> {
    let publisher = OffersPublisher::from_env(Handle::current())?;
    let database_url = match database_url {
        Ok(database_url) if database_url.is_empty() => return Ok(AppState::default()),
        Ok(database_url) => database_url,
        Err(VarError::NotPresent) => return Ok(AppState::default()),
        Err(error) => return Err(Error::new(IoErrorKind::InvalidInput, error)),
    };
    let object_storage = ObjectStorage::from_env(Handle::current()).map_err(Error::other)?;
    let offer_http = OfferClient::configured()?;
    let platform =
        spawn_blocking(move || Platform::connect(&database_url, object_storage, offer_http))
            .await
            .map_err(Error::other)?
            .map_err(Error::other)?;

    let mut bus = Bus::default();
    register(&mut bus, &platform);
    v2_register(&mut bus, &platform);
    offer_discovery_register(&mut bus, &platform);
    offer_delivery_register(&mut bus, &platform);
    development_register(&mut bus, &platform);
    monitoring_register(&mut bus, &platform);
    reporting_register(&mut bus, &platform);

    let db = platform.db.clone();
    let outbox = publisher.map(|publisher| {
        let service = AsyncOutboxService::from_postgres(db.clone(), Arc::new(publisher));
        OutboxAdapter::new(service)
    });
    let object_storage = platform.object_storage.clone();
    let bus = bus.start(platform.db).map_err(Error::other)?;
    Ok(AppState {
        bus: Some(bus),
        db: Some(db),
        object_storage: Some(object_storage),
        outbox,
    })
}

#[tokio::main]
async fn main() -> IoResult<()> {
    let _logs = init();
    let port = parse_port(var("PORT"))?;
    let command_workers = parse_command_workers(var("COMMAND_WORKERS"))?;
    let features = start_features(var("DATABASE_URL")).await?;
    let bus = features.bus.clone();

    // Complete all fallible startup before spawning workers.
    let routes = app(features);
    let listener = TcpListener::bind(format!("0.0.0.0:{port}")).await?;
    let signal = shutdown_signal()?;
    let workers = match &bus {
        Some(bus) => bus.start_command_workers(command_workers),
        None => {
            tracing_info!(
                configured_command_workers = command_workers,
                "command workers disabled: DATABASE_URL is not configured"
            );
            Default::default()
        }
    };
    let shutdown_bus = bus.clone();
    let result = serve(listener, routes)
        .with_graceful_shutdown(async move {
            signal.await;
            if let Some(bus) = shutdown_bus {
                bus.shutdown();
            }
        })
        .await;
    // A server error also cancels workers. Always join every loop after HTTP drains.
    if let Some(bus) = bus {
        bus.shutdown();
    }
    let joined = workers.join().await.map_err(Error::other);
    result.and(joined)
}

/// Resolves on Ctrl+C or, on Unix, SIGTERM, so in-flight requests can finish
/// before the server exits.
fn shutdown_signal() -> IoResult<impl Future<Output = ()> + Send + 'static> {
    #[cfg(unix)]
    let mut interrupt = signal_unix_signal(SignalKind::interrupt())?;
    #[cfg(unix)]
    let mut terminate = signal_unix_signal(SignalKind::terminate())?;
    Ok(async move {
        #[cfg(unix)]
        select! {
            _ = interrupt.recv() => {}
            _ = terminate.recv() => {}
        }
        #[cfg(not(unix))]
        if ctrl_c().await.is_err() {
            std_future_pending::<()>().await;
        }
    })
}

#[cfg(test)]
mod tests {
    use header::AUTHORIZATION;
    use header::CONTENT_TYPE;
    use header::WWW_AUTHENTICATE;
    use serde_json::from_slice;
    use std::collections::HashSet;
    use std::ffi::OsString as StdFfiOsString;
    use uuid::Uuid;

    pub(crate) fn assert_error_id(error: &str) {
        let id = Uuid::parse_str(error).expect("error response must contain only a UUID");
        assert_eq!(id.get_version_num(), 7);
    }

    pub(crate) fn assert_batch_errors(body: &Value, count: usize) {
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
    use super::{AppState, app, parse_command_workers, parse_port, start_features};

    #[test]
    fn command_worker_configuration_accepts_default_zero_and_counts() {
        assert_eq!(parse_command_workers(Err(VarError::NotPresent)).unwrap(), 4);
        for count in [0, 1, 8, usize::MAX] {
            assert_eq!(parse_command_workers(Ok(count.to_string())).unwrap(), count);
        }
    }

    #[test]
    fn command_worker_configuration_rejects_invalid_values_with_named_error() {
        for value in [
            "",
            "-1",
            "+1",
            "1.5",
            "four",
            " 4",
            "4 ",
            "99999999999999999999999999999999999999",
        ] {
            let error = parse_command_workers(Ok(value.into())).unwrap_err();
            assert_eq!(error.kind(), ErrorKind::InvalidInput);
            assert!(error.to_string().contains("COMMAND_WORKERS"));
        }
        let error =
            parse_command_workers(Err(VarError::NotUnicode(StdFfiOsString::new()))).unwrap_err();
        assert!(error.to_string().contains("COMMAND_WORKERS"));
    }

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
            .header(CONTENT_TYPE, "application/json")
            .body(Body::from(json!({ "query": "{ health }" }).to_string()))
            .unwrap();
        let response = app(AppState::default()).oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let json: Value = from_slice(&body(response).await).unwrap();
        assert_eq!(json, json!({ "data": { "health": "ok" } }));
    }

    #[tokio::test]
    async fn graphql_get_query_returns_ok() {
        let response = get("/graphql?query=%7B%20health%20%7D").await;
        assert_eq!(response.status(), StatusCode::OK);
        let json: Value = from_slice(&body(response).await).unwrap();
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
        let spec: Value = from_slice(&body(response).await).unwrap();

        assert!(
            spec["paths"]["/health"]["get"].is_object(),
            "OpenAPI must document GET /health",
        );
        for path in [
            "/sitemap-retrievals",
            "/commands/run",
            "/events/run",
            "/outbox/run",
        ] {
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
        assert_eq!(
            spec["paths"]["/reporting/database-tables"]["get"]["security"],
            json!([{ "bearer": [] }])
        );
        assert!(spec["paths"]["/reporting/database-tables"]["get"]["responses"]["200"].is_object());
        assert!(spec["paths"]["/reporting/entry-statuses"]["get"]["responses"]["200"].is_object());
        assert_eq!(
            spec["paths"]["/reporting/entry-statuses"]["get"]["security"],
            json!([{ "bearer": [] }])
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
            .header(CONTENT_TYPE, "application/json")
            .body(Body::from(body.to_string()))
            .unwrap();
        app(AppState::default()).oneshot(request).await.unwrap()
    }

    #[tokio::test]
    async fn sitemap_retrievals_reject_unknown_retailers() {
        let response = post_json("/api/sitemap-retrievals", json!({ "retailers": ["nope"] })).await;
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        let json: Value = from_slice(&body(response).await).unwrap();
        assert_eq!(json, json!({ "error": "unknown retailer code: nope" }));
    }

    #[tokio::test]
    async fn sitemap_retrievals_reject_disconnected_retailers() {
        for (code, expected) in [
            ("micom", StatusCode::BAD_REQUEST),
            ("ugreenca", StatusCode::SERVICE_UNAVAILABLE),
        ] {
            let response = post_json("/api/sitemap-retrievals", json!({"retailers": [code]})).await;
            assert_eq!(response.status(), expected);
        }
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
            .header(AUTHORIZATION, format!("Bearer {token}"))
            .body(Body::empty())
            .unwrap();
        app(AppState::default()).oneshot(request).await.unwrap()
    }

    /// The development source routes relative to `/api`, with their maximum page sizes.
    const DEVELOPMENT_SOURCES: [(&str, u32); 3] = [
        ("/development/sitemaps/sources", 100),
        ("/development/offer-details/sources", 1000),
        ("/development/offer-details-v2/sources", 1000),
    ];

    #[tokio::test]
    async fn development_sources_need_the_bearer_token() {
        for (path, _) in DEVELOPMENT_SOURCES {
            let uri = format!("/api{path}");
            let response = get(&uri).await;
            assert_eq!(response.status(), StatusCode::UNAUTHORIZED, "{uri}");
            assert_eq!(response.headers()[WWW_AUTHENTICATE], "Bearer");

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
