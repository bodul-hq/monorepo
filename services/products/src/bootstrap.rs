use io::Error as IoError;
use io::Result as IoResult;
use std::env::var;
use std::{env::VarError, io, time::Duration};

use diesel::{PgConnection, r2d2::ConnectionManager};
use diesel_migrations::{EmbeddedMigrations, MigrationHarness, embed_migrations};
use testimony_infra::prelude::DbPool;

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

pub fn database_pool() -> IoResult<DbPool> {
    let url = database_url(var("DATABASE_URL"))?;
    connect(&url)
}

fn database_url(value: Result<String, VarError>) -> IoResult<String> {
    match value {
        Ok(url) if !url.trim().is_empty() => Ok(url),
        Ok(_) => Err(IoError::other("DATABASE_URL cannot be empty")),
        Err(VarError::NotPresent) => Err(IoError::other("DATABASE_URL is required")),
        Err(_) => Err(IoError::other("DATABASE_URL must be valid Unicode")),
    }
}

fn connect(url: &str) -> IoResult<DbPool> {
    let pool = DbPool::builder()
        .connection_timeout(Duration::from_secs(5))
        .build(ConnectionManager::<PgConnection>::new(url))
        .map_err(|_| IoError::other("could not connect to the products database"))?;
    let mut connection = pool
        .get()
        .map_err(|_| IoError::other("could not acquire a database connection"))?;
    connection
        .run_pending_migrations(MIGRATIONS)
        .map_err(|_| IoError::other("products database migrations failed"))?;
    drop(connection);
    Ok(pool)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn database_configuration_is_required() {
        assert!(database_url(Err(VarError::NotPresent)).is_err());
        for value in ["", " ", "\n"] {
            assert!(database_url(Ok(value.into())).is_err());
        }
        assert_eq!(
            database_url(Ok("postgres://localhost/db".into())).unwrap(),
            "postgres://localhost/db"
        );
    }
}
