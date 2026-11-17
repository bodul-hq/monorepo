mod record;

use crate::platform::db::describe;
use crate::platform::object_storage::OfferStorageMetadata;
use diesel::dsl::now;
use diesel::insert_into;
use diesel::result::Error as DieselResultError;
use object_storage::Error;
use offer_details_raw::fetched_at;
use offer_details_raw::offer_id;
use offer_details_raw::retailer_code as offer_details_raw_retailer_code;
use offer_details_raw::storage as offer_details_raw_storage;
use offer_details_raw::table;
use record::validate;

use diesel::prelude::*;
use diesel::r2d2::PoolError;

use self::record::RawOfferPgRecord;
use super::super::schema::offer_details_raw;
use super::entity::RawOffer;
use super::handler::Store;
use crate::platform::db::DbPool;
use crate::platform::object_storage::{self, ObjectStorage, RAW_OFFERS_BUCKET};

#[derive(Debug, thiserror::Error)]
pub enum StoreError {
    #[error(transparent)]
    ObjectStorage(#[from] Error),
    #[error("connection pool error: {0}")]
    Connection(#[from] PoolError),

    #[error("database error: {}", describe(.0))]
    Database(#[from] DieselResultError),

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
    fn store_raw_offer(&self, entity: &RawOffer) -> Result<OfferStorageMetadata, StoreError> {
        validate(entity)?;
        let mut storage = self.objects.create_offer(RAW_OFFERS_BUCKET, &entity.body)?;
        storage.scraped_at = entity.scraped_at;
        let record = RawOfferPgRecord::new(entity, storage);
        insert_into(table)
            .values(&record)
            .on_conflict(offer_id)
            .do_update()
            .set((
                offer_details_raw_retailer_code.eq(&record.retailer_code),
                offer_details_raw_storage.eq(&record.storage),
                fetched_at.eq(now),
            ))
            .execute(&mut self.pool.get()?)?;
        Ok(storage)
    }
}
