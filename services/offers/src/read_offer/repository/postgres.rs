use super::{
    RetailerOffer, RetailerOfferRepository, RetailerOfferRepositoryError, err_insert, validate,
};
use diesel::{
    RunQueryDsl,
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
    fn insert(&self, entity: &RetailerOffer) -> Result<(), RetailerOfferRepositoryError> {
        let id = validate(entity)?;
        let mut connection = self.pool.get().map_err(err_insert)?;
        diesel::sql_query("INSERT INTO read_offers (id, retailer_code, url, title, price_amount, currency, created_at) VALUES ($1, $2, $3, $4, $5, $6, $7)")
        .bind::<SqlUuid, _>(id)
        .bind::<Text, _>(entity.retailer_code.as_str())
        .bind::<Text, _>(&entity.url)
        .bind::<Text, _>(&entity.title)
        .bind::<diesel::sql_types::BigInt, _>(entity.price.amount())
        .bind::<Text, _>(entity.price.currency().as_str())
        .bind::<diesel::sql_types::Timestamptz, _>(entity.created_at)
        .execute(&mut connection)
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
