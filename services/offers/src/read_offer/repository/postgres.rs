use super::RetailerOfferId as SuperRetailerOfferId;
use super::err_offer_missing;
use super::err_update as update_error;
use super::{
    RetailerOffer, RetailerOfferRepository, RetailerOfferRepositoryError, err_insert, validate,
};
use crate::offers::event::OfferImageAddedEvent;
use crate::offers::event::Pricing;
use crate::offers::retailer_offer_id::RetailerOfferId as RetailerOfferIdRetailerOfferId;
use chrono::DateTime;
use chrono::Utc;
use diesel::PgConnection;
use diesel::QueryableByName;
use diesel::result::Error as ResultError;
use diesel::sql_query;
use diesel::sql_types::BigInt;
use diesel::sql_types::Jsonb;
use diesel::sql_types::Nullable;
use diesel::sql_types::Timestamptz;
use diesel::{
    Connection, RunQueryDsl,
    sql_types::{Text, Uuid as SqlUuid},
};
use money::Money;
use offer_content::Content as OfferContentContent;
use offer_content::guard as offer_content_guard;
use offer_content::legacy_description;
use offer_content::legacy_language;
use offer_content::legacy_title;
use offer_content::selected as offer_content_selected;
use offer_content::sort;
use retailer_sauce::availability::Availability;
use retailer_sauce::identifier::ProductIdentifiers;
use retailer_sauce::item_condition::ItemCondition;
use retailer_sauce::localization::Localization;
use retailer_sauce::offer_content;
use retailer_sauce::structuralized::content::Content as ContentContent;
use serde_json::Value;
use serde_json::from_value;
use serde_json::to_value as encode_json_value;
use shared::images::is_valid;
use testimony_infra::prelude::DbPool;
use uuid::Uuid;

pub struct PostgresRetailerOfferRepository {
    pool: DbPool,
}

impl PostgresRetailerOfferRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
}

impl RetailerOfferRepository for PostgresRetailerOfferRepository {
    fn update_contents(
        &self,
        id: &SuperRetailerOfferId,
        content: &[OfferContentContent],
        language: &Option<String>,
    ) -> Result<(), RetailerOfferRepositoryError> {
        offer_content_guard(content, language).map_err(update_error)?;
        let id = Uuid::parse_str(id.value()).map_err(update_error)?;
        let mut connection = self.pool.get().map_err(update_error)?;
        write_contents(&mut connection, id, content, language)
    }
    fn update_content(
        &self,
        id: &SuperRetailerOfferId,
        content: &ContentContent,
    ) -> Result<(), RetailerOfferRepositoryError> {
        content.guard().map_err(update_error)?;
        self.mutate_contents(
            id,
            |list, active| legacy_description(list, active, content.description.clone()),
            None,
        )
    }

