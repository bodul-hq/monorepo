use crate::read_offers_identifiers::entity::RetailerOfferIdentifiers;
use bodul_shared::retailer::RetailerCode;
use diesel::{
    OptionalExtension, RunQueryDsl,
    sql_types::{Text, Uuid as SqlUuid},
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
        let id = Uuid::parse_str(id.value())
            .map_err(|error| RetailerOfferIdentifiersError(error.to_string()))?;
        let mut connection = self
            .pool
            .get()
            .map_err(|error| RetailerOfferIdentifiersError(error.to_string()))?;
        diesel::sql_query(
            "INSERT INTO read_offer_identities (id, retailer_code, url) VALUES ($1, $2, $3)",
        )
        .bind::<SqlUuid, _>(id)
        .bind::<Text, _>(retailer_code.as_str())
        .bind::<Text, _>(url)
        .execute(&mut connection)
        .map_err(|error| RetailerOfferIdentifiersError(error.to_string()))?;
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
        }

        let mut connection = self
            .pool
            .get()
            .map_err(|error| RetailerOfferIdentifiersError(error.to_string()))?;
        let row = diesel::sql_query(
            "SELECT id FROM read_offer_identities WHERE retailer_code = $1 AND url = $2",
        )
        .bind::<Text, _>(retailer_code.as_str())
        .bind::<Text, _>(url)
        .get_result::<IdentityRow>(&mut connection)
        .optional()
        .map_err(|error| RetailerOfferIdentifiersError(error.to_string()))?;

        Ok(row.map(|row| RetailerOfferIdentifiers {
            id: RetailerOfferId::new(row.id.to_string()),
            retailer_code,
            url: url.to_owned(),
        }))
    }
}
