mod record;

use crate::platform::db::describe;
use diesel::result::Error as DieselResultError;
use object_storage::Error;
use serde_json::Error as SerdeJsonError;
use sitemaps_consolidated::table;

use diesel::prelude::*;
use diesel::r2d2::PoolError;
use uuid::Uuid;

use self::record::{ConsolidatedSitemapPgRecord, sitemaps_consolidated};
use super::entity::ConsolidatedSitemap;
use super::handler::Store;
use crate::platform::db::DbPool;
use crate::platform::object_storage::{self, CONSOLIDATED_SITEMAPS_BUCKET, ObjectStorage};

#[derive(Debug, thiserror::Error)]
pub enum StoreError {
    #[error(transparent)]
    ObjectStorage(#[from] Error),
    #[error("connection pool error: {0}")]
    Connection(#[from] PoolError),

    #[error("database error: {}", describe(.0))]
    Database(#[from] DieselResultError),

    #[error("unreadable consolidated sitemap content: {0}")]
    Content(#[from] SerdeJsonError),
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
        let record = table
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
