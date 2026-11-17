use db::describe;
use diesel::insert_into;
use diesel::result::Error as ResultError;
use object_storage::Error as ObjectStorageError;
use record::validate;
use serde_json::Error as SerdeJsonError;
use serde_json::to_string;
use sitemaps_consolidated::table as sitemaps_consolidated_table;
use sitemaps_processed::table as sitemaps_processed_table;
use thiserror::Error as ThiserrorError;
mod record;

use diesel::prelude::*;
use diesel::r2d2::PoolError;
use shared::retailer::RetailerCodeConversionError;
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

#[derive(Debug, ThiserrorError)]
pub enum StoreError {
    #[error(transparent)]
    ObjectStorage(#[from] ObjectStorageError),
    #[error("connection pool error: {0}")]
    Connection(#[from] PoolError),

    #[error("database error: {}", describe(.0))]
    Database(#[from] ResultError),

    #[error("serialization error: {0}")]
    Serialization(#[from] SerdeJsonError),

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
        let record = sitemaps_processed_table
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
        let body = to_string(consolidated.links())?;
        validate(consolidated, &body)?;
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
        insert_into(sitemaps_consolidated_table)
            .values(record)
            .execute(&mut self.pool.get()?)?;
        Ok(Stored {
            entity: consolidated.clone(),
            object_storage_id,
        })
    }
}
