//! Technical plumbing shared by every feature: the database, outbound HTTP and
//! the command and event bus. Nothing here knows about the domain.

pub mod auth;
pub mod bus;
pub mod bus_api;
pub mod db;
pub mod http;
pub mod logging;
pub mod offers_publisher;
pub mod outbox;
pub mod state;
use db::DbError;
use db::DbPool;
use db::connect as db_connect;
use http::Client;
use http::OfferClient;
use object_storage::ObjectStorage;

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
    pub db: DbPool,
    pub http: Client,
    pub offer_http: OfferClient,
    pub object_storage: ObjectStorage,
}

impl Platform {
    /// Connects to Postgres and applies pending migrations. Blocks, so call it
    /// off the async runtime.
    pub fn connect(
        database_url: &str,
        object_storage: ObjectStorage,
        offer_http: OfferClient,
    ) -> Result<Self, DbError> {
        Ok(Self {
            db: db_connect(database_url)?,
            http: Client,
            offer_http,
            object_storage,
        })
    }
}
