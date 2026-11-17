use crate::platform::object_storage::OfferStorageMetadata as ObjectStorageOfferStorageMetadata;
use db::describe;
use diesel::dsl::now;
use diesel::insert_into;
use diesel::result::Error as ResultError;
use object_storage::Error as ObjectStorageError;
use offer_details_normalized::offer_id as offer_details_normalized_offer_id;
use offer_details_normalized::storage as offer_details_normalized_storage;
use offer_details_normalized::table as offer_details_normalized_table;
use offer_details_structuralized::offer_id as offer_details_structuralized_offer_id;
use offer_details_structuralized::storage as offer_details_structuralized_storage;
use offer_details_structuralized::structuralized_at;
use offer_details_structuralized::table as offer_details_structuralized_table;
use serde_json::Error as SerdeJsonError;
use serde_json::from_str;
use serde_json::to_string;
use thiserror::Error as ThiserrorError;
mod record;

use self::record::{NormalizedOfferPgRecord, StructuralizedOfferPgRecord};
use super::super::schema::{offer_details_normalized, offer_details_structuralized};
use super::entity::{NormalizedOffer, StructuralizedOffer};
use super::handler::Store;
use crate::platform::db::{self, DbPool};
use crate::platform::object_storage::{
    self, NORMALIZED_OFFERS_BUCKET, ObjectStorage, OfferStorageMetadata,
    STRUCTURALIZED_OFFERS_BUCKET,
};
use diesel::prelude::*;
use diesel::r2d2::PoolError;
use uuid::Uuid;

#[derive(Debug, ThiserrorError)]
pub enum StoreError {
    #[error("connection pool error: {0}")]
    Connection(#[from] PoolError),
    #[error("database error: {}", describe(.0))]
    Database(#[from] ResultError),
    #[error(transparent)]
    ObjectStorage(#[from] ObjectStorageError),
    #[error("serialization error: {0}")]
    Serialization(#[from] SerdeJsonError),
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
    fn normalized_offer(
        &self,
        offer_id: Uuid,
        storage: Option<ObjectStorageOfferStorageMetadata>,
    ) -> Result<Option<NormalizedOffer>, StoreError> {
        if let Some(storage) = storage {
            let body = self.objects.get_offer(NORMALIZED_OFFERS_BUCKET, &storage)?;
            return Ok(Some(NormalizedOffer {
                object_storage_id: storage.object_storage_id,
                offer_id,
                document: from_str(&body)?,
            }));
        }
        let mut connection = self.pool.get()?;
        let record = offer_details_normalized_table
            .filter(offer_details_normalized_offer_id.eq(offer_id))
            .select((
                offer_details_normalized_offer_id,
                offer_details_normalized_storage,
            ))
            .first::<NormalizedOfferPgRecord>(&mut connection)
            .optional()?;
        drop(connection);
        record
            .map(|record| {
                let body = self
                    .objects
                    .get_offer(NORMALIZED_OFFERS_BUCKET, &record.storage.0)?;
                record.into_entity(body)
            })
            .transpose()
    }

    fn store_structuralized_offer(
        &self,
        entity: &StructuralizedOffer,
    ) -> Result<OfferStorageMetadata, StoreError> {
        let body = to_string(&entity.document)?;
        let mut storage = self
            .objects
            .create_offer(STRUCTURALIZED_OFFERS_BUCKET, &body)?;
        storage.scraped_at = entity.scraped_at;
        let record = StructuralizedOfferPgRecord::new(entity, storage);
        insert_into(offer_details_structuralized_table)
            .values(&record)
            .on_conflict(offer_details_structuralized_offer_id)
            .do_update()
            .set((
                structuralized_at.eq(now),
                offer_details_structuralized_storage.eq(&record.storage),
            ))
            .execute(&mut self.pool.get()?)?;
        Ok(storage)
    }
}
