use crate::read_offers_identifiers::entity::RetailerOfferIdentifiers;
use bodul_shared::retailer::RetailerCode;
use diesel::{
    OptionalExtension, RunQueryDsl,
    sql_types::{Nullable, Text, Uuid as SqlUuid},
};
use retailer_sauce::identifier::{
    ExternalIdentifiers, Gtin, ProductIdentifiers, ShopifyHandle, ShopifyProductGid,
    ShopifyProductVariationGid, Sku,
};
use testimony_infra::prelude::DbPool;
use uuid::Uuid;

use crate::offers::{
    handler::{RetailerOfferIdentifiersError, RetailerOfferIdentifiersRepository},
    retailer_offer_id::RetailerOfferId,
};

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
        id: &RetailerOfferId,
        retailer_code: RetailerCode,
        url: &str,
    ) -> Result<(), RetailerOfferIdentifiersError> {
        let id = Uuid::parse_str(id.value()).map_err(err_identity_repository)?;
        let mut connection = self.pool.get().map_err(err_identity_repository)?;
        diesel::sql_query(
            "INSERT INTO read_offer_identities (id, retailer_code, url) VALUES ($1, $2, $3)",
        )
        .bind::<SqlUuid, _>(id)
        .bind::<Text, _>(retailer_code.as_str())
        .bind::<Text, _>(url)
        .execute(&mut connection)
        .map_err(err_identity_repository)?;
        Ok(())
    }

    fn find(
        &self,
        retailer_code: RetailerCode,
        url: &str,
    ) -> Result<Option<RetailerOfferIdentifiers>, RetailerOfferIdentifiersError> {
        #[derive(diesel::QueryableByName)]
        struct IdentityRow {
            #[diesel(sql_type = SqlUuid)]
            id: Uuid,
            #[diesel(sql_type = Nullable<Text>)]
            gtin: Option<String>,
            #[diesel(sql_type = Nullable<Text>)]
            sku: Option<String>,
            #[diesel(sql_type = Nullable<Text>)]
            shopify_product_gid: Option<String>,
            #[diesel(sql_type = Nullable<Text>)]
            shopify_product_variation_gid: Option<String>,
            #[diesel(sql_type = Nullable<Text>)]
            shopify_handle: Option<String>,
        }

        let mut connection = self.pool.get().map_err(err_identity_repository)?;
        let row = diesel::sql_query(
            "SELECT id, gtin, sku, shopify_product_gid, shopify_product_variation_gid, shopify_handle FROM read_offer_identities WHERE retailer_code = $1 AND url = $2",
        )
        .bind::<Text, _>(retailer_code.as_str())
        .bind::<Text, _>(url)
        .get_result::<IdentityRow>(&mut connection)
        .optional()
        .map_err(err_identity_repository)?;

        row.map(|row| {
            let identifiers = ProductIdentifiers {
                gtin: row
                    .gtin
                    .map(Gtin::try_from)
                    .transpose()
                    .map_err(err_identity_repository)?,
                sku: row
                    .sku
                    .map(Sku::try_from)
                    .transpose()
                    .map_err(err_identity_repository)?,
                external_identifiers: ExternalIdentifiers {
                    shopify_product_gid: row
                        .shopify_product_gid
                        .map(ShopifyProductGid::try_from)
                        .transpose()
                        .map_err(err_identity_repository)?,
                    shopify_product_variation_gid: row
                        .shopify_product_variation_gid
                        .map(ShopifyProductVariationGid::try_from)
                        .transpose()
                        .map_err(err_identity_repository)?,
                    shopify_handle: row
                        .shopify_handle
                        .map(ShopifyHandle::try_from)
                        .transpose()
                        .map_err(err_identity_repository)?,
                },
            };
            identifiers.guard().map_err(err_identity_repository)?;
            let identity = RetailerOfferIdentifiers {
                identifiers,
                id: RetailerOfferId::new(row.id.to_string()),
                retailer_code,
                url: url.to_owned(),
            };
            Ok(identity)
        })
        .transpose()
    }
}

fn err_identity_repository(error: impl std::fmt::Display) -> RetailerOfferIdentifiersError {
    RetailerOfferIdentifiersError(error.to_string())
}
