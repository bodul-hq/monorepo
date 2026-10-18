//! HTTP entry point: lists downloaded offer metadata, a page at a time.

use axum::Json;
use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use diesel::prelude::*;
use diesel::sql_types::{BigInt, Jsonb, Text, Uuid as SqlUuid};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;

use crate::platform::db::{DbPool, Json as DbJson};
use crate::platform::object_storage::OfferStorageMetadata;
use crate::platform::state::AppState;

/// The most source references one page may hold.
const MAX_LIMIT: u32 = 1000;

/// Each downloaded offer page with its offer, URL, retailer and storage, newest
/// first, skipping `$2` and returning up to `$1`.
const SOURCES_QUERY: &str = "
    SELECT r.id, r.offer_id, r.url, r.retailer_code, r.storage
    FROM offer_details_raw r
    ORDER BY r.fetched_at DESC, r.id
    LIMIT $1 OFFSET $2";

#[derive(Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct Page {
    /// How many sources to return, from 1 to 1000.
    #[serde(default = "default_limit")]
    #[param(default = 20, minimum = 1, maximum = 1000)]
    limit: u32,
    /// How many sources to skip.
    #[serde(default)]
    #[param(default = 0)]
    offset: u32,
}

fn default_limit() -> u32 {
    20
}

#[derive(Serialize, ToSchema)]
pub struct OfferSourcesResponse {
    sources: Vec<OfferSource>,
}

#[derive(Serialize, ToSchema)]
pub struct OfferSource {
    #[schema(value_type = String, format = "uuid")]
    id: Uuid,
    /// The offer the page belongs to.
    #[schema(value_type = String, format = "uuid")]
    offer_id: Uuid,
    /// The URL the page was downloaded from.
    url: String,
    /// The retailer the offer belongs to.
    retailer_code: String,
    /// Reference to the HTML in object storage's `offer_details_raw` bucket.
    storage: OfferStorageMetadata,
}

#[derive(QueryableByName)]
struct SourceRecord {
    #[diesel(sql_type = SqlUuid)]
    id: Uuid,
    #[diesel(sql_type = SqlUuid)]
    offer_id: Uuid,
    #[diesel(sql_type = Text)]
    url: String,
    #[diesel(sql_type = Text)]
    retailer_code: String,
    #[diesel(sql_type = Jsonb)]
    storage: DbJson<OfferStorageMetadata>,
}

#[derive(Serialize, ToSchema)]
pub struct ErrorBody {
    error: String,
}

/// Lists the downloaded offer pages, newest first.
#[utoipa::path(
    get,
    path = "/development/offer-details/sources",
    params(Page),
    security(("bearer" = [])),
    responses(
        (status = 200, description = "A page of sources", body = OfferSourcesResponse),
        (status = 400, description = "Limit out of range", body = ErrorBody),
        (status = 401, description = "Missing or wrong bearer token", body = ErrorBody),
        (status = 500, description = "The sources couldn't be read", body = ErrorBody),
        (status = 503, description = "No database (no DATABASE_URL)", body = ErrorBody),
    )
)]
pub async fn list_sources(State(state): State<AppState>, Query(page): Query<Page>) -> Response {
    if !(1..=MAX_LIMIT).contains(&page.limit) {
        let message = format!("limit must be between 1 and {MAX_LIMIT}");
        return error(StatusCode::BAD_REQUEST, message);
    }
    let Some(db) = state.db else {
        let message = "there is no database; start the service with DATABASE_URL";
        return error(StatusCode::SERVICE_UNAVAILABLE, message.to_string());
    };

    // Diesel's connections block, so query off the async runtime.
    match tokio::task::spawn_blocking(move || load_sources(&db, &page)).await {
        Ok(Ok(sources)) => Json(OfferSourcesResponse { sources }).into_response(),
        Ok(Err(message)) => error(StatusCode::INTERNAL_SERVER_ERROR, message),
        Err(join) => error(StatusCode::INTERNAL_SERVER_ERROR, join.to_string()),
    }
}

fn load_sources(db: &DbPool, page: &Page) -> Result<Vec<OfferSource>, String> {
    let mut connection = db.get().map_err(|error| error.to_string())?;
    let records = diesel::sql_query(SOURCES_QUERY)
        .bind::<BigInt, _>(i64::from(page.limit))
        .bind::<BigInt, _>(i64::from(page.offset))
        .load::<SourceRecord>(&mut connection)
        .map_err(|error| error.to_string())?;
    drop(connection);
    Ok(records
        .into_iter()
        .map(|record| OfferSource {
            id: record.id,
            offer_id: record.offer_id,
            url: record.url,
            retailer_code: record.retailer_code,
            storage: record.storage.0,
        })
        .collect())
}

fn error(status: StatusCode, error: String) -> Response {
    (status, Json(ErrorBody { error })).into_response()
}

#[cfg(test)]
mod postgres_tests {
    use super::*;
    use crate::platform::test_db::Database;
    use axum::body::Body;
    use axum::http::Request;
    use http_body_util::BodyExt;
    use serde_json::{Value, json};
    use tower::ServiceExt;

    #[test]
    #[ignore = "requires DATABASE_URL_TEST; run make test-postgres"]
    fn source_route_returns_storage_metadata_without_object_storage() {
        let db = Database::new();
        let runtime = tokio::runtime::Runtime::new().unwrap();
        let id = Uuid::now_v7();
        let offer = Uuid::now_v7();
        let body = "<html>ž 🔎</html>";
        let storage = OfferStorageMetadata {
            content_size: body.len() as u64,
            object_storage_id: Uuid::now_v7(),
            external_id: Uuid::now_v7(),
        };
        diesel::sql_query("INSERT INTO offer_details_raw (id,offer_id,url,retailer_code,storage) VALUES ($1,$2,'https://shop.test/a','ankerau',$3)")
            .bind::<SqlUuid,_>(id).bind::<SqlUuid,_>(offer)
            .bind::<Jsonb,_>(DbJson(storage)).execute(&mut db.pool.get().unwrap()).unwrap();
        let app = crate::app(AppState {
            db: Some(db.pool.clone()),
            ..Default::default()
        });
        let request = || {
            Request::builder()
                .uri("/api/development/offer-details/sources?limit=1")
                .header(
                    "authorization",
                    format!("Bearer {}", crate::platform::auth::BEARER_TOKEN),
                )
                .body(Body::empty())
                .unwrap()
        };
        let response = runtime.block_on(app.clone().oneshot(request())).unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let bytes = runtime
            .block_on(response.into_body().collect())
            .unwrap()
            .to_bytes();
        let value: Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(
            value,
            json!({"sources":[{"id":id,"offer_id":offer,"url":"https://shop.test/a","retailer_code":"ankerau","storage":storage}]})
        );
    }
}
