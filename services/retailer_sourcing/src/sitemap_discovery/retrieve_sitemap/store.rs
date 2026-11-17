mod record;

use crate::platform::db::describe;
use diesel::insert_into;
use diesel::result::Error;
use object_storage::Error as ObjectStorageError;
use record::validate;
use sitemaps_raw::table;

use diesel::prelude::*;
use diesel::r2d2::PoolError;

use self::record::RawSitemapDocumentPgRecord;
use super::super::schema::sitemaps_raw;
use super::entity::{RetrievedSitemapDocument, StoredSitemapDocument};
use super::handler::Store;
use crate::platform::db::DbPool;
use crate::platform::object_storage::{self, ObjectStorage, RAW_SITEMAPS_BUCKET};

#[derive(Debug, thiserror::Error)]
pub enum StoreError {
    #[error("connection pool error: {0}")]
    Connection(#[from] PoolError),
    #[error("database error: {}", describe(.0))]
    Database(#[from] Error),
    #[error(transparent)]
    ObjectStorage(#[from] ObjectStorageError),
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
            validate(document)?;
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
        self.pool
            .get()?
            .transaction(|connection| insert_into(table).values(&records).execute(connection))?;
        Ok(stored)
    }
}
