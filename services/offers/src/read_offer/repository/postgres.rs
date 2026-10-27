use super::{
    RetailerOffer, RetailerOfferRepository, RetailerOfferRepositoryError, err_insert, validate,
};
use diesel::{
    Connection, RunQueryDsl,
    sql_types::{Text, Uuid as SqlUuid},
};
use testimony_infra::prelude::DbPool;

pub struct PostgresRetailerOfferRepository {
    pool: DbPool,
}

impl PostgresRetailerOfferRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
}

impl RetailerOfferRepository for PostgresRetailerOfferRepository {
    fn update_content(
        &self,
        id: &super::RetailerOfferId,
        content: &retailer_sauce::structuralized::content::Content,
    ) -> Result<(), RetailerOfferRepositoryError> {
        content.guard().map_err(super::err_update)?;
        let id = uuid::Uuid::parse_str(id.value()).map_err(super::err_update)?;
        let count = diesel::sql_query("UPDATE read_offers SET description=$2 WHERE id=$1")
            .bind::<SqlUuid, _>(id)
            .bind::<diesel::sql_types::Nullable<Text>, _>(&content.description)
            .execute(&mut self.pool.get().map_err(super::err_update)?)
            .map_err(super::err_update)?;
        require_updated_offer(count)
    }
    fn update_item_condition(
        &self,
        id: &super::RetailerOfferId,
        item_condition: &Option<retailer_sauce::item_condition::ItemCondition>,
    ) -> Result<(), RetailerOfferRepositoryError> {
        if let Some(value) = item_condition {
            value.guard().map_err(super::err_update)?;
        }
        let id = uuid::Uuid::parse_str(id.value()).map_err(super::err_update)?;
        let count = diesel::sql_query("UPDATE read_offers SET item_condition=$2 WHERE id=$1")
            .bind::<SqlUuid, _>(id)
            .bind::<diesel::sql_types::Nullable<diesel::sql_types::Jsonb>, _>(
                item_condition
                    .as_ref()
                    .map(serde_json::to_value)
                    .transpose()
                    .map_err(super::err_update)?,
            )
            .execute(&mut self.pool.get().map_err(super::err_update)?)
            .map_err(super::err_update)?;
        require_updated_offer(count)
    }
    fn update_availability(
        &self,
        id: &super::RetailerOfferId,
        availability: &Option<retailer_sauce::availability::Availability>,
    ) -> Result<(), RetailerOfferRepositoryError> {
        if let Some(value) = availability {
            value.guard().map_err(super::err_update)?;
        }
        let id = uuid::Uuid::parse_str(id.value()).map_err(super::err_update)?;
        let count = diesel::sql_query("UPDATE read_offers SET availability=$2 WHERE id=$1")
            .bind::<SqlUuid, _>(id)
            .bind::<diesel::sql_types::Nullable<diesel::sql_types::Jsonb>, _>(
                availability
                    .as_ref()
                    .map(serde_json::to_value)
                    .transpose()
                    .map_err(super::err_update)?,
            )
            .execute(&mut self.pool.get().map_err(super::err_update)?)
            .map_err(super::err_update)?;
        require_updated_offer(count)
    }
    fn update_pricing(
        &self,
        id: &super::RetailerOfferId,
        pricing: &crate::offers::event::Pricing,
    ) -> Result<(), RetailerOfferRepositoryError> {
        pricing.guard().map_err(super::err_update)?;
        let id = uuid::Uuid::parse_str(id.value()).map_err(super::err_update)?;
        let count = diesel::sql_query("UPDATE read_offers SET price_amount=$2,currency=$3,was_price_amount=$4,was_price_currency=$5 WHERE id=$1").bind::<SqlUuid,_>(id)
        .bind::<diesel::sql_types::BigInt,_>(pricing.price.amount())
        .bind::<Text,_>(pricing.price.currency().as_str())
        .bind::<diesel::sql_types::Nullable<diesel::sql_types::BigInt>,_>(pricing.was_price.as_ref().map(|v|v.amount()))
        .bind::<diesel::sql_types::Nullable<Text>,_>(pricing.was_price.as_ref().map(|v|v.currency().as_str()))
        .execute(&mut self.pool.get().map_err(super::err_update)?).map_err(super::err_update)?;
        require_updated_offer(count)
    }
    fn update_identifiers(
        &self,
        id: &crate::offers::retailer_offer_id::RetailerOfferId,
        identifiers: &retailer_sauce::identifier::ProductIdentifiers,
    ) -> Result<(), RetailerOfferRepositoryError> {
        identifiers.guard().map_err(super::err_update)?;
        let id = uuid::Uuid::parse_str(id.value()).map_err(super::err_update)?;
        let count = diesel::sql_query("UPDATE read_offers SET gtin=$2, sku=$3, shopify_product_gid=$4, shopify_product_variation_gid=$5, shopify_handle=$6 WHERE id=$1").bind::<SqlUuid, _>(id)
        .bind::<diesel::sql_types::Nullable<Text>, _>(identifiers.gtin.as_ref().map(|value| &value.value))
            .bind::<diesel::sql_types::Nullable<Text>, _>(identifiers.sku.as_ref().map(|value| &value.value))
            .bind::<diesel::sql_types::Nullable<Text>, _>(identifiers.external_identifiers.shopify_product_gid.as_ref().map(|value| &value.value))
            .bind::<diesel::sql_types::Nullable<Text>, _>(identifiers.external_identifiers.shopify_product_variation_gid.as_ref().map(|value| &value.value))
            .bind::<diesel::sql_types::Nullable<Text>, _>(identifiers.external_identifiers.shopify_handle.as_ref().map(|value| &value.value))
            .execute(&mut self.pool.get().map_err(super::err_update)?).map_err(super::err_update)?;
        require_updated_offer(count)
    }
    fn update_localization(
        &self,
        id: &crate::offers::retailer_offer_id::RetailerOfferId,
        localization: &retailer_sauce::localization::Localization,
    ) -> Result<(), RetailerOfferRepositoryError> {
        localization.guard().map_err(super::err_update)?;
        let id = uuid::Uuid::parse_str(id.value()).map_err(super::err_update)?;
        let market = localization
            .target_market
            .as_ref()
            .map(serde_json::to_value)
            .transpose()
            .map_err(super::err_update)?;
        let count =
            diesel::sql_query("UPDATE read_offers SET target_market=$2, language=$3 WHERE id=$1")
                .bind::<SqlUuid, _>(id)
                .bind::<diesel::sql_types::Nullable<diesel::sql_types::Jsonb>, _>(market)
                .bind::<diesel::sql_types::Nullable<Text>, _>(&localization.language)
                .execute(&mut self.pool.get().map_err(super::err_update)?)
                .map_err(super::err_update)?;
        require_updated_offer(count)
    }
    fn update_scraped_at(
        &self,
        id: &super::RetailerOfferId,
        scraped_at: chrono::DateTime<chrono::Utc>,
    ) -> Result<(), RetailerOfferRepositoryError> {
        let id = uuid::Uuid::parse_str(id.value()).map_err(super::err_update)?;
        let count = diesel::sql_query(
            "UPDATE read_offers SET scraped_at = GREATEST(scraped_at, $2) WHERE id = $1",
        )
        .bind::<SqlUuid, _>(id)
        .bind::<diesel::sql_types::Timestamptz, _>(scraped_at)
        .execute(&mut self.pool.get().map_err(super::err_update)?)
        .map_err(super::err_update)?;
        require_updated_offer(count)
    }

