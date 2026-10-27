use super::{
    RetailerOfferIdentifiers, RetailerOfferIdentifiersRepository,
    RetailerOfferIdentifiersRepositoryError, err_identity_missing, err_insert, err_update,
    validate, validate_update,
};
use diesel::{
    RunQueryDsl,
    sql_types::{Nullable, Text, Uuid as SqlUuid},
};
use testimony_infra::prelude::DbPool;

pub struct PostgresRetailerOfferIdentifiersRepository {
    pool: DbPool,
}

impl PostgresRetailerOfferIdentifiersRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
}

impl RetailerOfferIdentifiersRepository for PostgresRetailerOfferIdentifiersRepository {
    fn insert(
        &self,
        entity: &RetailerOfferIdentifiers,
    ) -> Result<(), RetailerOfferIdentifiersRepositoryError> {
        let id = validate(entity)?;
        let mut connection = self.pool.get().map_err(err_insert)?;
        diesel::sql_query(
            "INSERT INTO read_offer_identities (id, retailer_code, url, gtin, sku, shopify_product_gid, shopify_product_variation_gid, shopify_handle) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
        )
        .bind::<SqlUuid, _>(id)
        .bind::<Text, _>(entity.retailer_code.as_str())
        .bind::<Text, _>(&entity.url)
        .bind::<Nullable<Text>, _>(entity.identifiers.gtin.as_ref().map(|value| &value.value))
        .bind::<Nullable<Text>, _>(entity.identifiers.sku.as_ref().map(|value| &value.value))
        .bind::<Nullable<Text>, _>(entity.identifiers.external_identifiers.shopify_product_gid.as_ref().map(|value| &value.value))
        .bind::<Nullable<Text>, _>(entity.identifiers.external_identifiers.shopify_product_variation_gid.as_ref().map(|value| &value.value))
        .bind::<Nullable<Text>, _>(entity.identifiers.external_identifiers.shopify_handle.as_ref().map(|value| &value.value))
        .execute(&mut connection)
        .map_err(err_insert)?;
        Ok(())
    }

    fn update_identifiers(
        &self,
        id: &crate::offers::retailer_offer_id::RetailerOfferId,
        identifiers: &retailer_sauce::identifier::ProductIdentifiers,
    ) -> Result<(), RetailerOfferIdentifiersRepositoryError> {
        let id = validate_update(id, identifiers)?;
        let count = diesel::sql_query(
            "UPDATE read_offer_identities SET gtin=$2, sku=$3, shopify_product_gid=$4, shopify_product_variation_gid=$5, shopify_handle=$6 WHERE id=$1",
        )
        .bind::<SqlUuid, _>(id)
        .bind::<Nullable<Text>, _>(identifiers.gtin.as_ref().map(|value| &value.value))
        .bind::<Nullable<Text>, _>(identifiers.sku.as_ref().map(|value| &value.value))
        .bind::<Nullable<Text>, _>(identifiers.external_identifiers.shopify_product_gid.as_ref().map(|value| &value.value))
        .bind::<Nullable<Text>, _>(identifiers.external_identifiers.shopify_product_variation_gid.as_ref().map(|value| &value.value))
        .bind::<Nullable<Text>, _>(identifiers.external_identifiers.shopify_handle.as_ref().map(|value| &value.value))
        .execute(&mut self.pool.get().map_err(err_update)?)
        .map_err(err_update)?;
        if count == 0 {
            return Err(err_identity_missing());
        }
        Ok(())
    }
}
