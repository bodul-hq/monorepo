mod record;

use diesel::prelude::*;
use diesel::r2d2::PoolError;
use diesel::sql_types::{Integer, Jsonb, Text, Uuid as SqlUuid};
use uuid::Uuid;

use self::record::{ExportStats, OfferExportPgRecord};
use super::conversion::{self, ConversionError};
use super::entity::{ExportSource, OfferExport, PreparedExport, SCHEMA_VERSION};
use super::handler::Store;
use crate::platform::db::{self, DbPool, Json};
use crate::platform::object_storage::{
    self, OFFER_EXPORT_BUCKET, ObjectStorage, ObjectStorageMetadata, STRUCTURALIZED_OFFERS_BUCKET,
};

#[derive(Debug, thiserror::Error)]
pub enum StoreError {
    #[error("connection pool error: {0}")]
    Connection(#[from] PoolError),
    #[error("database error: {}", db::describe(.0))]
    Database(#[from] diesel::result::Error),
    #[error(transparent)]
    ObjectStorage(#[from] object_storage::Error),
    #[error("export serialization failed: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error(transparent)]
    Conversion(#[from] ConversionError),
    #[error("export metadata does not match its immutable snapshot")]
    InvalidRecord,
    #[error("export source identity differs from the existing preparation")]
    SourceConflict,
    #[error("export completion was not persisted")]
    Incomplete,
}

pub struct PgStore {
    pool: DbPool,
    objects: ObjectStorage,
}

impl PgStore {
    pub fn new(pool: DbPool, objects: ObjectStorage) -> Self {
        Self { pool, objects }
    }

    /// Read only a completed export, validating the entire object reference.
    pub fn load(&self, id: Uuid) -> Result<Option<OfferExport>, StoreError> {
        let record =
            diesel::sql_query("SELECT * FROM offer_export WHERE id = $1 AND status = 'completed'")
                .bind::<SqlUuid, _>(id)
                .get_result::<OfferExportPgRecord>(&mut self.pool.get()?)
                .optional()?;
        record
            .map(|record| self.hydrate(record).map(|prepared| prepared.snapshot))
            .transpose()
    }

    fn find(&self, source: &ExportSource) -> Result<Option<OfferExportPgRecord>, StoreError> {
        Ok(diesel::sql_query(
            "SELECT * FROM offer_export WHERE offer_id = $1 AND source_object_id = $2 AND schema_version = $3",
        )
        .bind::<SqlUuid, _>(source.offer_id)
        .bind::<SqlUuid, _>(source.storage.object_storage_id)
        .bind::<Integer, _>(SCHEMA_VERSION as i32)
        .get_result(&mut self.pool.get()?)
        .optional()?)
    }

    fn hydrate(&self, record: OfferExportPgRecord) -> Result<PreparedExport, StoreError> {
        // The record has been loaded and its database connection released.
        let body = match (&record.prepared_body, &record.storage) {
            (Some(body), None) => body.clone(),
            (None, Some(storage)) => {
                self.objects
                    .get(OFFER_EXPORT_BUCKET, record.id, &storage.0)?
            }
            _ => return Err(StoreError::InvalidRecord),
        };
        record.into_prepared(body)
    }

    fn prepared_for(
        &self,
        record: OfferExportPgRecord,
        source: &ExportSource,
    ) -> Result<PreparedExport, StoreError> {
        if &record.source.0 != source {
            return Err(StoreError::SourceConflict);
        }
        self.hydrate(record)
    }
}

impl Store for PgStore {
    fn prepare(&self, source: ExportSource) -> Result<PreparedExport, StoreError> {
        if let Some(record) = self.find(&source)? {
            return self.prepared_for(record, &source);
        }

        let input = self
            .objects
            .get_offer(STRUCTURALIZED_OFFERS_BUCKET, &source.storage)?;
        let snapshot = conversion::convert(source.clone(), &input)?;
        let body = serde_json::to_string(&snapshot)?;
        let stats = Json(ExportStats::from(&snapshot));
        let references = Json(source.clone());
        // Only the winning preparation's IDs and exact bytes are used, including
        // when another consumer raced this one after reading the same source.
        diesel::sql_query(
            "INSERT INTO offer_export (id, offer_id, source_object_id, schema_version, source, stats, prepared_body)
             VALUES ($1,$2,$3,$4,$5,$6,$7)
             ON CONFLICT (offer_id, source_object_id, schema_version) DO NOTHING",
        )
        .bind::<SqlUuid, _>(snapshot.id)
        .bind::<SqlUuid, _>(source.offer_id)
        .bind::<SqlUuid, _>(source.storage.object_storage_id)
        .bind::<Integer, _>(SCHEMA_VERSION as i32)
        .bind::<Jsonb, _>(references)
        .bind::<Jsonb, _>(stats)
        .bind::<Text, _>(body)
        .execute(&mut self.pool.get()?)?;

        let record = self.find(&source)?.ok_or(StoreError::Incomplete)?;
        self.prepared_for(record, &source)
    }

    fn complete(&self, prepared: PreparedExport) -> Result<OfferExport, StoreError> {
        if prepared.storage.is_some() {
            return Ok(prepared.snapshot);
        }
        let id = prepared.snapshot.id;
        let object_storage_id = self
            .objects
            .create(OFFER_EXPORT_BUCKET, id, &prepared.body)?;
        let storage = Json(ObjectStorageMetadata {
            content_size: prepared.body.len() as u64,
            object_storage_id,
        });
        diesel::sql_query(
            "UPDATE offer_export SET storage = $2, prepared_body = NULL,
             status = 'completed', completed_at = now()
             WHERE id = $1 AND status = 'prepared' AND prepared_body = $3",
        )
        .bind::<SqlUuid, _>(id)
        .bind::<Jsonb, _>(storage)
        .bind::<Text, _>(&prepared.body)
        .execute(&mut self.pool.get()?)?;
        self.load(id)?.ok_or(StoreError::Incomplete)
    }
}
