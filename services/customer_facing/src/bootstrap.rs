//! Database configuration and service-owned queue migrations.

use std::{env::VarError, io, time::Duration};

use diesel::{PgConnection, r2d2::ConnectionManager};
use diesel_migrations::{EmbeddedMigrations, MigrationHarness, embed_migrations};
use kernel::io::DbPool;

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

pub fn database_pool() -> io::Result<DbPool> {
    connect(&database_url(std::env::var("DATABASE_URL"))?)
}

fn database_url(value: Result<String, VarError>) -> io::Result<String> {
    match value {
        Ok(value) if value.trim().is_empty() => Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "DATABASE_URL cannot be empty",
        )),
        Ok(value) => Ok(value),
        Err(VarError::NotPresent) => Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "DATABASE_URL is required",
        )),
        Err(VarError::NotUnicode(_)) => Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "DATABASE_URL must be valid Unicode",
        )),
    }
}

fn connect(url: &str) -> io::Result<DbPool> {
    let pool = DbPool::builder()
        .connection_timeout(Duration::from_secs(5))
        .build(ConnectionManager::<PgConnection>::new(url))
        .map_err(|_| io::Error::other("could not connect to the customer facing database"))?;
    migrate(&pool)?;
    Ok(pool)
}

pub fn migrate(pool: &DbPool) -> io::Result<()> {
    let mut connection = pool
        .get()
        .map_err(|_| io::Error::other("could not acquire a database connection"))?;
    connection
        .run_pending_migrations(MIGRATIONS)
        .map_err(|_| io::Error::other("customer facing database migrations failed"))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn database_configuration_is_required_and_preserved() {
        assert_eq!(
            database_url(Err(VarError::NotPresent))
                .unwrap_err()
                .to_string(),
            "DATABASE_URL is required"
        );
        for value in ["", " ", "\t\n"] {
            assert_eq!(
                database_url(Ok(value.into())).unwrap_err().to_string(),
                "DATABASE_URL cannot be empty"
            );
        }
        let url = "postgres://user:secret@localhost/customer_facing";
        assert_eq!(database_url(Ok(url.into())).unwrap(), url);
    }

    #[cfg(unix)]
    #[test]
    fn database_configuration_rejects_non_unicode() {
        use std::{ffi::OsString, os::unix::ffi::OsStringExt};
        let value = VarError::NotUnicode(OsString::from_vec(vec![0xff]));
        assert_eq!(
            database_url(Err(value)).unwrap_err().to_string(),
            "DATABASE_URL must be valid Unicode"
        );
    }

    #[test]
    fn inaccessible_database_errors_do_not_expose_the_url() {
        for url in [
            "not-a-database-url secret",
            "host=/dev/null/customer-facing-test dbname=test password=secret connect_timeout=1",
        ] {
            let error = connect(url).unwrap_err();
            assert_eq!(
                error.to_string(),
                "could not connect to the customer facing database"
            );
        }
    }
}
