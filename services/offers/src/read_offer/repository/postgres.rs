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
        let mut connection = self.pool.get().map_err(err_insert)?;
        connection
            .transaction::<_, diesel::result::Error, _>(|connection| {
                diesel::sql_query(
                    "INSERT INTO read_offers (id, retailer_code, url, title, price_amount, currency, created_at, scraped_at)
                     VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
                )
                .bind::<SqlUuid, _>(id)
                .bind::<Text, _>(entity.retailer_code.as_str())
                .bind::<Text, _>(&entity.url)
                .bind::<Text, _>(&entity.title)
                .bind::<diesel::sql_types::BigInt, _>(entity.price.amount())
                .bind::<Text, _>(entity.price.currency().as_str())
                .bind::<diesel::sql_types::Timestamptz, _>(entity.created_at)
                .bind::<diesel::sql_types::Nullable<diesel::sql_types::Timestamptz>, _>(entity.scraped_at)
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
