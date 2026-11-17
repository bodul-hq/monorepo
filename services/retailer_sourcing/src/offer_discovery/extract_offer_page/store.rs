mod record;

use crate::platform::db::describe;
use crate::platform::object_storage::OfferStorageMetadata;
use diesel::dsl::now;
use diesel::insert_into;
use diesel::result::Error;
use object_storage::Error as ObjectStorageError;
use offer_details_extracted::extracted_at;
use offer_details_extracted::offer_id as offer_details_extracted_offer_id;
use offer_details_extracted::storage as offer_details_extracted_storage;
use offer_details_extracted::table as offer_details_extracted_table;
use offer_details_raw::offer_id as offer_details_raw_offer_id;
use offer_details_raw::storage as offer_details_raw_storage;
use offer_details_raw::table;
use serde_json::Error as SerdeJsonError;
use serde_json::to_string;

use self::record::{ExtractedOfferPgRecord, RawOfferPgRecord};
use super::super::schema::{offer_details_extracted, offer_details_raw};
use super::entity::{ExtractedOffer, RawOffer};
use super::handler::Store;
use crate::platform::db::DbPool;
use crate::platform::object_storage::{
    self, EXTRACTED_OFFERS_BUCKET, ObjectStorage, RAW_OFFERS_BUCKET,
};
use diesel::prelude::*;
use diesel::r2d2::PoolError;
use uuid::Uuid;

#[derive(Debug, thiserror::Error)]
pub enum StoreError {
    #[error("connection pool error: {0}")]
    Connection(#[from] PoolError),
    #[error("database error: {}", describe(.0))]
    Database(#[from] Error),
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
    fn raw_offer(
        &self,
        offer_id: Uuid,
        storage: Option<OfferStorageMetadata>,
    ) -> Result<Option<RawOffer>, StoreError> {
        if let Some(storage) = storage {
            let body = self.objects.get_offer(RAW_OFFERS_BUCKET, &storage)?;
            return Ok(Some(RawOffer {
                object_storage_id: storage.object_storage_id,
                offer_id,
                body,
            }));
        }
        let mut connection = self.pool.get()?;
        let record = table
            .filter(offer_details_raw_offer_id.eq(offer_id))
            .select((offer_details_raw_offer_id, offer_details_raw_storage))
            .first::<RawOfferPgRecord>(&mut connection)
            .optional()?;
        drop(connection);
        record
            .map(|record| {
                let body = self
                    .objects
                    .get_offer(RAW_OFFERS_BUCKET, &record.storage.0)?;
                record.into_entity(body)
            })
            .transpose()
    }

    fn store_extracted_offer(
        &self,
        entity: &ExtractedOffer,
    ) -> Result<OfferStorageMetadata, StoreError> {
        let body = to_string(&entity.document)?;
        let mut storage = self.objects.create_offer(EXTRACTED_OFFERS_BUCKET, &body)?;
        storage.scraped_at = entity.scraped_at;
        let record = ExtractedOfferPgRecord::new(entity, storage);
        insert_into(offer_details_extracted_table)
            .values(&record)
            .on_conflict(offer_details_extracted_offer_id)
            .do_update()
            .set((
                extracted_at.eq(now),
                offer_details_extracted_storage.eq(&record.storage),
            ))
            .execute(&mut self.pool.get()?)?;
        Ok(storage)
    }
}
