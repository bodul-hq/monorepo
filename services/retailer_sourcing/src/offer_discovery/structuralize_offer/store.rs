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
    fn normalized_offer(&self, offer_id: Uuid) -> Result<Option<NormalizedOffer>, StoreError> {
        let mut connection = self.pool.get()?;
        let record = offer_details_normalized::table
            .filter(offer_details_normalized::offer_id.eq(offer_id))
            .select((
                offer_details_normalized::offer_id,
                offer_details_normalized::storage,
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
        let body = serde_json::to_string(&entity.document)?;
        let storage = self
            .objects
            .create_offer(STRUCTURALIZED_OFFERS_BUCKET, &body)?;
        let record = StructuralizedOfferPgRecord::new(entity, storage);
        diesel::insert_into(offer_details_structuralized::table)
            .values(&record)
            .on_conflict(offer_details_structuralized::offer_id)
            .do_update()
            .set((
                offer_details_structuralized::structuralized_at.eq(diesel::dsl::now),
                offer_details_structuralized::storage.eq(&record.storage),
            ))
            .execute(&mut self.pool.get()?)?;
        Ok(storage)
    }
}
