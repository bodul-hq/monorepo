mod record;

use crate::platform::db::describe;
use crate::platform::object_storage::OfferStorageMetadata;
use diesel::dsl::now;
use diesel::insert_into;
use diesel::result::Error;
use object_storage::Error as ObjectStorageError;
use offer_details_extracted::offer_id as offer_details_extracted_offer_id;
use offer_details_extracted::storage as offer_details_extracted_storage;
use offer_details_extracted::table;
use offer_details_normalized::normalized_at;
use offer_details_normalized::offer_id as offer_details_normalized_offer_id;
use offer_details_normalized::storage as offer_details_normalized_storage;
use offer_details_normalized::table as offer_details_normalized_table;
use serde_json::Error as SerdeJsonError;
use serde_json::from_str;
use serde_json::to_string;

use self::record::{ExtractedOfferPgRecord, NormalizedOfferPgRecord};
use super::super::schema::{offer_details_extracted, offer_details_normalized};
use super::entity::{ExtractedOffer, NormalizedOffer};
use super::handler::Store;
use crate::platform::db::DbPool;
use crate::platform::object_storage::{
    self, EXTRACTED_OFFERS_BUCKET, NORMALIZED_OFFERS_BUCKET, ObjectStorage,
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
    fn extracted_offer(
        &self,
        offer_id: Uuid,
        storage: Option<OfferStorageMetadata>,
    ) -> Result<Option<ExtractedOffer>, StoreError> {
        if let Some(storage) = storage {
            let body = self.objects.get_offer(EXTRACTED_OFFERS_BUCKET, &storage)?;
            return Ok(Some(ExtractedOffer {
                object_storage_id: storage.object_storage_id,
                offer_id,
                document: from_str(&body)?,
            }));
        }
        let mut connection = self.pool.get()?;
        let record = table
            .filter(offer_details_extracted_offer_id.eq(offer_id))
            .select((
                offer_details_extracted_offer_id,
                offer_details_extracted_storage,
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
    ) -> Result<OfferStorageMetadata, StoreError> {
        let body = to_string(&entity.document)?;
        let mut storage = self.objects.create_offer(NORMALIZED_OFFERS_BUCKET, &body)?;
        storage.scraped_at = entity.scraped_at;
        let record = NormalizedOfferPgRecord::new(entity, storage);
        insert_into(offer_details_normalized_table)
            .values(&record)
            .on_conflict(offer_details_normalized_offer_id)
            .do_update()
            .set((
                normalized_at.eq(now),
                offer_details_normalized_storage.eq(&record.storage),
            ))
            .execute(&mut self.pool.get()?)?;
        Ok(storage)
    }
}
