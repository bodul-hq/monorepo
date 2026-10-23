use super::{
    RetailerOfferIdentifiers, RetailerOfferIdentifiersRepository,
    RetailerOfferIdentifiersRepositoryError, err_insert, validate,
};
use diesel::{
    RunQueryDsl,
    sql_types::{Text, Uuid as SqlUuid},
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
            "INSERT INTO read_offer_identities (id, retailer_code, url) VALUES ($1, $2, $3)",
        )
        .bind::<SqlUuid, _>(id)
        .bind::<Text, _>(entity.retailer_code.as_str())
        .bind::<Text, _>(&entity.url)
        .execute(&mut connection)
        .map_err(err_insert)?;
        Ok(())
    }
}