    fn images(
        &self,
        id: &super::RetailerOfferId,
    ) -> Result<Vec<String>, RetailerOfferRepositoryError> {
        #[derive(diesel::QueryableByName)]
        struct Image {
            #[diesel(sql_type = Text)]
            image_url: String,
        }
        let id = uuid::Uuid::parse_str(id.value()).map_err(super::err_update)?;
        let rows = diesel::sql_query(
            "SELECT image_url FROM read_offer_images WHERE offer_id = $1 ORDER BY position",
        )
        .bind::<SqlUuid, _>(id)
        .load::<Image>(&mut self.pool.get().map_err(super::err_update)?)
        .map_err(super::err_update)?;
        Ok(rows.into_iter().map(|row| row.image_url).collect())
    }
    fn add_image(
        &self,
        event: &crate::offers::event::OfferImageAddedEvent,
    ) -> Result<(), RetailerOfferRepositoryError> {
        if !shared::images::is_valid(&event.image_url) {
            return Err(super::err_update("invalid image URL"));
        }
        let id = uuid::Uuid::parse_str(event.id.value()).map_err(super::err_update)?;
        let count = diesel::sql_query(
            "INSERT INTO read_offer_images (offer_id, image_url, position) VALUES ($1,$2,$3)
            ON CONFLICT (offer_id, position) DO UPDATE SET image_url = EXCLUDED.image_url
            WHERE read_offer_images.image_url = EXCLUDED.image_url",
        )
        .bind::<SqlUuid, _>(id)
        .bind::<Text, _>(&event.image_url)
        .bind::<diesel::sql_types::BigInt, _>(i64::from(event.position))
        .execute(&mut self.pool.get().map_err(super::err_update)?)
        .map_err(super::err_update)?;
        if count == 0 {
            return Err(super::err_update("conflicting image addition"));
        }
        Ok(())
    }

