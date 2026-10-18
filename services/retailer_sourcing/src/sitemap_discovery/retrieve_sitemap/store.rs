use chrono::{DateTime, Utc};
use diesel::Insertable;
use diesel::prelude::*;
use diesel::r2d2::PoolError;
use uuid::Uuid;

use super::super::schema::raw_sitemap_documents;
use super::handler::Store;
use crate::platform::db::DbPool;
use sitemap::RawSitemapDocument;

#[derive(Debug, thiserror::Error)]
pub enum StoreError {
    #[error("connection pool error: {0}")]
    Connection(#[from] PoolError),

    #[error("database error: {0}")]
    Database(#[from] diesel::result::Error),

    #[error("raw sitemap body is too large for storage: {0} bytes")]
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
#[diesel(table_name = raw_sitemap_documents)]
struct NewRawSitemapDocument<'a> {
    id: Uuid,
    retrieval_id: Uuid,
    url: &'a str,
    last_modified: Option<DateTime<Utc>>,
    body: &'a str,
    body_size: i32,
    fetched_at: DateTime<Utc>,
}

impl Store for PgStore {
    fn store_documents(
        &self,
        retrieval_id: Uuid,
        documents: &[RawSitemapDocument],
    ) -> Result<(), StoreError> {
        let fetched_at = Utc::now();
        let records = documents
            .iter()
            .map(|document| {
                let body_size = i32::try_from(document.body_size)
                    .map_err(|_| StoreError::BodyTooLarge(document.body_size))?;

                Ok(NewRawSitemapDocument {
                    id: Uuid::now_v7(),
                    retrieval_id,
                    url: &document.url,
                    last_modified: document.last_modified,
                    body: &document.body,
                    body_size,
                    fetched_at,
                })
            })
            .collect::<Result<Vec<_>, StoreError>>()?;

        let mut connection = self.pool.get()?;
        diesel::insert_into(raw_sitemap_documents::table)
            .values(&records)
            .execute(&mut connection)?;

        Ok(())
    }
}
