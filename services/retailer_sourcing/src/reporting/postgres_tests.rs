use super::store;
use crate::platform::test_db::Database;
use diesel::{connection::SimpleConnection, prelude::*};

#[test]
#[ignore = "requires DATABASE_URL_TEST; run make test-postgres"]
fn postgres_report_covers_active_schema_statistics_and_storage() {
    let db = Database::new();
    let other = Database::new();
    other
        .pool
        .get()
        .unwrap()
        .batch_execute("CREATE TABLE foreign_schema_marker(id integer)")
        .unwrap();
    let mut connection = db.pool.get().unwrap();
    connection.batch_execute("CREATE TABLE report_sample(id integer PRIMARY KEY, payload text); ALTER TABLE report_sample ALTER COLUMN payload SET STORAGE EXTERNAL; INSERT INTO report_sample SELECT i, repeat(md5(i::text),1000) FROM generate_series(1,3) i; CREATE VIEW report_view AS SELECT * FROM report_sample; ANALYZE report_sample;").unwrap();
    // Publish the fixture statistics before reading the estimate. Production
    // reporting never forces a flush or runs ANALYZE.
    connection
        .batch_execute("SELECT pg_stat_force_next_flush()")
        .unwrap();
    connection
        .batch_execute("SELECT pg_stat_clear_snapshot()")
        .unwrap();
    let report = store::load(&mut connection).unwrap();
    assert_eq!(
        report.schema_name,
        diesel::select(diesel::dsl::sql::<diesel::sql_types::Text>(
            "current_schema()::text"
        ))
        .get_result::<String>(&mut connection)
        .unwrap()
    );
    assert!(
        report
            .tables
            .windows(2)
            .all(|w| w[0].table_name < w[1].table_name)
    );
    assert!(
        report
            .tables
            .iter()
            .any(|t| t.table_name == "__diesel_schema_migrations")
    );
    assert!(
        report
            .tables
            .iter()
            .any(|t| t.table_name == "command_entries")
    );
    assert!(!report.tables.iter().any(|t| matches!(
        t.table_name.as_str(),
        "foreign_schema_marker" | "report_view"
    )));
    let table = report
        .tables
        .iter()
        .find(|t| t.table_name == "report_sample")
        .unwrap();
    assert_eq!(table.row_count, 3);
    assert!(table.table_bytes > 96000);
    assert!(table.index_bytes > 0);
    assert_eq!(table.total_bytes, table.table_bytes + table.index_bytes);
    #[derive(QueryableByName)]
    struct Size {
        #[diesel(sql_type=diesel::sql_types::BigInt)]
        bytes: i64,
    }
    let size: Size = diesel::sql_query("SELECT pg_total_relation_size('report_sample') AS bytes")
        .get_result(&mut connection)
        .unwrap();
    assert_eq!(table.total_bytes, size.bytes as u64);
    connection.test_transaction::<_, diesel::result::Error, _>(|connection| {
        let empty = format!("report_empty_{}", uuid::Uuid::now_v7().simple());
        connection.batch_execute(&format!(
            "CREATE SCHEMA {empty}; SET LOCAL search_path TO {empty}"
        ))?;
        let report = store::load(connection)?;
        assert_eq!(report.schema_name, empty);
        assert!(report.tables.is_empty());
        Ok(())
    });
}

#[test]
#[ignore = "requires DATABASE_URL_TEST; run make test-postgres"]
fn postgres_reporting_route_returns_typed_contract() {
    use axum::{body::Body, http::Request};
    use http_body_util::BodyExt;
    use tower::ServiceExt;
    let db = Database::new();
    let app = crate::app(crate::platform::state::AppState {
        db: Some(db.pool.clone()),
        ..Default::default()
    });
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let response = runtime
        .block_on(
            app.oneshot(
                Request::builder()
                    .uri("/api/reporting/database-tables")
                    .header(
                        "authorization",
                        format!("Bearer {}", crate::platform::auth::BEARER_TOKEN),
                    )
                    .body(Body::empty())
                    .unwrap(),
            ),
        )
        .unwrap();
    assert_eq!(response.status(), 200);
    assert_eq!(response.headers()["cache-control"], "no-store");
    let bytes = runtime
        .block_on(response.into_body().collect())
        .unwrap()
        .to_bytes();
    let report: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(report["row_count_kind"], "estimated");
    assert!(chrono::DateTime::parse_from_rfc3339(report["generated_at"].as_str().unwrap()).is_ok());
    for table in report["tables"].as_array().unwrap() {
        assert_eq!(table.as_object().unwrap().len(), 5);
        assert_eq!(
            table["total_bytes"].as_u64().unwrap(),
            table["table_bytes"].as_u64().unwrap() + table["index_bytes"].as_u64().unwrap()
        );
    }
}
