//! Lists complete initial-fill GraphQL artifacts for development dumping.

use axum::Json;
use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use diesel::prelude::*;
use diesel::sql_types::{BigInt, Jsonb, Text, Uuid as SqlUuid};
use serde::{Deserialize, Serialize};
use shared::retailer::RetailerCode;
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;

use crate::platform::db::{DbPool, Json as DbJson};
use crate::platform::object_storage::OfferStorageMetadata;
use crate::platform::state::AppState;

/// The most source references one page may hold.
const MAX_LIMIT: u32 = 1000;

/// Read persisted terminal events regardless of consumer status. These artifacts
/// have no legacy offer projection. Queue retention therefore bounds this listing.
const SOURCES_QUERY: &str = "
    SELECT id, (payload::jsonb ->> 'run_id')::uuid AS run_id,
           payload::jsonb #>> '{source,GraphQl,product_id}' AS product_id,
           payload::jsonb -> 'retailer_code' AS retailer_code,
           payload::jsonb ->> 'fetched_at' AS fetched_at,
           NULLIF(payload::jsonb -> 'context', 'null'::jsonb) AS context,
           payload::jsonb -> 'storage' AS storage
    FROM event_entries
    WHERE event_type = 'OfferDetailsFetched'
      AND payload::jsonb #>> '{source,GraphQl,product_id}' IS NOT NULL
    ORDER BY (payload::jsonb ->> 'fetched_at')::timestamptz DESC, id
    LIMIT $1 OFFSET $2";

#[derive(Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct PageV2 {
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
pub struct OfferSourcesV2Response {
    sources: Vec<OfferSourceV2>,
}

#[derive(Serialize, ToSchema)]
pub struct OfferSourceV2 {
    #[schema(value_type = String, format = "uuid")]
    id: Uuid,
    /// The initial-fill run that fetched this artifact.
    #[schema(value_type = String, format = "uuid")]
    run_id: Uuid,
    /// The complete Shopify product GID.
    product_id: String,
    /// The retailer the offer belongs to.
    #[schema(value_type = String)]
    retailer_code: RetailerCode,
    /// When the artifact was fetched (RFC 3339).
    fetched_at: String,
    /// Reference to the complete GraphQL JSON in object storage's `offer_details_raw` bucket.
    storage: OfferStorageMetadata,
    #[schema(value_type = Option<Object>)]
    context: Option<retailer_sauce::shopify::markets::MarketContext>,
}

#[derive(QueryableByName)]
struct SourceRecord {
    #[diesel(sql_type = SqlUuid)]
    id: Uuid,
    #[diesel(sql_type = SqlUuid)]
    run_id: Uuid,
    #[diesel(sql_type = Text)]
    product_id: String,
    #[diesel(sql_type = Jsonb)]
    retailer_code: DbJson<RetailerCode>,
    #[diesel(sql_type = Text)]
    fetched_at: String,
    #[diesel(sql_type = Jsonb)]
    storage: DbJson<OfferStorageMetadata>,
    #[diesel(sql_type = diesel::sql_types::Nullable<Jsonb>)]
    context: Option<DbJson<retailer_sauce::shopify::markets::MarketContext>>,
}

#[derive(Serialize, ToSchema)]
pub struct ErrorBodyV2 {
    error: String,
}

/// Lists complete GraphQL offer details v2 artifacts, newest first.
#[utoipa::path(
    get,
    path = "/development/offer-details-v2/sources",
    params(PageV2),
    security(("bearer" = [])),
    responses(
        (status = 200, description = "A page of sources", body = OfferSourcesV2Response),
        (status = 400, description = "Limit out of range", body = ErrorBodyV2),
        (status = 401, description = "Missing or wrong bearer token", body = ErrorBodyV2),
        (status = 500, description = "The sources couldn't be read", body = ErrorBodyV2),
        (status = 503, description = "No database (no DATABASE_URL)", body = ErrorBodyV2),
    )
)]
pub async fn list_sources(State(state): State<AppState>, Query(page): Query<PageV2>) -> Response {
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
        Ok(Ok(sources)) => Json(OfferSourcesV2Response { sources }).into_response(),
        Ok(Err(message)) => error(StatusCode::INTERNAL_SERVER_ERROR, message),
        Err(join) => error(StatusCode::INTERNAL_SERVER_ERROR, join.to_string()),
    }
}

fn load_sources(db: &DbPool, page: &PageV2) -> Result<Vec<OfferSourceV2>, String> {
    let mut connection = db.get().map_err(|error| error.to_string())?;
    let records = diesel::sql_query(SOURCES_QUERY)
        .bind::<BigInt, _>(i64::from(page.limit))
        .bind::<BigInt, _>(i64::from(page.offset))
        .load::<SourceRecord>(&mut connection)
        .map_err(|error| error.to_string())?;
    drop(connection);
    Ok(records
        .into_iter()
        .map(|record| OfferSourceV2 {
            id: record.id,
            run_id: record.run_id,
            product_id: record.product_id,
            retailer_code: record.retailer_code.0,
            fetched_at: record.fetched_at,
            storage: record.storage.0,
            context: record.context.map(|v| v.0),
        })
        .collect())
}

