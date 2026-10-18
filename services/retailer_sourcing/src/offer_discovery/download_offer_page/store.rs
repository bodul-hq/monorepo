mod record;

use diesel::prelude::*;
use diesel::r2d2::PoolError;

use self::record::RawOfferPgRecord;
use super::super::schema::offer_details_raw;
use super::entity::RawOffer;
use super::handler::Store;
use crate::platform::db::{self, DbPool};
use crate::platform::object_storage::{self, ObjectStorage, RAW_OFFERS_BUCKET, Stored};

#[derive(Debug, thiserror::Error)]
pub enum StoreError {
    #[error(transparent)]
    ObjectStorage(#[from] object_storage::Error),
    #[error("connection pool error: {0}")]
    Connection(#[from] PoolError),

    #[error("database error: {}", db::describe(.0))]
    Database(#[from] diesel::result::Error),

    #[error("offer page body is too large for storage: {0} bytes")]
    BodyTooLarge(usize),
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
    fn store_raw_offer(&self, entity: &RawOffer) -> Result<Stored<RawOffer>, StoreError> {
        record::validate(entity)?;
        let storage = self.objects.create_offer(RAW_OFFERS_BUCKET, &entity.body)?;
        let record = RawOfferPgRecord::new(entity, storage);
        diesel::insert_into(offer_details_raw::table)
            .values(&record)
            .on_conflict(offer_details_raw::offer_id)
            .do_update()
            .set((
                offer_details_raw::retailer_code.eq(&record.retailer_code),
                offer_details_raw::storage.eq(&record.storage),
                offer_details_raw::fetched_at.eq(diesel::dsl::now),
            ))
            .execute(&mut self.pool.get()?)?;
        Ok(Stored {
            entity: entity.clone(),
            object_storage_id: storage.object_storage_id,
        })
    }
}