    fn insert(&self, entity: &RetailerOffer) -> Result<(), RetailerOfferRepositoryError> {
        let id = validate(entity)?;
        let market = entity
            .localization
            .target_market
            .as_ref()
            .map(serde_json::to_value)
            .transpose()
            .map_err(err_insert)?;
        let availability = entity
            .facts
            .availability
            .as_ref()
            .map(serde_json::to_value)
            .transpose()
            .map_err(err_insert)?;
        let item_condition = entity
            .facts
            .item_condition
            .as_ref()
            .map(serde_json::to_value)
            .transpose()
            .map_err(err_insert)?;
        let mut connection = self.pool.get().map_err(err_insert)?;
        connection
            .transaction::<_, diesel::result::Error, _>(|connection| {
                diesel::sql_query(
                    "INSERT INTO read_offers (id, retailer_code, url, title, price_amount, currency, created_at, scraped_at, target_market, language, gtin, sku, shopify_product_gid, shopify_product_variation_gid, shopify_handle, was_price_amount, was_price_currency, availability, item_condition, description)
                     VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20)",
                )
                .bind::<SqlUuid, _>(id)
                .bind::<Text, _>(entity.retailer_code.as_str())
                .bind::<Text, _>(&entity.url)
                .bind::<Text, _>(&entity.title)
                .bind::<diesel::sql_types::BigInt, _>(entity.price.amount())
                .bind::<Text, _>(entity.price.currency().as_str())
                .bind::<diesel::sql_types::Timestamptz, _>(entity.created_at)
                .bind::<diesel::sql_types::Nullable<diesel::sql_types::Timestamptz>, _>(entity.scraped_at)
                .bind::<diesel::sql_types::Nullable<diesel::sql_types::Jsonb>, _>(&market)
                .bind::<diesel::sql_types::Nullable<Text>, _>(&entity.localization.language)
                .bind::<diesel::sql_types::Nullable<Text>, _>(entity.identifiers.gtin.as_ref().map(|value| &value.value))
                .bind::<diesel::sql_types::Nullable<Text>, _>(entity.identifiers.sku.as_ref().map(|value| &value.value))
                .bind::<diesel::sql_types::Nullable<Text>, _>(entity.identifiers.external_identifiers.shopify_product_gid.as_ref().map(|value| &value.value))
                .bind::<diesel::sql_types::Nullable<Text>, _>(entity.identifiers.external_identifiers.shopify_product_variation_gid.as_ref().map(|value| &value.value))
                .bind::<diesel::sql_types::Nullable<Text>, _>(entity.identifiers.external_identifiers.shopify_handle.as_ref().map(|value| &value.value))
                .bind::<diesel::sql_types::Nullable<diesel::sql_types::BigInt>,_>(entity.facts.was_price.as_ref().map(|v|v.amount()))
                .bind::<diesel::sql_types::Nullable<Text>,_>(entity.facts.was_price.as_ref().map(|v|v.currency().as_str()))
                .bind::<diesel::sql_types::Nullable<diesel::sql_types::Jsonb>,_>(&availability)
                .bind::<diesel::sql_types::Nullable<diesel::sql_types::Jsonb>,_>(&item_condition)
                .bind::<diesel::sql_types::Nullable<Text>,_>(&entity.facts.content.description)
                .execute(connection)?;
                for (position, image_url) in entity.images.iter().enumerate() {
                    diesel::sql_query(
                        "INSERT INTO read_offer_images (offer_id, image_url, position) VALUES ($1,$2,$3)",
                    )
                    .bind::<SqlUuid, _>(id)
                    .bind::<Text, _>(image_url)
                    .bind::<diesel::sql_types::BigInt, _>(position as i64)
                    .execute(connection)?;
                }
                Ok(())
            })
            .map_err(err_insert)?;
        Ok(())
    }
    fn update_title(
        &self,
        id: &super::RetailerOfferId,
        title: &str,
    ) -> Result<(), RetailerOfferRepositoryError> {
        let id = uuid::Uuid::parse_str(id.value()).map_err(super::err_update)?;
        let mut connection = self.pool.get().map_err(super::err_update)?;
        let count = diesel::sql_query("UPDATE read_offers SET title = $2 WHERE id = $1")
            .bind::<SqlUuid, _>(id)
            .bind::<Text, _>(title)
            .execute(&mut connection)
            .map_err(super::err_update)?;
        require_updated_offer(count)
    }

    fn update_price(
        &self,
        id: &super::RetailerOfferId,
        price: &money::Money,
    ) -> Result<(), RetailerOfferRepositoryError> {
        let id = uuid::Uuid::parse_str(id.value()).map_err(super::err_update)?;
        let mut connection = self.pool.get().map_err(super::err_update)?;
        let count = diesel::sql_query(
            "UPDATE read_offers SET price_amount = $2, currency = $3 WHERE id = $1",
        )
        .bind::<SqlUuid, _>(id)
        .bind::<diesel::sql_types::BigInt, _>(price.amount())
        .bind::<Text, _>(price.currency().as_str())
        .execute(&mut connection)
        .map_err(super::err_update)?;
        require_updated_offer(count)
    }
}

fn require_updated_offer(count: usize) -> Result<(), RetailerOfferRepositoryError> {
    if count == 0 {
        Err(super::err_offer_missing())
    } else {
        Ok(())
    }
}