    fn update_item_condition(
        &self,
        id: &SuperRetailerOfferId,
        item_condition: &Option<ItemCondition>,
    ) -> Result<(), RetailerOfferRepositoryError> {
        if let Some(value) = item_condition {
            value.guard().map_err(update_error)?;
        }
        let id = Uuid::parse_str(id.value()).map_err(update_error)?;
        let count = sql_query("UPDATE read_offers SET item_condition=$2 WHERE id=$1")
            .bind::<SqlUuid, _>(id)
            .bind::<Nullable<Jsonb>, _>(
                item_condition
                    .as_ref()
                    .map(encode_json_value)
                    .transpose()
                    .map_err(update_error)?,
            )
            .execute(&mut self.pool.get().map_err(update_error)?)
            .map_err(update_error)?;
        require_updated_offer(count)
    }
    fn update_availability(
        &self,
        id: &SuperRetailerOfferId,
        availability: &Option<Availability>,
    ) -> Result<(), RetailerOfferRepositoryError> {
        if let Some(value) = availability {
            value.guard().map_err(update_error)?;
        }
        let id = Uuid::parse_str(id.value()).map_err(update_error)?;
        let count = sql_query("UPDATE read_offers SET availability=$2 WHERE id=$1")
            .bind::<SqlUuid, _>(id)
            .bind::<Nullable<Jsonb>, _>(
                availability
                    .as_ref()
                    .map(encode_json_value)
                    .transpose()
                    .map_err(update_error)?,
            )
            .execute(&mut self.pool.get().map_err(update_error)?)
            .map_err(update_error)?;
        require_updated_offer(count)
    }
    fn update_pricing(
        &self,
        id: &SuperRetailerOfferId,
        pricing: &Pricing,
    ) -> Result<(), RetailerOfferRepositoryError> {
        pricing.guard().map_err(update_error)?;
        let id = Uuid::parse_str(id.value()).map_err(update_error)?;
        let count = sql_query("UPDATE read_offers SET price_amount=$2,currency=$3,was_price_amount=$4,was_price_currency=$5 WHERE id=$1").bind::<SqlUuid,_>(id)
        .bind::<BigInt,_>(pricing.price.amount())
        .bind::<Text,_>(pricing.price.currency().as_str())
        .bind::<Nullable<BigInt>,_>(pricing.was_price.as_ref().map(|v|v.amount()))
        .bind::<Nullable<Text>,_>(pricing.was_price.as_ref().map(|v|v.currency().as_str()))
        .execute(&mut self.pool.get().map_err(update_error)?).map_err(update_error)?;
        require_updated_offer(count)
    }
    fn update_identifiers(
        &self,
        id: &RetailerOfferIdRetailerOfferId,
        identifiers: &ProductIdentifiers,
    ) -> Result<(), RetailerOfferRepositoryError> {
        identifiers.guard().map_err(update_error)?;
        let id = Uuid::parse_str(id.value()).map_err(update_error)?;
        let count = sql_query("UPDATE read_offers SET gtin=$2, sku=$3, shopify_product_gid=$4, shopify_product_variation_gid=$5, shopify_handle=$6 WHERE id=$1").bind::<SqlUuid, _>(id)
        .bind::<Nullable<Text>, _>(identifiers.gtin.as_ref().map(|value| &value.value))
            .bind::<Nullable<Text>, _>(identifiers.sku.as_ref().map(|value| &value.value))
            .bind::<Nullable<Text>, _>(identifiers.external_identifiers.shopify_product_gid.as_ref().map(|value| &value.value))
            .bind::<Nullable<Text>, _>(identifiers.external_identifiers.shopify_product_variation_gid.as_ref().map(|value| &value.value))
            .bind::<Nullable<Text>, _>(identifiers.external_identifiers.shopify_handle.as_ref().map(|value| &value.value))
            .execute(&mut self.pool.get().map_err(update_error)?).map_err(update_error)?;
        require_updated_offer(count)
    }
    fn update_localization(
        &self,
        id: &RetailerOfferIdRetailerOfferId,
        localization: &Localization,
    ) -> Result<(), RetailerOfferRepositoryError> {
        localization.guard().map_err(update_error)?;
        self.mutate_contents(
            id,
            |list, active| legacy_language(list, active, localization.language.clone()),
            Some(localization),
        )
    }

    fn update_scraped_at(
        &self,
        id: &SuperRetailerOfferId,
        scraped_at: DateTime<Utc>,
    ) -> Result<(), RetailerOfferRepositoryError> {
        let id = Uuid::parse_str(id.value()).map_err(update_error)?;
        let count =
            sql_query("UPDATE read_offers SET scraped_at = GREATEST(scraped_at, $2) WHERE id = $1")
                .bind::<SqlUuid, _>(id)
                .bind::<Timestamptz, _>(scraped_at)
                .execute(&mut self.pool.get().map_err(update_error)?)
                .map_err(update_error)?;
        require_updated_offer(count)
    }

    fn images(
        &self,
        id: &SuperRetailerOfferId,
    ) -> Result<Vec<String>, RetailerOfferRepositoryError> {
        #[derive(QueryableByName)]
        struct Image {
            #[diesel(sql_type = Text)]
            image_url: String,
        }
        let id = Uuid::parse_str(id.value()).map_err(update_error)?;
        let rows = sql_query(
            "SELECT image_url FROM read_offer_images WHERE offer_id = $1 ORDER BY position",
        )
        .bind::<SqlUuid, _>(id)
        .load::<Image>(&mut self.pool.get().map_err(update_error)?)
        .map_err(update_error)?;
        Ok(rows.into_iter().map(|row| row.image_url).collect())
    }
    fn add_image(&self, event: &OfferImageAddedEvent) -> Result<(), RetailerOfferRepositoryError> {
        if !is_valid(&event.image_url) {
            return Err(update_error("invalid image URL"));
        }
        let id = Uuid::parse_str(event.id.value()).map_err(update_error)?;
        let count = sql_query(
            "INSERT INTO read_offer_images (offer_id, image_url, position) VALUES ($1,$2,$3)
            ON CONFLICT (offer_id, position) DO UPDATE SET image_url = EXCLUDED.image_url
            WHERE read_offer_images.image_url = EXCLUDED.image_url",
        )
        .bind::<SqlUuid, _>(id)
        .bind::<Text, _>(&event.image_url)
        .bind::<BigInt, _>(i64::from(event.position))
        .execute(&mut self.pool.get().map_err(update_error)?)
        .map_err(update_error)?;
        if count == 0 {
            return Err(update_error("conflicting image addition"));
        }
        Ok(())
    }

