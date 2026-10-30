fn reporting_now() -> chrono::DateTime<chrono::Utc> {
    chrono::DateTime::parse_from_rfc3339("2026-12-31T23:59:59Z")
        .unwrap()
        .with_timezone(&chrono::Utc)
}

use super::*;

async fn report(client: &TestClient<poem::Route>, name: &str) -> Value {
    let response = client
        .get(format!("/api/reporting/{name}"))
        .header("authorization", "Bearer reporting-test-token")
        .send()
        .await;
    response.assert_status_is_ok();
    assert_eq!(response.0.headers()["cache-control"], "no-store");
    let report: Value = response.json().await.value().deserialize();
    chrono::DateTime::parse_from_rfc3339(report["generated_at"].as_str().unwrap()).unwrap();
    report
}

#[tokio::test]
#[ignore = "requires DATABASE_URL_TEST with CREATE SCHEMA permission"]
async fn reports_active_schema_tables_and_storage_without_changing_queues() {
    let (schema, pool) = database(true);
    let (_other_schema, other_pool) = database(false);
    other_pool
        .get()
        .unwrap()
        .batch_execute("CREATE TABLE outside_schema (id INT)")
        .unwrap();
    pool.get().unwrap().batch_execute(
        "CREATE TABLE z_report (id INT PRIMARY KEY, value TEXT);
         INSERT INTO z_report SELECT n, repeat('value', 100) FROM generate_series(1,100) n;
         CREATE VIEW report_view AS SELECT * FROM z_report;
         INSERT INTO inbox_entries(id,payload,meta,scheduled_at) VALUES(gen_random_uuid(),'{}','{}','2026-12-31T12:00:00Z');
         INSERT INTO outbox_entries(id,payload,meta,scheduled_at) VALUES(gen_random_uuid(),'{}','{}','2026-12-31T12:00:00Z');
         INSERT INTO command_entries(id,command_type,payload,scheduled_at) VALUES(gen_random_uuid(),'Probe','{}','2026-12-31T12:00:00Z');
         INSERT INTO event_entries(id,event_type,payload,scheduled_at) VALUES(gen_random_uuid(),'Probe','{}','2026-12-31T12:00:00Z');"
    ).unwrap();
    let snapshot_sql = "SELECT jsonb_build_object(
        'inbox',(SELECT jsonb_agg(to_jsonb(t) ORDER BY id) FROM inbox_entries t),
        'commands',(SELECT jsonb_agg(to_jsonb(t) ORDER BY id) FROM command_entries t),
        'events',(SELECT jsonb_agg(to_jsonb(t) ORDER BY id) FROM event_entries t),
        'outbox',(SELECT jsonb_agg(to_jsonb(t) ORDER BY id) FROM outbox_entries t)
    ) AS data";
    let before = snapshot(&pool, snapshot_sql);
    let (kernel, mut state) = start_mulac(pool.clone()).unwrap();
    state.reporting = crate::reporting::Reporting::postgres_with_clock(pool.clone(), reporting_now);
    let client = TestClient::new(crate::app_with_reporting_token(
        state,
        "reporting-test-token".into(),
    ));
    let tables = report(&client, "database-tables").await;
    assert_eq!(tables["schema_name"], schema.name);
    assert_eq!(tables["row_count_kind"], "estimated");
    let rows = tables["tables"].as_array().unwrap();
    let names: Vec<_> = rows
        .iter()
        .map(|row| row["table_name"].as_str().unwrap())
        .collect();
    assert!(names.windows(2).all(|names| names[0] < names[1]));
    for name in [
        "offers",
        "inbox_entries",
        "command_entries",
        "event_entries",
        "outbox_entries",
        "__diesel_schema_migrations",
        "z_report",
    ] {
        assert!(names.contains(&name), "{name}");
    }
    assert!(!names.contains(&"outside_schema"));
    assert!(!names.contains(&"report_view"));
    for row in rows {
        assert!(row["row_count"].is_u64());
        assert_eq!(
            row["total_bytes"].as_u64().unwrap(),
            row["table_bytes"].as_u64().unwrap() + row["index_bytes"].as_u64().unwrap()
        );
    }
    let row = rows
        .iter()
        .find(|row| row["table_name"] == "z_report")
        .unwrap();
    let storage = snapshot(
        &pool,
        "SELECT jsonb_build_object('table',pg_table_size('z_report'),'index',pg_indexes_size('z_report')) AS data",
    );
    assert_eq!(row["table_bytes"], storage["table"]);
    assert_eq!(row["index_bytes"], storage["index"]);
    report(&client, "entry-statuses").await;
    assert_eq!(snapshot(&pool, snapshot_sql), before);
    kernel.shutdown();
}

