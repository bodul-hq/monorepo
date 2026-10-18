//! Private PostgreSQL metadata and domain mappings.
use super::super::super::schema::offer_details_extracted;
use super::super::entity::{ExtractedOffer, RawOffer};
use super::StoreError;
use crate::platform::db::Json;
use crate::platform::object_storage::OfferStorageMetadata;
use diesel::prelude::*;
use uuid::Uuid;

#[derive(Queryable)]
pub(super) struct RawOfferPgRecord {
    pub offer_id: Uuid,
    pub storage: Json<OfferStorageMetadata>,
}

impl RawOfferPgRecord {
    pub fn into_entity(self, body: String) -> Result<RawOffer, StoreError> {
        Ok(RawOffer {
            offer_id: self.offer_id,
            object_storage_id: self.storage.0.object_storage_id,
            body,
        })
    }
}

#[derive(Insertable)]
#[diesel(table_name = offer_details_extracted)]
pub(super) struct ExtractedOfferPgRecord {
    pub id: Uuid,
    pub offer_id: Uuid,
    pub storage: Json<OfferStorageMetadata>,
}

impl ExtractedOfferPgRecord {
    pub fn new(entity: &ExtractedOffer, storage: OfferStorageMetadata) -> Self {
        Self {
            id: Uuid::now_v7(),
            offer_id: entity.offer_id,
            storage: Json(storage),
        }
    }
}
