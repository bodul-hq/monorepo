//! Read-only reports using the same contracts as the other dashboard services.

use chrono::DateTime;
use chrono::Utc;
use diesel::QueryResult;
use diesel::sql_query;
use store::load as store_load;
use store::load_entry_statuses_at;
use tokio::task::spawn_blocking;
mod api;
mod store;
#[cfg(test)]
pub(crate) mod tests;

use std::{sync::Arc, time::Duration};

use diesel::{PgConnection, RunQueryDsl};
use kernel::prelude::DbPool;

pub use api::{DEFAULT_REPORTING_TOKEN, ReportingApi};
pub use store::{DatabaseTables, EntryStatuses};

#[derive(Clone, Copy, Debug)]
pub enum ReportError {
    Unavailable,
    Query,
}

pub trait Repository: Send + Sync {
    fn database_tables(&self) -> Result<DatabaseTables, ReportError>;
    fn entry_statuses(&self) -> Result<EntryStatuses, ReportError>;
}

struct PostgresRepository {
    pool: DbPool,
    clock: fn() -> DateTime<Utc>,
}

impl PostgresRepository {
    fn read<T>(
        &self,
        load: impl FnOnce(&mut PgConnection) -> QueryResult<T>,
    ) -> Result<T, ReportError> {
        let mut connection = self
            .pool
            .get_timeout(Duration::from_secs(3))
            .map_err(|_| ReportError::Unavailable)?;
        load(&mut connection).map_err(|_| {
            if sql_query("SELECT 1").execute(&mut connection).is_err() {
                ReportError::Unavailable
            } else {
                ReportError::Query
            }
        })
    }
}

impl Repository for PostgresRepository {
    fn database_tables(&self) -> Result<DatabaseTables, ReportError> {
        self.read(store_load)
    }

    fn entry_statuses(&self) -> Result<EntryStatuses, ReportError> {
        self.read(|connection| load_entry_statuses_at(connection, (self.clock)()))
    }
}

#[derive(Clone)]
pub struct Reporting(Arc<dyn Repository>);

impl Reporting {
    pub fn new(repository: Arc<dyn Repository>) -> Self {
        Self(repository)
    }

    pub fn postgres(pool: DbPool) -> Self {
        Self::postgres_with_clock(pool, Utc::now)
    }

    pub(crate) fn postgres_with_clock(pool: DbPool, clock: fn() -> DateTime<Utc>) -> Self {
        Self::new(Arc::new(PostgresRepository { pool, clock }))
    }

    async fn run<T: Send + 'static>(
        &self,
        load: impl FnOnce(&dyn Repository) -> Result<T, ReportError> + Send + 'static,
    ) -> Result<T, ReportError> {
        let repository = self.0.clone();
        spawn_blocking(move || load(repository.as_ref()))
            .await
            .map_err(|_| ReportError::Query)?
    }

    pub async fn database_tables(&self) -> Result<DatabaseTables, ReportError> {
        self.run(|repository| repository.database_tables()).await
    }

    pub async fn entry_statuses(&self) -> Result<EntryStatuses, ReportError> {
        self.run(|repository| repository.entry_statuses()).await
    }
}
