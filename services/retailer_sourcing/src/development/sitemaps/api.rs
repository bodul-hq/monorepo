//! HTTP entry point: lists the fetched sitemap files, a page at a time.

use axum::Json;
use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use diesel::prelude::*;
use diesel::sql_types::{BigInt, Jsonb, Nullable, Text, Uuid as SqlUuid};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;

use crate::platform::db::{DbPool, Json as DbJson};
use crate::platform::object_storage::{ObjectStorage, ObjectStorageMetadata, RAW_SITEMAPS_BUCKET};
use crate::platform::state::AppState;

/// The most sources one page may hold. Each carries a whole sitemap file.
const MAX_LIMIT: u32 = 100;

/// Each fetched sitemap file with its URL, crawl, body and the retailer of the
/// `RetrieveSitemap` command that fetched it, newest first, skipping `$2` and
/// returning up to `$1`.
const SOURCES_QUERY: &str = "
    SELECT r.id, r.retrieval_id, r.url, r.storage, (
        SELECT c.payload::jsonb ->> 'retailer_code'
        FROM command_entries c
        WHERE c.command_type = 'RetrieveSitemap'
            AND c.payload::jsonb ->> 'retrieval_id' = r.retrieval_id::text
        LIMIT 1
    ) AS retailer_code
    FROM sitemaps_raw r
    ORDER BY r.fetched_at DESC, r.id
    LIMIT $1 OFFSET $2";

#[derive(Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct Page {
    /// How many sources to return, from 1 to 100.
    #[serde(default = "default_limit")]
    #[param(default = 20, minimum = 1, maximum = 100)]
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
pub struct SourcesResponse {
    sources: Vec<Source>,
}

#[derive(Serialize, ToSchema)]
pub struct Source {
    #[schema(value_type = String, format = "uuid")]
    id: Uuid,
    /// The crawl that fetched the file.
    #[schema(value_type = String, format = "uuid")]
    retrieval_id: Uuid,
    /// The URL the file was fetched from.
    url: String,
    /// `null` when the command that fetched the file is gone.
    retailer_code: Option<String>,
    /// The sitemap file as fetched.
    body: String,
}

#[derive(QueryableByName)]
struct SourceRecord {
    #[diesel(sql_type = SqlUuid)]
    id: Uuid,
    #[diesel(sql_type = SqlUuid)]
    retrieval_id: Uuid,
    #[diesel(sql_type = Text)]
    url: String,
    #[diesel(sql_type = Nullable<Text>)]
    retailer_code: Option<String>,
    #[diesel(sql_type = Jsonb)]
    storage: DbJson<ObjectStorageMetadata>,
}

#[derive(Serialize, ToSchema)]
pub struct ErrorBody {
    error: String,
}

/// Lists the fetched sitemap files, newest first.
#[utoipa::path(
    get,
    path = "/development/sitemaps/sources",
    params(Page),
    security(("bearer" = [])),
    responses(
        (status = 200, description = "A page of sources", body = SourcesResponse),
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
    let (Some(db), Some(objects)) = (state.db, state.object_storage) else {
        let message = "there is no database; start the service with DATABASE_URL";
        return error(StatusCode::SERVICE_UNAVAILABLE, message.to_string());
    };

    // Diesel's connections block, so query off the async runtime.
    match tokio::task::spawn_blocking(move || load_sources(&db, &objects, &page)).await {
        Ok(Ok(sources)) => Json(SourcesResponse { sources }).into_response(),
        Ok(Err(message)) => error(StatusCode::INTERNAL_SERVER_ERROR, message),
        Err(join) => error(StatusCode::INTERNAL_SERVER_ERROR, join.to_string()),
    }
}

fn load_sources(db: &DbPool, objects: &ObjectStorage, page: &Page) -> Result<Vec<Source>, String> {
    let mut connection = db.get().map_err(|error| error.to_string())?;
    let records = diesel::sql_query(SOURCES_QUERY)
        .bind::<BigInt, _>(i64::from(page.limit))
        .bind::<BigInt, _>(i64::from(page.offset))
        .load::<SourceRecord>(&mut connection)
        .map_err(|error| error.to_string())?;
    drop(connection);
    records
        .into_iter()
        .map(|record| {
            let body = objects
                .get(RAW_SITEMAPS_BUCKET, record.id, &record.storage.0)
                .map_err(|error| error.to_string())?;
            Ok(Source {
                id: record.id,
                retrieval_id: record.retrieval_id,
                url: record.url,
                retailer_code: record.retailer_code,
                body,
            })
        })
        .collect()
}

fn error(status: StatusCode, error: String) -> Response {
    (status, Json(ErrorBody { error })).into_response()
}

#[cfg(test)]
mod postgres_tests {
    use super::*;
    use crate::platform::test_db::Database;
    use crate::platform::test_objects::Server;
    use axum::body::Body;
    use axum::http::Request;
    use http_body_util::BodyExt;
    use serde_json::{Value, json};
    use tower::ServiceExt;

    #[test]
    #[ignore = "requires DATABASE_URL_TEST; run make test-postgres"]
    fn source_route_fetches_object_bodies_and_reports_failures() {
        let db = Database::new();
        let server = Server::new();
        let objects = server.client();
        let id = Uuid::now_v7();
        let retrieval = Uuid::now_v7();
        let body = "<urlset>ž 🔎</urlset>";
        let object_id = objects.create(RAW_SITEMAPS_BUCKET, id, body).unwrap();
        diesel::sql_query("INSERT INTO sitemaps_raw (id,retrieval_id,url,storage) VALUES ($1,$2,'https://shop.test/a',$3)")
            .bind::<SqlUuid,_>(id).bind::<SqlUuid,_>(retrieval)
            .bind::<Jsonb,_>(DbJson(ObjectStorageMetadata { content_size:body.len() as u64, object_storage_id:object_id }))
            .execute(&mut db.pool.get().unwrap()).unwrap();
        let app = crate::app(AppState {
            db: Some(db.pool.clone()),
            object_storage: Some(objects),
            ..Default::default()
        });
        let request = || {
            Request::builder()
                .uri("/api/development/sitemaps/sources?limit=1")
                .header(
                    "authorization",
                    format!("Bearer {}", crate::platform::auth::BEARER_TOKEN),
                )
                .body(Body::empty())
                .unwrap()
        };
        let response = server
            .runtime
            .block_on(app.clone().oneshot(request()))
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let bytes = server
            .runtime
            .block_on(response.into_body().collect())
            .unwrap()
            .to_bytes();
        let value: Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(
            value,
            json!({"sources":[{"id":id,"retrieval_id":retrieval,"url":"https://shop.test/a","retailer_code":null,"body":body}]})
        );
        for (status, body) in [(404, "{}"), (503, "{}"), (200, "malformed")] {
            server.data.lock().unwrap().get_response = Some((status, body.into()));
            let response = server
                .runtime
                .block_on(app.clone().oneshot(request()))
                .unwrap();
            assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
        }
    }
}
