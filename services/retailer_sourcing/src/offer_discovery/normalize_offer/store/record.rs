//! Private PostgreSQL metadata and domain mappings.
use super::super::super::schema::offer_details_normalized;
use super::super::entity::{ExtractedOffer, NormalizedOffer};
use super::StoreError;
use crate::platform::db::Json;
use crate::platform::object_storage::OfferStorageMetadata;
use diesel::prelude::*;
use uuid::Uuid;

#[derive(Queryable)]
pub(super) struct ExtractedOfferPgRecord {
    pub offer_id: Uuid,
    pub storage: Json<OfferStorageMetadata>,
}

impl ExtractedOfferPgRecord {
    pub fn into_entity(self, body: String) -> Result<ExtractedOffer, StoreError> {
        Ok(ExtractedOffer {
            offer_id: self.offer_id,
            object_storage_id: self.storage.0.object_storage_id,
            document: serde_json::from_str(&body)?,
        })
    }
}

#[derive(Insertable)]
#[diesel(table_name = offer_details_normalized)]
pub(super) struct NormalizedOfferPgRecord {
    pub id: Uuid,
    pub offer_id: Uuid,
    pub storage: Json<OfferStorageMetadata>,
}

impl NormalizedOfferPgRecord {
    pub fn new(entity: &NormalizedOffer, storage: OfferStorageMetadata) -> Self {
        Self {
            id: Uuid::now_v7(),
            offer_id: entity.offer_id,
            storage: Json(storage),
        }
    }
}
