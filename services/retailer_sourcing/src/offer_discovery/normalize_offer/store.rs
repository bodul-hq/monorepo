mod record;

use self::record::{ExtractedOfferPgRecord, NormalizedOfferPgRecord};
use super::super::schema::{offer_details_extracted, offer_details_normalized};
use super::entity::{ExtractedOffer, NormalizedOffer};
use super::handler::Store;
use crate::platform::db::{self, DbPool};
use crate::platform::object_storage::{
    self, EXTRACTED_OFFERS_BUCKET, NORMALIZED_OFFERS_BUCKET, ObjectStorage, Stored,
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
    fn extracted_offer(&self, offer_id: Uuid) -> Result<Option<ExtractedOffer>, StoreError> {
        let mut connection = self.pool.get()?;
        let record = offer_details_extracted::table
            .filter(offer_details_extracted::offer_id.eq(offer_id))
            .select((
                offer_details_extracted::offer_id,
                offer_details_extracted::storage,
            ))
            .first::<ExtractedOfferPgRecord>(&mut connection)
            .optional()?;
        drop(connection);
        record
            .map(|record| {
                let body = self
                    .objects
                    .get_offer(EXTRACTED_OFFERS_BUCKET, &record.storage.0)?;
                record.into_entity(body)
            })
            .transpose()
    }

    fn store_normalized_offer(
        &self,
        entity: &NormalizedOffer,
    ) -> Result<Stored<NormalizedOffer>, StoreError> {
        let body = serde_json::to_string(&entity.document)?;
        let storage = self.objects.create_offer(NORMALIZED_OFFERS_BUCKET, &body)?;
        let record = NormalizedOfferPgRecord::new(entity, storage);
        diesel::insert_into(offer_details_normalized::table)
            .values(&record)
            .on_conflict(offer_details_normalized::offer_id)
            .do_update()
            .set((
                offer_details_normalized::normalized_at.eq(diesel::dsl::now),
                offer_details_normalized::storage.eq(&record.storage),
            ))
            .execute(&mut self.pool.get()?)?;
        Ok(Stored {
            entity: entity.clone(),
            object_storage_id: storage.object_storage_id,
        })
    }
}