fn error(status: StatusCode, error: String) -> Response {
    (status, Json(ErrorBodyV2 { error })).into_response()
}

#[cfg(test)]
mod postgres_tests {
    use super::*;
    use crate::platform::test_db::Database;
    use axum::{body::Body, http::Request};
    use http_body_util::BodyExt;
    use serde_json::{Value, json};
    use tower::ServiceExt;

    #[test]
    #[ignore = "requires DATABASE_URL_TEST; run make test-postgres"]
    fn sources_include_terminal_events_in_time_order_and_normalize_historical_codes() {
        let db = Database::new();
        let storage = OfferStorageMetadata {
            scraped_at: None,
            content_size: 123,
            object_storage_id: Uuid::now_v7(),
            external_id: Uuid::now_v7(),
        };
        let run_id = Uuid::now_v7();
        let older = Uuid::now_v7();
        let newest = Uuid::now_v7();
        let tied = Uuid::now_v7();
        let mut conn = db.pool.get().unwrap();
        for (id, kind, source, retailer, time, status) in [
            (
                older,
                "OfferDetailsFetched",
                json!({"GraphQl":{"product_id":"gid://shopify/Product/1"}}),
                "AnkerAu",
                "2026-11-01T10:00:00Z",
                0,
            ),
            (
                newest,
                "OfferDetailsFetched",
                json!({"GraphQl":{"product_id":"gid://shopify/Product/2"}}),
                "ankerca",
                "2026-11-01T12:00:00Z",
                2,
            ),
            (
                tied,
                "OfferDetailsFetched",
                json!({"GraphQl":{"product_id":"gid://shopify/Product/3"}}),
                "ankerca",
                "2026-11-01T12:00:00Z",
                0,
            ),
            (
                Uuid::now_v7(),
                "OfferDetailsFetched",
                json!({"Html":{"url":"https://example.test/p"}}),
                "ankerau",
                "2026-11-02T12:00:00Z",
                0,
            ),
            (
                Uuid::now_v7(),
                "UnrelatedEvent",
                json!({"GraphQl":{"product_id":"gid://shopify/Product/4"}}),
                "ankerau",
                "2026-11-03T12:00:00Z",
                0,
            ),
        ] {
            let mut payload = json!({"run_id":run_id,"retailer_code":retailer,"source":source,"fetched_at":time,"storage":storage});
            if id == newest || id == tied {
                payload["context"] = json!({"country":"CA","language":if id==newest {"EN"} else {"FR"},"currency":"CAD"});
            }
            diesel::sql_query(
                "INSERT INTO event_entries (id,event_type,payload,status) VALUES ($1,$2,$3,$4)",
            )
            .bind::<SqlUuid, _>(id)
            .bind::<Text, _>(kind)
            .bind::<Text, _>(payload.to_string())
            .bind::<diesel::sql_types::Integer, _>(status)
            .execute(&mut conn)
            .unwrap();
        }
        drop(conn);
        let runtime = tokio::runtime::Runtime::new().unwrap();
        let app = crate::app(AppState {
            db: Some(db.pool.clone()),
            ..Default::default()
        });
        for (offset, id, retailer) in [
            (0, newest, "ankerca"),
            (1, tied, "ankerca"),
            (2, older, "ankerau"),
        ] {
            let response = runtime
                .block_on(
                    app.clone().oneshot(
                        Request::builder()
                            .uri(format!(
                                "/api/development/offer-details-v2/sources?limit=1&offset={offset}"
                            ))
                            .header(
                                "authorization",
                                format!("Bearer {}", crate::platform::auth::BEARER_TOKEN),
                            )
                            .body(Body::empty())
                            .unwrap(),
                    ),
                )
                .unwrap();
            assert_eq!(response.status(), StatusCode::OK);
            let body = runtime
                .block_on(response.into_body().collect())
                .unwrap()
                .to_bytes();
            let value: Value = serde_json::from_slice(&body).unwrap();
            assert_eq!(value["sources"].as_array().unwrap().len(), 1);
            assert_eq!(value["sources"][0]["id"], id.to_string());
            assert_eq!(value["sources"][0]["run_id"], run_id.to_string());
            assert_eq!(value["sources"][0]["retailer_code"], retailer);
            assert_eq!(value["sources"][0]["storage"], json!(storage));
            if id == newest || id == tied {
                assert_eq!(
                    value["sources"][0]["context"]["language"],
                    if id == newest { "EN" } else { "FR" }
                );
            } else {
                assert!(value["sources"][0]["context"].is_null());
            }
        }
        assert!(
            load_sources(
                &db.pool,
                &PageV2 {
                    limit: 1,
                    offset: 3
                }
            )
            .unwrap()
            .is_empty()
        );
    }
}
