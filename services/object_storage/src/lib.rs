pub mod error;
pub mod graphql;
pub mod model;
mod openapi;
mod reporting;
pub mod store;

use async_graphql::http::GraphiQLSource;
use axum::extract::rejection::JsonRejection;
use axum::extract::rejection::PathRejection;
use axum::http::Method;
use graphql::handle;
use graphql::schema as graphql_schema;
use header::AUTHORIZATION;
use header::CONTENT_TYPE;
use header::LOCATION;
use middleware::from_fn;
use model::Record;
use openapi::document;
use reporting::routes as reporting_routes;
use shared::health as shared_health;
use tracing::Level;

use axum::{
    Json, Router,
    extract::{DefaultBodyLimit, Path, Request, State},
    http::{StatusCode, header},
    middleware::{self, Next},
    response::{Html, IntoResponse, Response},
    routing::{get, post},
};
use error::Error;
use model::Create;
use store::Store;
use tower_http::trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer};
use utoipa_swagger_ui::SwaggerUi;

/// Development credential only; every repository reader can use this token.
pub const BEARER_TOKEN: &str = "6fe0382d64bba74e205af86e397d902c0f5e6f3d8e15b1a4f87c6a309d752eb1";

async fn authenticate(request: Request, next: Next) -> Response {
    let mut values = request.headers().get_all(AUTHORIZATION).iter();
    let credential = values
        .next()
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.split_once(' '))
        .filter(|(scheme, _)| scheme.eq_ignore_ascii_case("bearer"))
        .map(|(_, token)| token.as_bytes());
    let valid = credential.is_some_and(|token| {
        token.len() == BEARER_TOKEN.len()
            && token
                .iter()
                .zip(BEARER_TOKEN.bytes())
                .fold(0, |diff, (a, b)| diff | (a ^ b))
                == 0
    });
    if valid && values.next().is_none() {
        next.run(request).await
    } else {
        Error::Unauthorized.into_response()
    }
}

async fn content_type(request: Request, next: Next) -> Response {
    if request.method() != Method::POST {
        return next.run(request).await;
    }
    let supported = request
        .headers()
        .get(CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.split(';').next())
        .is_some_and(|v| v.trim().eq_ignore_ascii_case("application/json"));
    if supported {
        next.run(request).await
    } else {
        Error::ContentType.into_response()
    }
}

async fn create(
    State(store): State<Store>,
    path: Result<Path<String>, PathRejection>,
    input: Result<Json<Create>, JsonRejection>,
) -> Result<Response, Error> {
    let Path(bucket) = path.map_err(|_| Error::Input("Invalid bucket path."))?;
    let Json(input) = input.map_err(|_| {
        Error::Input(
            "Expected exactly idempotency_key, payload, and non-null metadata in a JSON object.",
        )
    })?;
    let created = store.create(bucket.clone(), input).await?;
    let status = if created.fresh {
        StatusCode::CREATED
    } else {
        StatusCode::OK
    };
    let location = format!("/api/buckets/{bucket}/records/{}", created.id);
    Ok((status, [(LOCATION, location)], Json(created)).into_response())
}
async fn retrieve(
    State(store): State<Store>,
    path: Result<Path<(String, String)>, PathRejection>,
) -> Result<Json<Record>, Error> {
    let Path((bucket, id)) = path.map_err(|_| Error::Input("Invalid record path."))?;
    store.get(bucket, id).await.map(Json)
}

/// All routes (including fallbacks, documentation assets and health) require
/// authentication. The outer auth layer runs before extractors read the body.
pub fn app(store: Store) -> Router {
    let schema = graphql_schema(store.clone());
    let rest = Router::new()
        .route(
            "/api/buckets/{bucket}/records",
            post(create).layer(from_fn(content_type)),
        )
        .route("/api/buckets/{bucket}/records/{id}", get(retrieve))
        .merge(reporting_routes())
        .with_state(store);
    let graphql = Router::new()
        .route("/graphql", post(handle).layer(from_fn(content_type)))
        .with_state(schema);
    let mut router = Router::new()
        .merge(rest)
        .merge(graphql)
        .route("/api/health", get(|| async { shared_health() }))
        .merge(SwaggerUi::new("/api/docs").external_url_unchecked("/api/openapi.json", document()))
        .fallback(|| async { Error::NotFound });
    if cfg!(debug_assertions) {
        router = router.route(
            "/graphql/ide",
            get(|| async { Html(GraphiQLSource::build().endpoint("/graphql").finish()) }),
        );
    }
    router
        .method_not_allowed_fallback(|| async { Error::MethodNotAllowed })
        .layer(DefaultBodyLimit::disable())
        .layer(from_fn(authenticate))
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::new().level(Level::INFO))
                .on_response(DefaultOnResponse::new().level(Level::INFO)),
        )
}
