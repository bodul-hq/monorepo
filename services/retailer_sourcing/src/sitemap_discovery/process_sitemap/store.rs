mod record;

use diesel::prelude::*;
use diesel::r2d2::PoolError;
use uuid::Uuid;

use self::record::{ProcessedSitemapPgRecord, RawSitemapDocumentPgRecord};
use super::super::schema::{sitemaps_processed, sitemaps_raw};
use super::entity::{ProcessedSitemap, RawSitemapDocument};
use super::handler::Store;
use crate::platform::db::{self, DbPool};
use crate::platform::object_storage::{
    self, ObjectStorage, ObjectStorageMetadata, PROCESSED_SITEMAPS_BUCKET, RAW_SITEMAPS_BUCKET,
    Stored,
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

    #[error("processed sitemap document is too large for storage")]
    DocumentTooLarge,

    #[error("sitemap contains {0} URLs, which exceeds the supported storage maximum")]
    TooManyUrls(usize),

    #[error(
        "storage.content_size must be a nonnegative integer within the supported storage range"
    )]
    InvalidContentSize,
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
    fn raw_documents(&self, retrieval_id: Uuid) -> Result<Vec<RawSitemapDocument>, StoreError> {
        let mut connection = self.pool.get()?;
        let records = sitemaps_raw::table
            .filter(sitemaps_raw::retrieval_id.eq(retrieval_id))
            .order((sitemaps_raw::fetched_at.asc(), sitemaps_raw::id.asc()))
            .select(RawSitemapDocumentPgRecord::as_select())
            .load::<RawSitemapDocumentPgRecord>(&mut connection)?;
        drop(connection);
        records
            .into_iter()
            .map(|record| {
                i32::try_from(record.storage.0.content_size)
                    .map_err(|_| StoreError::InvalidContentSize)?;
                let body = self
                    .objects
                    .get(RAW_SITEMAPS_BUCKET, record.id, &record.storage.0)?;
                record.into_entity(body)
            })
            .collect()
    }

    fn store_processed(
        &self,
        sitemap: &ProcessedSitemap,
    ) -> Result<Stored<ProcessedSitemap>, StoreError> {
        let body = serde_json::to_string(sitemap.document())?;
        record::validate(sitemap, &body)?;
        let object_storage_id =
            self.objects
                .create(PROCESSED_SITEMAPS_BUCKET, sitemap.id, &body)?;
        let storage = ObjectStorageMetadata {
            content_size: body.len() as u64,
            object_storage_id,
        };
        let record = ProcessedSitemapPgRecord::new(sitemap, storage)?;
        diesel::insert_into(sitemaps_processed::table)
            .values(record)
            .execute(&mut self.pool.get()?)?;
        Ok(Stored {
            entity: sitemap.clone(),
            object_storage_id,
        })
    }
}
