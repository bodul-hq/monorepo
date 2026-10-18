//! PostgreSQL mappings for downloaded offer pages.
use super::super::super::schema::offer_details_raw;
use super::super::entity::RawOffer;
use super::StoreError;
use crate::platform::db::Json;
use crate::platform::object_storage::OfferStorageMetadata;
use diesel::Insertable;
use uuid::Uuid;

#[derive(Insertable)]
#[diesel(table_name = offer_details_raw)]
pub(super) struct RawOfferPgRecord<'a> {
    pub id: Uuid,
    pub offer_id: Uuid,
    pub url: &'a str,
    pub storage: Json<OfferStorageMetadata>,
    pub retailer_code: String,
}

impl<'a> RawOfferPgRecord<'a> {
    pub fn new(entity: &'a RawOffer, storage: OfferStorageMetadata) -> Self {
        Self {
            id: Uuid::now_v7(),
            offer_id: entity.offer_id,
            url: &entity.url,
            retailer_code: entity.retailer_code.as_str(),
            storage: Json(storage),
        }
    }
}

pub(super) fn validate(entity: &RawOffer) -> Result<(), StoreError> {
    stored_size(entity.body.len())?;
    Ok(())
}

fn stored_size(size: usize) -> Result<i32, StoreError> {
    i32::try_from(size).map_err(|_| StoreError::BodyTooLarge(size))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn checks_storage_size_bounds() {
        assert_eq!(stored_size(0).unwrap(), 0);
        assert_eq!(stored_size(i32::MAX as usize).unwrap(), i32::MAX);
        assert!(matches!(
            stored_size(i32::MAX as usize + 1),
            Err(StoreError::BodyTooLarge(_))
        ));
    }
}
