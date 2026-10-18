mod record;

use diesel::prelude::*;
use diesel::r2d2::PoolError;
use uuid::Uuid;

use self::record::{ConsolidatedSitemapPgRecord, sitemaps_consolidated};
use super::entity::ConsolidatedSitemap;
use super::handler::Store;
use crate::platform::db::{self, DbPool};
use crate::platform::object_storage::{self, CONSOLIDATED_SITEMAPS_BUCKET, ObjectStorage};

#[derive(Debug, thiserror::Error)]
pub enum StoreError {
    #[error(transparent)]
    ObjectStorage(#[from] object_storage::Error),
    #[error("connection pool error: {0}")]
    Connection(#[from] PoolError),

    #[error("database error: {}", db::describe(.0))]
    Database(#[from] diesel::result::Error),

    #[error("unreadable consolidated sitemap content: {0}")]
    Content(#[from] serde_json::Error),
}

/// Postgres storage for this slice.
pub struct PgStore {
    pool: DbPool,
    objects: ObjectStorage,
}

impl PgStore {
    pub fn new(pool: DbPool, objects: ObjectStorage) -> Self {
        Self { pool, objects }
    }
}

impl Store for PgStore {
    fn consolidated_sitemap(&self, id: Uuid) -> Result<ConsolidatedSitemap, StoreError> {
        let mut connection = self.pool.get()?;
        let record = sitemaps_consolidated::table
            .find(id)
            .select(ConsolidatedSitemapPgRecord::as_select())
            .get_result::<ConsolidatedSitemapPgRecord>(&mut connection)?;
        drop(connection);
        let body = self
            .objects
            .get(CONSOLIDATED_SITEMAPS_BUCKET, id, &record.storage.0)?;
        record.into_entity(&body)
    }
}
