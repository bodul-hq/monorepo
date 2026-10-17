use crate::{
    error::Error,
    model::{self, Create, Created, Record},
};
use diesel::{
    prelude::*,
    r2d2::{ConnectionManager, Pool},
    result::{DatabaseErrorKind, Error as DbError},
    sql_types::{Bool, Integer, Jsonb, Text, Timestamptz, Uuid as SqlUuid},
};
use diesel_migrations::{EmbeddedMigrations, MigrationHarness, embed_migrations};
use std::time::Duration;
use uuid::Uuid;

pub type DbPool = Pool<ConnectionManager<PgConnection>>;
pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

#[derive(Clone)]
pub struct Store(pub DbPool);

impl Store {
    /// Blocking startup: require connectivity, UTF8 encoding, and migrations.
    pub fn connect(url: &str) -> Result<Self, &'static str> {
        let pool = Pool::builder()
            .connection_timeout(Duration::from_secs(3))
            .build(ConnectionManager::<PgConnection>::new(url))
            .map_err(|_| "Database connection failed.")?;
        let mut connection = pool.get().map_err(|_| "Database connection failed.")?;
        #[derive(QueryableByName)]
        struct Encoding {
            #[diesel(sql_type = Text)]
            server_encoding: String,
            #[diesel(sql_type = Integer)]
            server_version: i32,
        }
        let encoding: Encoding = diesel::sql_query("SELECT current_setting('server_encoding') AS server_encoding, current_setting('server_version_num')::integer AS server_version")
            .get_result(&mut connection)
            .map_err(|_| "Database encoding check failed.")?;
        if encoding.server_encoding != "UTF8" {
            return Err("Database must use UTF8 encoding.");
        }
        if encoding.server_version < 160_000 {
            return Err("PostgreSQL 16 or later is required.");
        }
        connection
            .run_pending_migrations(MIGRATIONS)
            .map_err(|_| "Database migrations failed.")?;
        drop(connection);
        Ok(Self(pool))
    }

    pub(crate) async fn blocking<T: Send + 'static>(
        &self,
        f: impl FnOnce(&mut PgConnection) -> Result<T, Error> + Send + 'static,
    ) -> Result<T, Error> {
        let pool = self.0.clone();
        tokio::task::spawn_blocking(move || {
            let mut connection = pool.get().map_err(|_| Error::Unavailable)?;
            let result = f(&mut connection);
            // Diesel does not expose SQLSTATE for every PostgreSQL error. If
            // an otherwise unexpected failure broke the connection, report a
            // temporary outage without inspecting or returning diagnostic text.
            if matches!(result, Err(Error::Internal))
                && diesel::sql_query("SELECT 1")
                    .execute(&mut connection)
                    .is_err()
            {
                return Err(Error::Unavailable);
            }
            result
        })
        .await
        .map_err(|_| Error::Internal)?
    }

    pub async fn create(&self, bucket: String, input: Create) -> Result<Created, Error> {
        let key = input.validate(&bucket)?;
        self.blocking(move |connection| {
            // Native soft-error validation precisely follows PostgreSQL's
            // JSONB/numeric restrictions (including zero and decimal exponents).
            // A rejected value cannot begin a write or expose diagnostics.
            #[derive(QueryableByName)]
            struct ValidJson { #[diesel(sql_type = Bool)] valid: bool }
            let valid: ValidJson = diesel::sql_query("SELECT pg_input_is_valid($1, 'jsonb') AS valid")
                .bind::<Text,_>(input.metadata.to_string()).get_result(connection).map_err(database_error)?;
            if !valid.valid { return Err(Error::Input("Metadata contains values unsupported by PostgreSQL JSONB.")); }
            let id = Uuid::now_v7();
            let data_id = Uuid::now_v7();
            let inserted = connection.transaction::<_, DbError, _>(|connection| {
                diesel::sql_query("INSERT INTO data (id, payload) VALUES ($1, $2)")
                    .bind::<SqlUuid,_>(data_id).bind::<Text,_>(&input.payload).execute(connection)?;
                diesel::sql_query("INSERT INTO records (id, bucket, external_id, data_id, metadata, created_at) VALUES ($1,$2,$3,$4,$5,$6)")
                    .bind::<SqlUuid,_>(id).bind::<Text,_>(&bucket).bind::<SqlUuid,_>(key)
                    .bind::<SqlUuid,_>(data_id).bind::<Jsonb,_>(&input.metadata)
                    .bind::<Timestamptz,_>(chrono::Utc::now()).execute(connection)?;
                Ok(Created { id, fresh: true })
            });
            match inserted {
                Ok(created) => Ok(created),
                // The transaction has rolled back the fresh data row. A new
                // READ COMMITTED statement sees the competing committed record.
                Err(DbError::DatabaseError(DatabaseErrorKind::UniqueViolation, info))
                    if info.constraint_name() == Some("records_bucket_external_id") => {
                    #[derive(QueryableByName)]
                    struct Existing {
                        #[diesel(sql_type = SqlUuid)] id: Uuid,
                        #[diesel(sql_type = Bool)] matches: bool,
                    }
                    let existing: Existing = diesel::sql_query("SELECT r.id, (d.payload COLLATE \"C\" = $3 AND r.metadata = $4) AS matches FROM records r JOIN data d ON r.data_id = d.id WHERE r.bucket = $1 AND r.external_id = $2")
                        .bind::<Text,_>(&bucket).bind::<SqlUuid,_>(key)
                        .bind::<Text,_>(&input.payload).bind::<Jsonb,_>(&input.metadata)
                        .get_result(connection).map_err(database_error)?;
                    if existing.matches { Ok(Created {id: existing.id, fresh: false}) } else { Err(Error::Conflict) }
                }
                Err(error) => Err(database_error(error)),
            }
        }).await
    }

    pub async fn get(&self, bucket: String, id: String) -> Result<Record, Error> {
        model::bucket(&bucket)?;
        let id = model::uuid7(&id)?;
        self.blocking(move |connection| {
            diesel::sql_query("SELECT r.id, r.bucket, r.external_id AS idempotency_key, d.payload, r.metadata, r.created_at FROM records r JOIN data d ON r.data_id = d.id WHERE r.bucket = $1 AND r.id = $2")
                .bind::<Text,_>(bucket).bind::<SqlUuid,_>(id).get_result(connection).map_err(database_error)
        }).await
    }
}

pub(crate) fn database_error(error: DbError) -> Error {
    match error {
        DbError::NotFound => Error::NotFound,
        DbError::DatabaseError(
            DatabaseErrorKind::ClosedConnection
            | DatabaseErrorKind::UnableToSendCommand
            | DatabaseErrorKind::SerializationFailure,
            _,
        ) => Error::Unavailable,
        _ => Error::Internal,
    }
}
