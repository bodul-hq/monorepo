use chrono::{DateTime, Utc};
use diesel::prelude::*;
use diesel::r2d2::PoolError;
use diesel::{Insertable, Queryable, Selectable};
use uuid::Uuid;

use super::super::schema::{processed_sitemaps, raw_sitemap_documents};
use super::handler::{ProcessedSitemap, Store};
use crate::platform::db::DbPool;
use sitemap::RawSitemapDocument;

#[derive(Debug, thiserror::Error)]
pub enum StoreError {
    #[error("connection pool error: {0}")]
    Connection(#[from] PoolError),

    #[error("database error: {0}")]
    Database(#[from] diesel::result::Error),

    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("processed sitemap document is too large for storage")]
    DocumentTooLarge,
}

/// Postgres storage for this slice.
pub struct PgStore {
    pool: DbPool,
}

impl PgStore {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
}

#[derive(Queryable, Selectable)]
#[diesel(table_name = raw_sitemap_documents)]
struct RawSitemapDocumentRecord {
    url: String,
    last_modified: Option<DateTime<Utc>>,
    body: String,
    body_size: i32,
}

#[derive(Insertable)]
#[diesel(table_name = processed_sitemaps)]
struct NewProcessedSitemap {
    id: Uuid,
    retrieval_id: Uuid,
    retailer_code: String,
    document: serde_json::Value,
    url_count: i32,
    document_size: i32,
    processed_at: DateTime<Utc>,
}

impl Store for PgStore {
    fn raw_documents(&self, retrieval_id: Uuid) -> Result<Vec<RawSitemapDocument>, StoreError> {
        let mut connection = self.pool.get()?;
        let records = raw_sitemap_documents::table
            .filter(raw_sitemap_documents::retrieval_id.eq(retrieval_id))
            .order(raw_sitemap_documents::fetched_at.asc())
            .select(RawSitemapDocumentRecord::as_select())
            .load(&mut connection)?;

        Ok(records
            .into_iter()
            .map(|record| RawSitemapDocument {
                url: record.url,
                last_modified: record.last_modified,
                body: record.body,
                body_size: record.body_size as usize,
            })
            .collect())
    }

    fn store_processed(&self, sitemap: &ProcessedSitemap) -> Result<(), StoreError> {
        let document = serde_json::to_value(&sitemap.document)?;
        let document_size =
            i32::try_from(document.to_string().len()).map_err(|_| StoreError::DocumentTooLarge)?;
        let record = NewProcessedSitemap {
            id: sitemap.id,
            retrieval_id: sitemap.retrieval_id,
            retailer_code: sitemap.retailer_code.as_str(),
            document,
            url_count: sitemap.url_count,
            document_size,
            processed_at: sitemap.processed_at,
        };

        let mut connection = self.pool.get()?;
        diesel::insert_into(processed_sitemaps::table)
            .values(record)
            .execute(&mut connection)?;

        Ok(())
    }
}
