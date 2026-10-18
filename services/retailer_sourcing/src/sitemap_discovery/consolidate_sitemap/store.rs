mod record;

use bodul_shared::retailer::RetailerCodeConversionError;
use diesel::prelude::*;
use diesel::r2d2::PoolError;
use uuid::Uuid;

use self::record::{ConsolidatedSitemapPgRecord, ProcessedSitemapPgRecord};
use super::super::schema::{sitemaps_consolidated, sitemaps_processed};
use super::entity::{ConsolidatedSitemap, ProcessedSitemap};
use super::handler::Store;
use crate::platform::db::{self, DbPool};
use crate::platform::object_storage::{
    self, CONSOLIDATED_SITEMAPS_BUCKET, ObjectStorage, ObjectStorageMetadata,
    PROCESSED_SITEMAPS_BUCKET, Stored,
};

#[derive(Debug, thiserror::Error)]
pub enum StoreError {
    #[error(transparent)]
    ObjectStorage(#[from] object_storage::Error),
    #[error("connection pool error: {0}")]
    Connection(#[from] PoolError),

    #[error("database error: {}", db::describe(.0))]
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
    objects: ObjectStorage,
}

impl PgStore {
    pub fn new(pool: DbPool, objects: ObjectStorage) -> Self {
        Self { pool, objects }
    }
}

impl Store for PgStore {
    fn processed_sitemap(&self, id: Uuid) -> Result<ProcessedSitemap, StoreError> {
        let mut connection = self.pool.get()?;
        let record = sitemaps_processed::table
            .find(id)
            .select(ProcessedSitemapPgRecord::as_select())
            .get_result::<ProcessedSitemapPgRecord>(&mut connection)?;
        drop(connection);
        let body = self
            .objects
            .get(PROCESSED_SITEMAPS_BUCKET, id, &record.storage.0)?;
        record.into_entity(&body)
    }

    fn store_consolidated(
        &self,
        consolidated: &ConsolidatedSitemap,
    ) -> Result<Stored<ConsolidatedSitemap>, StoreError> {
        let body = serde_json::to_string(consolidated.links())?;
        record::validate(consolidated, &body)?;
        let object_storage_id =
            self.objects
                .create(CONSOLIDATED_SITEMAPS_BUCKET, consolidated.id, &body)?;
        let record = ConsolidatedSitemapPgRecord::new(
            consolidated,
            ObjectStorageMetadata {
                content_size: body.len() as u64,
                object_storage_id,
            },
        )?;
        diesel::insert_into(sitemaps_consolidated::table)
            .values(record)
            .execute(&mut self.pool.get()?)?;
        Ok(Stored {
            entity: consolidated.clone(),
            object_storage_id,
        })
    }
}
