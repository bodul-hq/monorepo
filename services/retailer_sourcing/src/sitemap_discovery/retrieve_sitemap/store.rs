mod record;

use diesel::prelude::*;
use diesel::r2d2::PoolError;

use self::record::RawSitemapDocumentPgRecord;
use super::super::schema::sitemaps_raw;
use super::entity::{RetrievedSitemapDocument, StoredSitemapDocument};
use super::handler::Store;
use crate::platform::db::{self, DbPool};
use crate::platform::object_storage::{self, ObjectStorage, RAW_SITEMAPS_BUCKET};

#[derive(Debug, thiserror::Error)]
pub enum StoreError {
    #[error("connection pool error: {0}")]
    Connection(#[from] PoolError),
    #[error("database error: {}", db::describe(.0))]
    Database(#[from] diesel::result::Error),
    #[error(transparent)]
    ObjectStorage(#[from] object_storage::Error),
    #[error("raw sitemap body is too large for storage: {0} bytes")]
    BodyTooLarge(usize),
    #[error("raw sitemap size does not match the body")]
    InvalidBodySize,
}

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
    fn store_documents(
        &self,
        documents: &[RetrievedSitemapDocument],
    ) -> Result<Vec<StoredSitemapDocument>, StoreError> {
        for document in documents {
            record::validate(document)?;
        }
        let stored = documents
            .iter()
            .map(|document| {
                let object_storage_id = self.objects.create(
                    RAW_SITEMAPS_BUCKET,
                    document.id,
                    &document.document.body,
                )?;
                Ok(StoredSitemapDocument {
                    sitemap: document.clone(),
                    object_storage_id,
                })
            })
            .collect::<Result<Vec<_>, StoreError>>()?;
        if stored.is_empty() {
            return Ok(stored);
        }
        let records: Vec<_> = stored
            .iter()
            .map(RawSitemapDocumentPgRecord::from)
            .collect();
        // No connection or transaction is held during the HTTP uploads. An
        // unsuccessful local commit can leave unreferenced immutable objects.
        self.pool.get()?.transaction(|connection| {
            diesel::insert_into(sitemaps_raw::table)
                .values(&records)
                .execute(connection)
        })?;
        Ok(stored)
    }
}
