//! Read-only reports using the same contracts as the other dashboard services.
mod api;
mod store;
#[cfg(test)]
pub(crate) mod tests;

use std::{sync::Arc, time::Duration};

use diesel::{PgConnection, RunQueryDsl};
use kernel::io::DbPool;

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

struct PostgresRepository(DbPool);

impl PostgresRepository {
    fn read<T>(
        &self,
        load: fn(&mut PgConnection) -> diesel::QueryResult<T>,
    ) -> Result<T, ReportError> {
        let mut connection = self
            .0
            .get_timeout(Duration::from_secs(3))
            .map_err(|_| ReportError::Unavailable)?;
        load(&mut connection).map_err(|_| {
            if diesel::sql_query("SELECT 1")
                .execute(&mut connection)
                .is_err()
            {
                ReportError::Unavailable
            } else {
                ReportError::Query
            }
        })
    }
}

impl Repository for PostgresRepository {
    fn database_tables(&self) -> Result<DatabaseTables, ReportError> {
        self.read(store::load)
    }

    fn entry_statuses(&self) -> Result<EntryStatuses, ReportError> {
        self.read(store::load_entry_statuses)
    }
}

#[derive(Clone)]
pub struct Reporting(Arc<dyn Repository>);

impl Reporting {
    pub fn new(repository: Arc<dyn Repository>) -> Self {
        Self(repository)
    }

    pub fn postgres(pool: DbPool) -> Self {
        Self::new(Arc::new(PostgresRepository(pool)))
    }

    async fn run<T: Send + 'static>(
        &self,
        load: impl FnOnce(&dyn Repository) -> Result<T, ReportError> + Send + 'static,
    ) -> Result<T, ReportError> {
        let repository = self.0.clone();
        tokio::task::spawn_blocking(move || load(repository.as_ref()))
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
