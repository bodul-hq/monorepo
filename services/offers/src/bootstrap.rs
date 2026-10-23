use std::{env::VarError, io, time::Duration};

use diesel::{PgConnection, r2d2::ConnectionManager};
use diesel_migrations::{EmbeddedMigrations, MigrationHarness, embed_migrations};
use testimony_infra::prelude::DbPool;

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

pub fn database_pool() -> io::Result<Option<DbPool>> {
    let url = match std::env::var("DATABASE_URL") {
        Ok(url) if !url.trim().is_empty() => url,
        Ok(_) => return Err(io::Error::other("DATABASE_URL cannot be empty")),
        Err(VarError::NotPresent) => return Ok(None),
        Err(_) => return Err(io::Error::other("DATABASE_URL must be valid Unicode")),
    };
    let pool = DbPool::builder()
        .connection_timeout(Duration::from_secs(5))
        .build(ConnectionManager::<PgConnection>::new(url))
        .map_err(|_| io::Error::other("could not connect to the offers database"))?;
    let mut connection = pool
        .get()
        .map_err(|_| io::Error::other("could not acquire a database connection"))?;
    connection
        .run_pending_migrations(MIGRATIONS)
        .map_err(|_| io::Error::other("offers database migrations failed"))?;
    drop(connection);
    Ok(Some(pool))
}
