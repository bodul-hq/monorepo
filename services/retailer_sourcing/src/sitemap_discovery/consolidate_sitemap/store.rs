use bodul_shared::retailer::{RetailerCode, RetailerCodeConversionError};
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use diesel::r2d2::PoolError;
use diesel::{Insertable, Queryable, Selectable};
use uuid::Uuid;

use super::super::schema::{consolidated_sitemaps, processed_sitemaps};
use super::handler::{ConsolidatedSitemap, ProcessedSitemap, Store};
use crate::platform::db::DbPool;

#[derive(Debug, thiserror::Error)]
pub enum StoreError {
    #[error("connection pool error: {0}")]
    Connection(#[from] PoolError),

    #[error("database error: {0}")]
    Database(#[from] diesel::result::Error),

    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("invalid stored retailer code: {0}")]
    InvalidRetailerCode(#[from] RetailerCodeConversionError),

    #[error("{0}")]
    Unexpected(String),
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
#[diesel(table_name = processed_sitemaps)]
struct ProcessedSitemapRecord {
    id: Uuid,
    retrieval_id: Uuid,
    retailer_code: String,
    document: serde_json::Value,
}

#[derive(Insertable)]
#[diesel(table_name = consolidated_sitemaps)]
struct NewConsolidatedSitemap {
    id: Uuid,
    processed_sitemap_id: Uuid,
    retrieval_id: Uuid,
    retailer_code: String,
    content: serde_json::Value,
    product_count: i32,
    catalog_count: i32,
    content_count: i32,
    not_interested_count: i32,
    unknown_count: i32,
    content_size: i32,
    consolidated_at: DateTime<Utc>,
}

impl Store for PgStore {
    fn processed_sitemap(&self, id: Uuid) -> Result<ProcessedSitemap, StoreError> {
        let mut connection = self.pool.get()?;
        let record = processed_sitemaps::table
            .find(id)
            .select(ProcessedSitemapRecord::as_select())
            .get_result(&mut connection)?;

        Ok(ProcessedSitemap {
            id: record.id,
            retrieval_id: record.retrieval_id,
            retailer_code: RetailerCode::try_from(record.retailer_code.as_str())?,
            document: serde_json::from_value(record.document)?,
        })
    }

    fn store_consolidated(&self, consolidated: &ConsolidatedSitemap) -> Result<(), StoreError> {
        let links = &consolidated.links;
        let content = serde_json::to_value(links)?;
        let content_size = i32::try_from(content.to_string().len()).map_err(|_| {
            StoreError::Unexpected(
                "consolidated sitemap content is too large for storage".to_string(),
            )
        })?;
        let record = NewConsolidatedSitemap {
            id: consolidated.id,
            processed_sitemap_id: consolidated.processed_sitemap_id,
            retrieval_id: consolidated.retrieval_id,
            retailer_code: consolidated.retailer_code.as_str(),
            content,
            product_count: stored_count(links.product.len())?,
            catalog_count: stored_count(links.catalog.len())?,
            content_count: stored_count(links.content.len())?,
            not_interested_count: stored_count(links.not_interested.len())?,
            unknown_count: stored_count(links.unknown.len())?,
            content_size,
            consolidated_at: Utc::now(),
        };

        let mut connection = self.pool.get()?;
        diesel::insert_into(consolidated_sitemaps::table)
            .values(record)
            .execute(&mut connection)?;

        Ok(())
    }
}

fn stored_count(count: usize) -> Result<i32, StoreError> {
    i32::try_from(count).map_err(|_| {
        StoreError::Unexpected(format!(
            "consolidated sitemap count is too large for storage: {count}"
        ))
    })
}
