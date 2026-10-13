//! Technical plumbing shared by every feature: the database, outbound HTTP and
//! the command and event bus. Nothing here knows about the domain.

pub mod bus;
pub mod db;
pub mod http;

/// The raw capabilities features are built from.
pub struct Platform {
    pub db: db::DbPool,
    pub http: http::Client,
}

impl Platform {
    /// Connects to Postgres and applies pending migrations. Blocks, so call it
    /// off the async runtime.
    pub fn connect(database_url: &str) -> Result<Self, db::DbError> {
        Ok(Self {
            db: db::connect(database_url)?,
            http: http::Client,
        })
    }
}