    fn insert(&self, entity: &RetailerOffer) -> Result<(), RetailerOfferRepositoryError> {
        let id = validate(entity)?;
        let market = entity
            .localization
            .target_market
            .as_ref()
            .map(|market| encode_json_value(market.canonicalized()))
            .transpose()
            .map_err(err_insert)?;
        let availability = entity
            .facts
            .availability
            .as_ref()
            .map(encode_json_value)
            .transpose()
            .map_err(err_insert)?;
        let item_condition = entity
            .facts
            .item_condition
            .as_ref()
            .map(encode_json_value)
            .transpose()
            .map_err(err_insert)?;
        let mut content = entity.facts.content.clone();
        sort(&mut content);
        let description = &offer_content_selected(&content, &entity.localization.language)
            .map_err(err_insert)?
            .description;
        let content_json = encode_json_value(&content).map_err(err_insert)?;
        let mut connection = self.pool.get().map_err(err_insert)?;
        connection
            .transaction::<_, ResultError, _>(|connection| {
                sql_query(
                    "INSERT INTO read_offers (id, retailer_code, url, title, price_amount, currency, created_at, scraped_at, target_market, language, gtin, sku, shopify_product_gid, shopify_product_variation_gid, shopify_handle, was_price_amount, was_price_currency, availability, item_condition, description, content)
                     VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21)",
                )
                .bind::<SqlUuid, _>(id)
                .bind::<Text, _>(entity.retailer_code.as_str())
                .bind::<Text, _>(&entity.url)
                .bind::<Text, _>(&entity.title)
                .bind::<BigInt, _>(entity.price.amount())
                .bind::<Text, _>(entity.price.currency().as_str())
                .bind::<Timestamptz, _>(entity.created_at)
                .bind::<Nullable<Timestamptz>, _>(entity.scraped_at)
                .bind::<Nullable<Jsonb>, _>(&market)
                .bind::<Nullable<Text>, _>(&entity.localization.language)
                .bind::<Nullable<Text>, _>(entity.identifiers.gtin.as_ref().map(|value| &value.value))
                .bind::<Nullable<Text>, _>(entity.identifiers.sku.as_ref().map(|value| &value.value))
                .bind::<Nullable<Text>, _>(entity.identifiers.external_identifiers.shopify_product_gid.as_ref().map(|value| &value.value))
                .bind::<Nullable<Text>, _>(entity.identifiers.external_identifiers.shopify_product_variation_gid.as_ref().map(|value| &value.value))
                .bind::<Nullable<Text>, _>(entity.identifiers.external_identifiers.shopify_handle.as_ref().map(|value| &value.value))
                .bind::<Nullable<BigInt>,_>(entity.facts.was_price.as_ref().map(|v|v.amount()))
                .bind::<Nullable<Text>,_>(entity.facts.was_price.as_ref().map(|v|v.currency().as_str()))
                .bind::<Nullable<Jsonb>,_>(&availability)
                .bind::<Nullable<Jsonb>,_>(&item_condition)
                .bind::<Nullable<Text>,_>(description)
                .bind::<Jsonb,_>(&content_json)
                .execute(connection)?;
                for (position, image_url) in entity.images.iter().enumerate() {
                    sql_query(
                        "INSERT INTO read_offer_images (offer_id, image_url, position) VALUES ($1,$2,$3)",
                    )
                    .bind::<SqlUuid, _>(id)
                    .bind::<Text, _>(image_url)
                    .bind::<BigInt, _>(position as i64)
                    .execute(connection)?;
                }
                Ok(())
            })
            .map_err(err_insert)?;
        Ok(())
    }
    fn update_title(
        &self,
        id: &SuperRetailerOfferId,
        title: &str,
    ) -> Result<(), RetailerOfferRepositoryError> {
        self.mutate_contents(
            id,
            |list, active| legacy_title(list, active, title.to_owned()),
            None,
        )
    }

    fn update_price(
        &self,
        id: &SuperRetailerOfferId,
        price: &Money,
    ) -> Result<(), RetailerOfferRepositoryError> {
        let id = Uuid::parse_str(id.value()).map_err(update_error)?;
        let mut connection = self.pool.get().map_err(update_error)?;
        let count =
            sql_query("UPDATE read_offers SET price_amount = $2, currency = $3 WHERE id = $1")
                .bind::<SqlUuid, _>(id)
                .bind::<BigInt, _>(price.amount())
                .bind::<Text, _>(price.currency().as_str())
                .execute(&mut connection)
                .map_err(update_error)?;
        require_updated_offer(count)
    }
}

