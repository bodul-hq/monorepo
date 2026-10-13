//! Postgres connection pool and schema migrations.

use diesel::PgConnection;
use diesel::r2d2::{ConnectionManager, Pool, PoolError};
use diesel_migrations::{EmbeddedMigrations, MigrationHarness, embed_migrations};

pub type DbPool = Pool<ConnectionManager<PgConnection>>;

const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

#[derive(Debug, thiserror::Error)]
pub enum DbError {
    #[error("database connection failed: {0}")]
    Pool(#[from] PoolError),

    #[error("database migration failed: {0}")]
    Migration(String),
}

/// Connects to `database_url` and applies any pending migrations.
///
/// Blocks while the pool opens its connections, so call it off the async
/// runtime.
pub fn connect(database_url: &str) -> Result<DbPool, DbError> {
    let pool = Pool::builder().build(ConnectionManager::<PgConnection>::new(database_url))?;

    pool.get()?
        .run_pending_migrations(MIGRATIONS)
        .map_err(|error| DbError::Migration(error.to_string()))?;

    Ok(pool)
}
