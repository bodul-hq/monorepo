mod record;

use self::record::{ExtractedOfferPgRecord, RawOfferPgRecord};
use super::super::schema::{offer_details_extracted, offer_details_raw};
use super::entity::{ExtractedOffer, RawOffer};
use super::handler::Store;
use crate::platform::db::{self, DbPool};
use crate::platform::object_storage::{
    self, EXTRACTED_OFFERS_BUCKET, ObjectStorage, RAW_OFFERS_BUCKET, Stored,
};
use diesel::prelude::*;
use diesel::r2d2::PoolError;
use uuid::Uuid;

#[derive(Debug, thiserror::Error)]
pub enum StoreError {
    #[error("connection pool error: {0}")]
    Connection(#[from] PoolError),
    #[error("database error: {}", db::describe(.0))]
    Database(#[from] diesel::result::Error),
    #[error(transparent)]
    ObjectStorage(#[from] object_storage::Error),
    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
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
    fn raw_offer(&self, offer_id: Uuid) -> Result<Option<RawOffer>, StoreError> {
        let mut connection = self.pool.get()?;
        let record = offer_details_raw::table
            .filter(offer_details_raw::offer_id.eq(offer_id))
            .select((offer_details_raw::offer_id, offer_details_raw::storage))
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
    ) -> Result<Stored<ExtractedOffer>, StoreError> {
        let body = serde_json::to_string(&entity.document)?;
        let storage = self.objects.create_offer(EXTRACTED_OFFERS_BUCKET, &body)?;
        let record = ExtractedOfferPgRecord::new(entity, storage);
        diesel::insert_into(offer_details_extracted::table)
            .values(&record)
            .on_conflict(offer_details_extracted::offer_id)
            .do_update()
            .set((
                offer_details_extracted::extracted_at.eq(diesel::dsl::now),
                offer_details_extracted::storage.eq(&record.storage),
            ))
            .execute(&mut self.pool.get()?)?;
        Ok(Stored {
            entity: entity.clone(),
            object_storage_id: storage.object_storage_id,
        })
    }
}
