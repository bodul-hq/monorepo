use super::prelude::routes_with_clock;
use crate::app as crate_app;
use crate::platform::auth::BEARER_TOKEN;
use crate::platform::state::AppState;
use axum::Router;
use chrono::DateTime;
use chrono::Utc;
use diesel::dsl::sql;
use diesel::result::Error as ResultError;
use diesel::select;
use diesel::sql_query;
use diesel::sql_types::BigInt;
use diesel::sql_types::Text;
use serde_json::Value;
use serde_json::from_slice;
use serde_json::json;
use store::load;
use store::load_entry_statuses_at;
use tokio::runtime::Runtime;
use tokio::test as tokio_test;
use uuid::Uuid;
fn reporting_now() -> DateTime<Utc> {
    DateTime::parse_from_rfc3339("2026-12-31T23:59:59Z")
        .unwrap()
        .with_timezone(&Utc)
}

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
    let report = load(&mut connection).unwrap();
    assert_eq!(
        report.schema_name,
        select(sql::<Text>("current_schema()::text"))
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
        #[diesel(sql_type=BigInt)]
        bytes: i64,
    }
    let size: Size = sql_query("SELECT pg_total_relation_size('report_sample') AS bytes")
        .get_result(&mut connection)
        .unwrap();
    assert_eq!(table.total_bytes, size.bytes as u64);
    connection.test_transaction::<_, ResultError, _>(|connection| {
        let empty = format!("report_empty_{}", Uuid::now_v7().simple());
        connection.batch_execute(&format!(
            "CREATE SCHEMA {empty}; SET LOCAL search_path TO {empty}"
        ))?;
        let report = load(connection)?;
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
    let app = crate_app(AppState {
        db: Some(db.pool.clone()),
        ..Default::default()
    });
    let runtime = Runtime::new().unwrap();
    let response = runtime
        .block_on(
            app.oneshot(
                Request::builder()
                    .uri("/api/reporting/database-tables")
                    .header("authorization", format!("Bearer {}", BEARER_TOKEN))
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
    let report: Value = from_slice(&bytes).unwrap();
    assert_eq!(report["row_count_kind"], "estimated");
    assert!(DateTime::parse_from_rfc3339(report["generated_at"].as_str().unwrap()).is_ok());
    for table in report["tables"].as_array().unwrap() {
        assert_eq!(table.as_object().unwrap().len(), 5);
        assert_eq!(
            table["total_bytes"].as_u64().unwrap(),
            table["table_bytes"].as_u64().unwrap() + table["index_bytes"].as_u64().unwrap()
        );
    }
}

#[test]
#[ignore = "requires DATABASE_URL_TEST; run make test-postgres"]
fn postgres_entry_statuses_group_counts_and_report_changes() {
    let db = Database::new();
    let mut conn = db.pool.get().unwrap();
    let empty = load_entry_statuses_at(&mut conn, reporting_now()).unwrap();
    assert!(empty.command_entries.is_empty());
    assert!(empty.event_entries.is_empty());
    assert!(empty.inbox_entries.is_empty());
    assert!(empty.outbox_entries.is_empty());
    conn.batch_execute(
        "INSERT INTO command_entries(id,command_type,payload,status,scheduled_at)
        SELECT gen_random_uuid(), 'Test', '{}', s,'2026-12-31T12:00:00Z' FROM unnest(ARRAY[0,0,2,4,5,7,8,99]) s;
        INSERT INTO event_entries(id,event_type,payload,status,scheduled_at)
        VALUES (gen_random_uuid(), 'Test', '{}', 4, '2026-12-31T12:00:00Z'), (gen_random_uuid(), 'Test', '{}', 4, '2026-12-31T12:00:00Z');
        INSERT INTO inbox_entries(id,payload,meta,status,scheduled_at) SELECT gen_random_uuid(), '{}', '{}', s,'2026-12-31T12:00:00Z' FROM unnest(ARRAY[0,0,0,99]) s;
        INSERT INTO outbox_entries(id,payload,meta,status,scheduled_at) SELECT gen_random_uuid(), '{}', '{}', s,'2026-12-31T12:00:00Z' FROM unnest(ARRAY[5,5,8]) s",
    )
    .unwrap();
    let report = load_entry_statuses_at(&mut conn, reporting_now()).unwrap();
    let groups: Vec<_> = report
        .command_entries
        .iter()
        .map(|r| (r.status, r.status_text, r.count))
        .collect();
    assert_eq!(
        groups,
        vec![
            (0, "received", 2),
            (2, "reserved", 1),
            (4, "failed", 1),
            (5, "completed", 1),
            (7, "dead", 1),
            (8, "archive", 1),
            (99, "unknown", 1)
        ]
    );
    assert_eq!(
        report
            .inbox_entries
            .iter()
            .map(|r| (r.status, r.count))
            .collect::<Vec<_>>(),
        vec![(0, 3), (99, 1)]
    );
    assert_eq!(
        report
            .outbox_entries
            .iter()
            .map(|r| (r.status, r.count))
            .collect::<Vec<_>>(),
        vec![(5, 2), (8, 1)]
    );
    assert_eq!(report.event_entries.len(), 1);
    assert_eq!(report.event_entries[0].status_text, "failed");
    assert_eq!(report.event_entries[0].count, 2);
    conn.batch_execute("UPDATE command_entries SET status = 5 WHERE status = 0")
        .unwrap();
    let report = load_entry_statuses_at(&mut conn, reporting_now()).unwrap();
    assert!(!report.command_entries.iter().any(|r| r.status == 0));
    assert_eq!(
        report
            .command_entries
            .iter()
            .find(|r| r.status == 5)
            .unwrap()
            .count,
        3
    );
}

#[tokio_test]
#[ignore = "requires DATABASE_URL_TEST; run make test-postgres"]
async fn postgres_entry_statuses_route_returns_exact_contract() {
    use axum::{body::Body, http::Request};
    use http_body_util::BodyExt;
    use tower::ServiceExt;
    let db = Database::new();
    db.pool.get().unwrap().batch_execute(
        "INSERT INTO event_entries(id,event_type,payload,status,scheduled_at) VALUES(gen_random_uuid(),'Test','{}',2,'2026-12-31T12:00:00Z')"
    ).unwrap();
    let router = routes_with_clock(reporting_now).with_state(AppState {
        db: Some(db.pool.clone()),
        ..Default::default()
    });
    let app = Router::new().nest("/api", router.split_for_parts().0);
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/reporting/entry-statuses")
                .header("authorization", format!("Bearer {}", BEARER_TOKEN))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), 200);
    assert_eq!(response.headers()["cache-control"], "no-store");
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let report: Value = from_slice(&bytes).unwrap();
    assert!(DateTime::parse_from_rfc3339(report["generated_at"].as_str().unwrap()).is_ok());
    assert_eq!(report["generated_at"], "2026-12-31T23:59:59Z");
    assert_eq!(report["window_start"], "2026-12-31T00:00:00Z");
    assert_eq!(report["window_end"], "2027-01-01T00:00:00Z");
    assert_eq!(report["command_entries"], json!([]));
    assert_eq!(report["inbox_entries"], json!([]));
    assert_eq!(report["outbox_entries"], json!([]));
    assert_eq!(
        report["event_entries"],
        json!([{"status":2,"status_text":"reserved","count":1}])
    );
}
