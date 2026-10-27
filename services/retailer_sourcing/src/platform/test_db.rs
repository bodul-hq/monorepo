//! Isolated PostgreSQL schemas for opt-in service tests.
use super::db::DbPool;
use diesel::connection::SimpleConnection;
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, CustomizeConnection, Pool};
use diesel_migrations::{EmbeddedMigrations, MigrationHarness, embed_migrations};
use std::time::Duration;
use uuid::Uuid;
const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

#[derive(Debug)]
struct SearchPath(String);
impl CustomizeConnection<PgConnection, diesel::r2d2::Error> for SearchPath {
    fn on_acquire(&self, conn: &mut PgConnection) -> Result<(), diesel::r2d2::Error> {
        conn.batch_execute(&format!("SET search_path TO {}", self.0))
            .map_err(diesel::r2d2::Error::QueryError)
    }
}

pub(crate) struct Database {
    admin: PgConnection,
    schema: String,
    pub pool: DbPool,
}
impl Database {
    /// Keep historical migration tests independent of newly appended migrations.
    pub fn at(version: &str) -> Self {
        let db = Self::new();
        {
            let mut connection = db.pool.get().unwrap();
            assert!(
                connection
                    .applied_migrations()
                    .unwrap()
                    .iter()
                    .any(|entry| entry.to_string() == version)
            );
            while connection.applied_migrations().unwrap()[0].to_string() != version {
                connection.revert_last_migration(MIGRATIONS).unwrap();
            }
        }
        db
    }

    pub fn new() -> Self {
        let url = std::env::var("DATABASE_URL_TEST")
            .expect("DATABASE_URL_TEST is required; use make test-postgres");
        let mut admin = PgConnection::establish(&url)
            .expect("DATABASE_URL_TEST must point to an available PostgreSQL database");
        let schema = format!("sitemap_test_{}", Uuid::now_v7().simple());
        admin
            .batch_execute(&format!("CREATE SCHEMA {schema}"))
            .unwrap();
        let result = Pool::builder()
            .max_size(2)
            .connection_timeout(Duration::from_secs(5))
            .connection_customizer(Box::new(SearchPath(schema.clone())))
            .build(ConnectionManager::new(url));
        let pool = match result {
            Ok(pool) => pool,
            Err(error) => {
                admin
                    .batch_execute(&format!("DROP SCHEMA {schema} CASCADE"))
                    .unwrap();
                panic!("test connection pool failed: {error}");
            }
        };
        let db = Self {
            admin,
            schema,
            pool,
        };
        db.pool
            .get()
            .unwrap()
            .run_pending_migrations(MIGRATIONS)
            .unwrap();
        db
    }
}
impl Drop for Database {
    fn drop(&mut self) {
        if let Err(error) = self
            .admin
            .batch_execute(&format!("DROP SCHEMA {} CASCADE", self.schema))
        {
            eprintln!("could not remove test schema {}: {error}", self.schema);
        }
    }
}

#[derive(diesel::QueryableByName)]
struct JsonRow {
    #[diesel(sql_type = diesel::sql_types::Jsonb)]
    value: serde_json::Value,
}

pub(crate) fn json_query(conn: &mut PgConnection, sql: &str) -> serde_json::Value {
    diesel::sql_query(sql)
        .get_result::<JsonRow>(conn)
        .unwrap()
        .value
}

mod postgres_tests {
    use super::*;

    #[test]
    #[ignore = "requires DATABASE_URL_TEST; run make test-postgres"]
    fn postgres_table_renames_preserve_rows_payloads_and_foreign_keys() {
        let db = Database::at("20261016000012");
        let mut conn = db.pool.get().unwrap();
        conn.revert_last_migration(MIGRATIONS).unwrap(); // Undo remaining object storage.
        conn.revert_last_migration(MIGRATIONS).unwrap(); // Undo raw object storage.
        conn.revert_last_migration(MIGRATIONS).unwrap(); // Undo pipeline storage.
        conn.revert_last_migration(MIGRATIONS).unwrap();
        let tables = [
            (
                "raw_offers",
                "offer_details_raw",
                "offer_id,url,body_size,retailer_code",
                "gen_random_uuid(),'https://shop.test',1,'ankerau'",
                "body",
                "'ž'",
            ),
            (
                "extracted_offers",
                "offer_details_extracted",
                "offer_id",
                "gen_random_uuid()",
                "document",
                "'{}'",
            ),
            (
                "normalized_offers",
                "offer_details_normalized",
                "offer_id",
                "gen_random_uuid()",
                "document",
                "'{}'",
            ),
            (
                "structuralized_offers",
                "offer_details_structuralized",
                "offer_id",
                "gen_random_uuid()",
                "document",
                "'{}'",
            ),
            (
                "raw_sitemap_documents",
                "sitemaps_raw",
                "retrieval_id,url,body_size",
                "gen_random_uuid(),'https://shop.test/sitemap.xml',1",
                "body",
                "'ž'",
            ),
            (
                "processed_sitemaps",
                "sitemaps_processed",
                "retrieval_id,retailer_code,document_size,url_count",
                "gen_random_uuid(),'ankerau',2,0",
                "document",
                "'{}'",
            ),
            (
                "consolidated_sitemaps",
                "sitemaps_consolidated",
                "stats,\"references\",storage",
                "'{}','{}','{}'",
                "content",
                "'{}'",
            ),
        ];
        let snapshot = |conn: &mut PgConnection, table: &str, payload: &str| {
            json_query(
                conn,
                &format!(
                    "SELECT jsonb_build_object('oid', '{table}'::regclass::oid, 'row', to_jsonb(t), 'payload', to_jsonb(d)) AS value FROM {table} t JOIN {payload} d USING(id)"
                ),
            )
        };
        let mut before = Vec::new();
        for (old, _, columns, values, payload_column, payload_value) in tables {
            conn.batch_execute(&format!(
            "INSERT INTO {old} (id,{columns}) VALUES (gen_random_uuid(),{values}); INSERT INTO {old}_data (id,{payload_column}) SELECT id,{payload_value} FROM {old}"
        )).unwrap();
            before.push(snapshot(&mut conn, old, &format!("{old}_data")));
        }
        conn.run_next_migration(MIGRATIONS).unwrap();
        for ((old, new, ..), expected) in tables.iter().zip(&before) {
            assert_eq!(snapshot(&mut conn, new, &format!("{old}_data")), *expected);
            assert_eq!(
                json_query(
                    &mut conn,
                    &format!("SELECT to_jsonb(to_regclass('{old}') IS NULL) AS value")
                ),
                serde_json::json!(true)
            );
        }
        conn.revert_last_migration(MIGRATIONS).unwrap();
        for ((old, ..), expected) in tables.iter().zip(&before) {
            assert_eq!(snapshot(&mut conn, old, &format!("{old}_data")), *expected);
        }
        conn.run_next_migration(MIGRATIONS).unwrap();
        for ((old, new, ..), expected) in tables.iter().zip(&before) {
            assert_eq!(snapshot(&mut conn, new, &format!("{old}_data")), *expected);
            conn.batch_execute(&format!("DELETE FROM {new}")).unwrap();
            assert_eq!(
                json_query(
                    &mut conn,
                    &format!("SELECT to_jsonb(count(*)) AS value FROM {old}_data")
                ),
                serde_json::json!(0)
            );
        }
    }
}
