use diesel::prelude::*;
use diesel::r2d2::PoolError;
use serde::Deserialize;
use uuid::Uuid;

use super::handler::Store;
use crate::platform::db::DbPool;

// The columns this slice reads from sitemap discovery's table.
diesel::table! {
    consolidated_sitemaps (id) {
        id -> Uuid,
        content -> Jsonb,
    }
}

#[derive(Debug, thiserror::Error)]
pub enum StoreError {
    #[error("connection pool error: {0}")]
    Connection(#[from] PoolError),

    #[error("database error: {0}")]
    Database(#[from] diesel::result::Error),

    #[error("unreadable consolidated sitemap content: {0}")]
    Content(#[from] serde_json::Error),
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

/// The part of a consolidated sitemap's content this slice needs.
#[derive(Deserialize)]
struct ConsolidatedContent {
    product: Vec<String>,
}

impl Store for PgStore {
    fn product_urls(&self, consolidated_sitemap_id: Uuid) -> Result<Vec<String>, StoreError> {
        let mut connection = self.pool.get()?;
        let content: serde_json::Value = consolidated_sitemaps::table
            .find(consolidated_sitemap_id)
            .select(consolidated_sitemaps::content)
            .get_result(&mut connection)?;

        let content: ConsolidatedContent = serde_json::from_value(content)?;
        Ok(content.product)
    }
}
