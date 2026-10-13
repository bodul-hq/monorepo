use chrono::{DateTime, Utc};
use diesel::Insertable;
use diesel::prelude::*;
use diesel::r2d2::PoolError;
use uuid::Uuid;

use super::super::schema::raw_offers;
use super::handler::Store;
use crate::platform::db::DbPool;

#[derive(Debug, thiserror::Error)]
pub enum StoreError {
    #[error("connection pool error: {0}")]
    Connection(#[from] PoolError),

    #[error("database error: {0}")]
    Database(#[from] diesel::result::Error),

    #[error("offer page body is too large for storage: {0} bytes")]
    BodyTooLarge(usize),
}

/// Postgres storage for this slice.
pub struct PgStore {
    pool: DbPool,
}

impl PgStore {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
}

#[derive(Insertable)]
#[diesel(table_name = raw_offers)]
struct NewRawOffer<'a> {
    id: Uuid,
    offer_id: Uuid,
    url: &'a str,
    body: &'a str,
    body_size: i32,
}

impl Store for PgStore {
    fn recent_raw_offer(
        &self,
        url: &str,
        since: DateTime<Utc>,
    ) -> Result<Option<Uuid>, StoreError> {
        let mut connection = self.pool.get()?;
        let raw_offer_id = raw_offers::table
            .filter(raw_offers::url.eq(url))
            .filter(raw_offers::fetched_at.ge(since))
            .order(raw_offers::fetched_at.desc())
            .select(raw_offers::id)
            .first::<Uuid>(&mut connection)
            .optional()?;

        Ok(raw_offer_id)
    }

    fn store_raw_offer(&self, offer_id: Uuid, url: &str, body: &str) -> Result<(), StoreError> {
        let body_size =
            i32::try_from(body.len()).map_err(|_| StoreError::BodyTooLarge(body.len()))?;
        let mut connection = self.pool.get()?;

        diesel::insert_into(raw_offers::table)
            .values(NewRawOffer {
                id: Uuid::now_v7(),
                offer_id,
                url,
                body,
                body_size,
            })
            .on_conflict(raw_offers::offer_id)
            .do_update()
            .set((
                raw_offers::body.eq(body),
                raw_offers::body_size.eq(body_size),
                raw_offers::fetched_at.eq(diesel::dsl::now),
            ))
            .execute(&mut connection)?;

        Ok(())
    }
}
