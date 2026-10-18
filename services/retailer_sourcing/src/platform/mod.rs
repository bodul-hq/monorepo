//! Technical plumbing shared by every feature: the database, outbound HTTP and
//! the command and event bus. Nothing here knows about the domain.

pub mod auth;
pub mod bus;
pub mod bus_api;
pub mod db;
pub mod http;
pub mod logging;
pub mod state;
pub use retailer_sourcing::object_storage;
#[cfg(test)]
mod postgres_tests;
#[cfg(test)]
pub(crate) mod test_db;
#[cfg(test)]
#[path = "../../tests/support/mod.rs"]
pub(crate) mod test_objects;

/// The raw capabilities features are built from.
pub struct Platform {
    pub db: db::DbPool,
    pub http: http::Client,
    pub object_storage: object_storage::ObjectStorage,
}

impl Platform {
    /// Connects to Postgres and applies pending migrations. Blocks, so call it
    /// off the async runtime.
    pub fn connect(
        database_url: &str,
        object_storage: object_storage::ObjectStorage,
    ) -> Result<Self, db::DbError> {
        Ok(Self {
            db: db::connect(database_url)?,
            http: http::Client,
            object_storage,
        })
    }
}
