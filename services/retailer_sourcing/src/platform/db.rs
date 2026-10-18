//! Postgres connection pool and schema migrations.

pub(crate) use retailer_sourcing::json::Json;
#[cfg(test)]
pub(crate) use retailer_sourcing::json::Storage;

use diesel::PgConnection;
use diesel::r2d2::{ConnectionManager, Pool, PoolError};
use diesel::result::Error;
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

/// Describes `error` with Postgres's detail, such as the key values a unique
/// constraint rejected, which diesel's `Display` leaves out.
pub fn describe(error: &Error) -> String {
    match error {
        Error::DatabaseError(_, information) => match information.details() {
            Some(details) => format!("{}: {details}", information.message()),
            None => information.message().to_string(),
        },
        error => error.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use diesel::result::{DatabaseErrorInformation, DatabaseErrorKind, Error};

    use super::describe;

    struct Information(Option<&'static str>);

    impl DatabaseErrorInformation for Information {
        fn message(&self) -> &str {
            "duplicate key value violates unique constraint \"raw_sitemap_documents_retrieval_id_url_key\""
        }
        fn details(&self) -> Option<&str> {
            self.0
        }
        fn hint(&self) -> Option<&str> {
            None
        }
        fn table_name(&self) -> Option<&str> {
            None
        }
        fn column_name(&self) -> Option<&str> {
            None
        }
        fn constraint_name(&self) -> Option<&str> {
            None
        }
        fn statement_position(&self) -> Option<i32> {
            None
        }
    }

    fn unique_violation(details: Option<&'static str>) -> Error {
        Error::DatabaseError(
            DatabaseErrorKind::UniqueViolation,
            Box::new(Information(details)),
        )
    }

    #[test]
    fn adds_the_rejected_values() {
        let details =
            "Key (retrieval_id, url)=(01a1, https://shop.test/sitemap.xml) already exists.";

        assert_eq!(
            describe(&unique_violation(Some(details))),
            format!(
                "duplicate key value violates unique constraint \"raw_sitemap_documents_retrieval_id_url_key\": {details}"
            )
        );
    }

    #[test]
    fn keeps_the_message_without_details() {
        assert_eq!(
            describe(&unique_violation(None)),
            "duplicate key value violates unique constraint \"raw_sitemap_documents_retrieval_id_url_key\""
        );
    }

    #[test]
    fn describes_other_errors_as_diesel_does() {
        assert_eq!(describe(&Error::NotFound), Error::NotFound.to_string());
    }
}