fn require_updated_offer(count: usize) -> Result<(), RetailerOfferRepositoryError> {
    if count == 0 {
        Err(err_offer_missing())
    } else {
        Ok(())
    }
}

#[derive(QueryableByName)]
struct ContentRow {
    #[diesel(sql_type = Jsonb)]
    content: Value,
    #[diesel(sql_type = Nullable<Text>)]
    language: Option<String>,
}

impl PostgresRetailerOfferRepository {
    /// Legacy scalar events must reconcile the list while holding the row lock.
    fn mutate_contents(
        &self,
        id: &SuperRetailerOfferId,
        update: impl FnOnce(
            &[OfferContentContent],
            &Option<String>,
        ) -> Result<Vec<OfferContentContent>, String>,
        localization: Option<&Localization>,
    ) -> Result<(), RetailerOfferRepositoryError> {
        let id = Uuid::parse_str(id.value()).map_err(update_error)?;
        let mut connection = self.pool.get().map_err(update_error)?;
        connection.transaction::<_, RetailerOfferRepositoryError, _>(|connection| {
            let row = sql_query("SELECT content, language FROM read_offers WHERE id=$1 FOR UPDATE")
                .bind::<SqlUuid, _>(id)
                .get_result::<ContentRow>(connection)
                .map_err(update_error)?;
            let content: Vec<OfferContentContent> =
                from_value(row.content).map_err(update_error)?;
            let content = update(&content, &row.language).map_err(update_error)?;
            let language = localization
                .map(|value| &value.language)
                .unwrap_or(&row.language);
            write_contents(connection, id, &content, language)?;
            if let Some(localization) = localization {
                let market = localization
                    .target_market
                    .as_ref()
                    .map(|market| encode_json_value(market.canonicalized()))
                    .transpose()
                    .map_err(update_error)?;
                sql_query("UPDATE read_offers SET target_market=$2 WHERE id=$1")
                    .bind::<SqlUuid, _>(id)
                    .bind::<Nullable<Jsonb>, _>(market)
                    .execute(connection)
                    .map_err(update_error)?;
            }
            Ok(())
        })
    }
}
impl From<ResultError> for RetailerOfferRepositoryError {
    fn from(error: ResultError) -> Self {
        update_error(error)
    }
}
fn write_contents(
    connection: &mut PgConnection,
    id: Uuid,
    content: &[OfferContentContent],
    language: &Option<String>,
) -> Result<(), RetailerOfferRepositoryError> {
    offer_content_guard(content, language).map_err(update_error)?;
    let mut content = content.to_vec();
    sort(&mut content);
    let selected = offer_content_selected(&content, language).map_err(update_error)?;
    let json = encode_json_value(&content).map_err(update_error)?;
    let count = sql_query(
        "UPDATE read_offers SET content=$2, language=$3, title=$4, description=$5 WHERE id=$1",
    )
    .bind::<SqlUuid, _>(id)
    .bind::<Jsonb, _>(json)
    .bind::<Nullable<Text>, _>(language)
    .bind::<Text, _>(&selected.title)
    .bind::<Nullable<Text>, _>(&selected.description)
    .execute(connection)
    .map_err(update_error)?;
    require_updated_offer(count)
}