#[tokio::test]
#[ignore = "requires DATABASE_URL_TEST with CREATE SCHEMA permission"]
async fn reports_exact_status_counts_empty_queues_and_unknown_codes() {
    let (_schema, pool) = database(true);
    let (kernel, mut state) = start_mulac(pool.clone()).unwrap();
    state.reporting = crate::reporting::Reporting::postgres_with_clock(pool.clone(), reporting_now);
    let client = TestClient::new(crate::app_with_reporting_token(
        state,
        "reporting-test-token".into(),
    ));
    let empty = report(&client, "entry-statuses").await;
    assert_eq!(empty["command_entries"], json!([]));
    assert_eq!(empty["event_entries"], json!([]));
    pool.get()
        .unwrap()
        .batch_execute(
            "INSERT INTO command_entries(id,command_type,payload,status,scheduled_at)
         SELECT gen_random_uuid(),'Probe','{}',s,'2026-12-31T12:00:00Z' FROM unnest(ARRAY[0,0,2,4,5,7,8,99]) s;
         INSERT INTO event_entries(id,event_type,payload,status,scheduled_at)
         SELECT gen_random_uuid(),'Probe','{}',s,'2026-12-31T12:00:00Z' FROM unnest(ARRAY[0,2,4,5,5,7,8,99]) s;",
        )
        .unwrap();
    let statuses = report(&client, "entry-statuses").await;
    assert_eq!(statuses["generated_at"], "2026-12-31T23:59:59Z");
    assert_eq!(statuses["window_start"], "2026-12-31T00:00:00Z");
    assert_eq!(statuses["window_end"], "2027-01-01T00:00:00Z");
    for (queue, repeated) in [("command_entries", 0), ("event_entries", 5)] {
        let expected: Vec<_> = [(0,"received"),(2,"reserved"),(4,"failed"),(5,"completed"),(7,"dead"),(8,"archive"),(99,"unknown")].into_iter().map(|(status,label)| json!({"status":status,"status_text":label,"count":if status==repeated {2}else{1}})).collect();
        assert_eq!(statuses[queue], json!(expected));
    }
    kernel.shutdown();
}

#[tokio::test]
#[ignore = "requires DATABASE_URL_TEST with CREATE SCHEMA permission"]
async fn missing_queue_tables_are_sanitized_and_empty_schema_is_valid() {
    let (schema, pool) = database(false);
    let mut state = crate::queue_tests::state();
    state.reporting = crate::reporting::Reporting::postgres(pool);
    let client = TestClient::new(crate::app_with_reporting_token(
        state,
        "reporting-test-token".into(),
    ));
    let tables = report(&client, "database-tables").await;
    assert_eq!(tables["schema_name"], schema.name);
    assert_eq!(tables["tables"], json!([]));
    let response = client
        .get("/api/reporting/entry-statuses")
        .header("authorization", "Bearer reporting-test-token")
        .send()
        .await;
    response.assert_status(StatusCode::INTERNAL_SERVER_ERROR);
    assert_eq!(response.0.headers()["cache-control"], "no-store");
    response
        .assert_json(json!({"error":"Unable to generate the database report."}))
        .await;
}
