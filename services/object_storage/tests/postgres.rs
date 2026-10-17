//! Explicit opt-in: make test-postgres. Every test owns a random PostgreSQL
//! schema and removes it on drop; no existing tables or records are touched.
mod common;
use axum::{
    Router,
    http::{StatusCode, header},
};
use common::*;
use diesel::{connection::SimpleConnection, prelude::*, sql_types::BigInt};
use object_storage::{app, model::BUCKETS, store::Store};
use serde_json::{Value, json};
use uuid::Uuid;

struct SchemaGuard {
    url: String,
    schema: String,
}
struct Database {
    app: Router,
    store: Store,
    schema: SchemaGuard,
}
impl Database {
    fn new() -> Self {
        let url = std::env::var("DATABASE_URL_TEST")
            .expect("DATABASE_URL_TEST must point to an available PostgreSQL test database");
        let mut connection =
            PgConnection::establish(&url).expect("DATABASE_URL_TEST is unavailable");
        let schema = format!("object_storage_test_{}", Uuid::new_v4().simple());
        connection
            .batch_execute(&format!("CREATE SCHEMA {schema}"))
            .unwrap();
        let guard = SchemaGuard {
            url: url.clone(),
            schema: schema.clone(),
        };
        let separator = if url.contains('?') { '&' } else { '?' };
        let scoped = format!("{url}{separator}options=-csearch_path%3D{schema}");
        let store =
            Store::connect(&scoped).expect("test database migrations and UTF8 check must succeed");
        Self {
            app: app(store.clone()),
            store,
            schema: guard,
        }
    }
    fn counts(&self) -> (i64, i64) {
        #[derive(QueryableByName)]
        struct Counts {
            #[diesel(sql_type = BigInt)]
            records: i64,
            #[diesel(sql_type = BigInt)]
            data: i64,
        }
        let counts: Counts = diesel::sql_query(
            "SELECT (SELECT count(*) FROM records) AS records, (SELECT count(*) FROM data) AS data",
        )
        .get_result(&mut self.store.0.get().unwrap())
        .unwrap();
        (counts.records, counts.data)
    }
    fn sql(&self, sql: &str) {
        self.store.0.get().unwrap().batch_execute(sql).unwrap();
    }
}
impl Drop for SchemaGuard {
    fn drop(&mut self) {
        if let Ok(mut connection) = PgConnection::establish(&self.url) {
            connection
                .batch_execute(&format!("DROP SCHEMA {} CASCADE", self.schema))
                .expect("clean up isolated test schema");
        }
    }
}
async fn database() -> Database {
    tokio::task::spawn_blocking(Database::new).await.unwrap()
}

#[tokio::test]
#[ignore = "requires DATABASE_URL_TEST; run make test-postgres"]
async fn round_trip_all_buckets_through_both_apis() {
    let db = database().await;
    let mut body = input();
    for bucket in BUCKETS {
        let path = format!("/api/buckets/{bucket}/records");
        let (status, headers, created) = request(&db.app, "POST", &path, body.clone()).await;
        assert_eq!(status, StatusCode::CREATED, "{created}");
        assert_eq!(headers[header::CONTENT_TYPE], "application/json");
        assert_eq!(created.as_object().unwrap().len(), 1);
        let id = created["id"].as_str().unwrap();
        assert_eq!(Uuid::parse_str(id).unwrap().get_version_num(), 7);
        let location = format!("{path}/{id}");
        assert_eq!(headers[header::LOCATION], location);
        let (status, headers, retry) = request(&db.app, "POST", &path, body.clone()).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(retry, created);
        assert_eq!(headers[header::LOCATION], location);
        let (status, _, record) = request(&db.app, "GET", &location, Value::Null).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(record.as_object().unwrap().len(), 6);
        assert_eq!(record["payload"], body["payload"]);
        assert_eq!(record["metadata"], body["metadata"]);
        assert_eq!(record["bucket"], *bucket);
        assert_eq!(
            record["idempotency_key"],
            body["idempotency_key"].as_str().unwrap().to_lowercase()
        );
        assert!(record["created_at"].as_str().unwrap().ends_with('Z'));
        chrono::DateTime::parse_from_rfc3339(record["created_at"].as_str().unwrap()).unwrap();
        let (status, headers, gql) =
            request(&db.app, "POST", "/graphql", gql_input(bucket, &body)).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(
            headers[header::CONTENT_TYPE],
            "application/graphql-response+json"
        );
        assert_eq!(gql, json!({"data":{"createRecord":created}}));
        let (status, _, gql) = request(&db.app, "POST", "/graphql", json!({"query":"query($bucket:String!, $id:ID!){record(bucket:$bucket,id:$id){id bucket idempotencyKey payload metadata createdAt}}", "variables":{"bucket":bucket,"id":id}})).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(gql["data"]["record"]["payload"], body["payload"]);
        assert_eq!(gql["data"]["record"]["metadata"], body["metadata"]);
        assert_eq!(gql["data"]["record"]["createdAt"], record["created_at"]);
        // The same parsed key works independently in every bucket.
        body["idempotency_key"] = body["idempotency_key"]
            .as_str()
            .unwrap()
            .to_uppercase()
            .into();
    }
    assert_eq!(db.counts(), (BUCKETS.len() as i64, BUCKETS.len() as i64));
}

#[tokio::test]
#[ignore = "requires DATABASE_URL_TEST; run make test-postgres"]
async fn missing_records_wrong_buckets_and_execution_errors() {
    let db = database().await;
    let (_, _, result) =
        request(&db.app, "POST", "/api/buckets/sitemap_raw/records", input()).await;
    let id = result["id"].as_str().unwrap();
    for id in [id.to_owned(), Uuid::now_v7().to_string()] {
        let (status, headers, body) = request(
            &db.app,
            "GET",
            &format!("/api/buckets/offer_details_raw/records/{id}"),
            Value::Null,
        )
        .await;
        assert_eq!(status, StatusCode::NOT_FOUND);
        problem(status, &headers, &body);
        let (status, _, body) = request(&db.app, "POST", "/graphql", json!({"query":"query($id:ID!){ok:__typename missing:record(bucket:\"offer_details_raw\",id:$id){id}}", "variables":{"id":id}})).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(body["data"], json!({"ok":"Query", "missing":null}));
        assert_eq!(body["errors"][0]["extensions"]["code"], "NOT_FOUND");
        assert_eq!(body["errors"][0]["path"], json!(["missing"]));
        assert!(body["errors"][0]["locations"].is_array());
    }
    for key in ["invalid".to_owned(), Uuid::new_v4().to_string()] {
        let mut body = input();
        body["idempotency_key"] = key.into();
        let (_, _, result) =
            request(&db.app, "POST", "/graphql", gql_input("sitemap_raw", &body)).await;
        assert_eq!(result["errors"][0]["extensions"]["code"], "BAD_USER_INPUT");
    }
    assert_eq!(db.counts(), (1, 1));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
#[ignore = "requires DATABASE_URL_TEST; run make test-postgres"]
async fn concurrent_cross_api_retries_never_leave_orphans() {
    let db = database().await;
    for conflict in [false, true] {
        let body = input();
        let barrier = std::sync::Arc::new(tokio::sync::Barrier::new(16));
        let mut jobs = tokio::task::JoinSet::new();
        for i in 0..16 {
            let app = db.app.clone();
            let mut body = body.clone();
            let barrier = barrier.clone();
            if conflict {
                body["payload"] = format!("contender {i}").into();
            }
            jobs.spawn(async move {
                barrier.wait().await;
                if i % 2 == 0 {
                    let (status, _, result) =
                        request(&app, "POST", "/api/buckets/sitemap_raw/records", body).await;
                    (status == StatusCode::CONFLICT, result["id"].clone())
                } else {
                    let (status, _, result) =
                        request(&app, "POST", "/graphql", gql_input("sitemap_raw", &body)).await;
                    assert_eq!(status, StatusCode::OK);
                    (
                        result["errors"][0]["extensions"]["code"] == "CONFLICT",
                        result["data"]["createRecord"]["id"].clone(),
                    )
                }
            });
        }
        let mut ids = Vec::new();
        let mut conflicts = 0;
        while let Some(result) = jobs.join_next().await {
            let (conflicted, id) = result.unwrap();
            if conflicted {
                conflicts += 1;
            } else {
                assert!(id.is_string());
                ids.push(id);
            }
        }
        assert_eq!(conflicts, if conflict { 15 } else { 0 });
        assert!(ids.iter().all(|id| id == &ids[0]));
    }
    assert_eq!(db.counts(), (2, 2));
    let mut body = input();
    for _ in 0..2 {
        body["idempotency_key"] = Uuid::now_v7().to_string().into();
        assert_eq!(
            request(
                &db.app,
                "POST",
                "/api/buckets/sitemap_raw/records",
                body.clone()
            )
            .await
            .0,
            StatusCode::CREATED
        );
    }
    assert_eq!(db.counts(), (4, 4));
    #[derive(QueryableByName)]
    struct Count {
        #[diesel(sql_type = BigInt)]
        count: i64,
    }
    let unique: Count = diesel::sql_query("SELECT count(DISTINCT data_id) AS count FROM records")
        .get_result(&mut db.store.0.get().unwrap())
        .unwrap();
    assert_eq!(unique.count, 4);
    // The schema intentionally allows records to share data after future deduplication.
    db.sql("UPDATE records SET data_id = (SELECT data_id FROM records LIMIT 1)");
}

#[tokio::test]
#[ignore = "requires DATABASE_URL_TEST; run make test-postgres"]
async fn precise_metadata_equality_and_unsupported_values() {
    let db = database().await;
    let key = Uuid::now_v7();
    let body: Value = serde_json::from_str(&format!(r#"{{"idempotency_key":"{key}","payload":"雪 🌍","metadata":{{"large":1234567890123456789012345678901234567890,"decimal":0.123456789012345678901234567890,"list":[1,2]}}}}"#)).unwrap();
    let (_, headers, created) = request(
        &db.app,
        "POST",
        "/api/buckets/sitemap_raw/records",
        body.clone(),
    )
    .await;
    let location = headers[header::LOCATION].to_str().unwrap();
    let (_, _, record) = request(&db.app, "GET", location, Value::Null).await;
    assert_eq!(
        record["metadata"]["large"].to_string(),
        "1234567890123456789012345678901234567890"
    );
    assert_eq!(
        record["metadata"]["decimal"].to_string(),
        "0.123456789012345678901234567890"
    );
    let mut retry = body.clone();
    retry["metadata"] = serde_json::from_str(r#"{"list":[1.0,2e0],"decimal":0.1234567890123456789012345678900,"large":1234567890123456789012345678901234567890.0}"#).unwrap();
    let (_, _, gql) = request(
        &db.app,
        "POST",
        "/graphql",
        gql_input("sitemap_raw", &retry),
    )
    .await;
    assert_eq!(gql, json!({"data":{"createRecord":created}}));
    let (_, _, gql) = request(&db.app, "POST", "/graphql", json!({"query":"query($id:ID!){record(bucket:\"sitemap_raw\",id:$id){metadata}}","variables":{"id":created["id"]}})).await;
    assert_eq!(gql["data"]["record"]["metadata"], record["metadata"]);
    // Inline GraphQL numbers preserve the same precision as JSON variables.
    let query = format!(
        r#"mutation {{ createRecord(input: {{bucket:"sitemap_raw",idempotencyKey:"{key}",payload:"雪 🌍",metadata:{{list:[1,2],large:1234567890123456789012345678901234567890,decimal:0.123456789012345678901234567890}}}}){{id}} }}"#
    );
    let (_, _, gql) = request(&db.app, "POST", "/graphql", json!({"query":query})).await;
    assert_eq!(gql, json!({"data":{"createRecord":created}}));
    retry["metadata"]["list"] = json!([2, 1]);
    assert_eq!(
        request(&db.app, "POST", "/api/buckets/sitemap_raw/records", retry)
            .await
            .0,
        StatusCode::CONFLICT
    );
    let mut retry = body.clone();
    retry["metadata"]["decimal"] =
        serde_json::from_str("0.123456789012345678901234567891").unwrap();
    let (_, _, gql) = request(
        &db.app,
        "POST",
        "/graphql",
        gql_input("sitemap_raw", &retry),
    )
    .await;
    assert_eq!(gql["errors"][0]["extensions"]["code"], "CONFLICT");
    for metadata in [
        Value::Null,
        json!({"key":"\u{0}"}),
        json!({"\u{0}":true}),
        serde_json::from_str("1e131072").unwrap(),
        serde_json::from_str("1e-16384").unwrap(),
    ] {
        let mut invalid = input();
        invalid["metadata"] = metadata;
        let (status, headers, result) = request(
            &db.app,
            "POST",
            "/api/buckets/sitemap_raw/records",
            invalid.clone(),
        )
        .await;
        assert_eq!(status, StatusCode::BAD_REQUEST, "{result}");
        problem(status, &headers, &result);
        let (status, _, result) = request(
            &db.app,
            "POST",
            "/graphql",
            gql_input("sitemap_raw", &invalid),
        )
        .await;
        assert!(status == StatusCode::OK || status == StatusCode::BAD_REQUEST);
        assert_eq!(result["errors"][0]["extensions"]["code"], "BAD_USER_INPUT");
    }
    assert_eq!(db.counts(), (1, 1));
}

#[tokio::test]
#[ignore = "requires DATABASE_URL_TEST; run make test-postgres"]
async fn invalid_variables_prevent_all_mutations_and_valid_defaults_work() {
    let db = database().await;
    let body = input();
    let gql = gql_input("sitemap_raw", &body);
    for invalid in [
        Value::Null,
        json!({}),
        json!({"bucket":"sitemap_raw","idempotencyKey":body["idempotency_key"],"payload":4,"metadata":{}}),
    ] {
        let query = "mutation($good:CreateRecordInput!, $bad:CreateRecordInput!) { first:createRecord(input:$good){id} second:createRecord(input:$bad){id} }";
        let (status, _, result) = request(
            &db.app,
            "POST",
            "/graphql",
            json!({"query":query,"variables":{"good":gql["variables"]["input"],"bad":invalid}}),
        )
        .await;
        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert!(result.get("data").is_none());
        assert_eq!(result["errors"][0]["extensions"]["code"], "BAD_USER_INPUT");
        assert_eq!(db.counts(), (0, 0));
    }
    let query = "mutation($good:CreateRecordInput!, $bad:Boolean!) { first:createRecord(input:$good){id} second:createRecord(input:$good) @skip(if:$bad){id} }";
    assert_eq!(
        request(
            &db.app,
            "POST",
            "/graphql",
            json!({"query":query,"variables":{"good":gql["variables"]["input"],"bad":1}})
        )
        .await
        .0,
        StatusCode::BAD_REQUEST
    );
    assert_eq!(db.counts(), (0, 0));
    let query = format!(
        r#"mutation($key:String! = "{}", $meta:JSON! = {{value:123456789012345678901234567890}}) {{createRecord(input:{{bucket:"sitemap_raw",idempotencyKey:$key,payload:"default",metadata:$meta}}){{id}}}}"#,
        Uuid::now_v7()
    );
    let (_, _, result) = request(&db.app, "POST", "/graphql", json!({"query":query})).await;
    assert!(result["data"]["createRecord"]["id"].is_string(), "{result}");
    assert_eq!(db.counts(), (1, 1));
}

#[tokio::test]
#[ignore = "requires DATABASE_URL_TEST; run make test-postgres"]
async fn large_bodies_and_transaction_rollback() {
    let db = database().await;
    let mut body = input();
    body["payload"] = "雪".repeat(800_000).into();
    body["metadata"] = json!({"large":"m".repeat(2_200_000)});
    let (status, headers, _) = request(
        &db.app,
        "POST",
        "/api/buckets/sitemap_raw/records",
        body.clone(),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    let (_, _, record) = request(
        &db.app,
        "GET",
        headers[header::LOCATION].to_str().unwrap(),
        Value::Null,
    )
    .await;
    assert_eq!(record["payload"], body["payload"]);
    assert_eq!(record["metadata"], body["metadata"]);
    body["idempotency_key"] = Uuid::now_v7().to_string().into();
    let (_, _, result) =
        request(&db.app, "POST", "/graphql", gql_input("sitemap_raw", &body)).await;
    let id = result["data"]["createRecord"]["id"].as_str().unwrap();
    let (_, _, result) = request(&db.app, "POST", "/graphql", json!({"query":"query($id:ID!){record(bucket:\"sitemap_raw\",id:$id){payload metadata}}","variables":{"id":id}})).await;
    assert_eq!(result["data"]["record"]["payload"], body["payload"]);
    assert_eq!(result["data"]["record"]["metadata"], body["metadata"]);
    assert_eq!(db.counts(), (2, 2));
    // Inject a records-table failure after the data insert, inside this schema.
    db.sql("ALTER TABLE records ADD CONSTRAINT injected_failure CHECK (bucket <> 'sitemap_raw') NOT VALID");
    let (status, headers, result) =
        request(&db.app, "POST", "/api/buckets/sitemap_raw/records", input()).await;
    assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
    problem(status, &headers, &result);
    assert!(!result.to_string().contains("injected_failure"));
    let (_, _, result) = request(
        &db.app,
        "POST",
        "/graphql",
        gql_input("sitemap_raw", &input()),
    )
    .await;
    assert_eq!(
        result["errors"][0]["extensions"]["code"],
        "INTERNAL_SERVER_ERROR"
    );
    assert_eq!(db.counts(), (2, 2));
}

#[tokio::test]
#[ignore = "requires DATABASE_URL_TEST; run make test-postgres"]
async fn database_disconnects_are_temporary_and_recover() {
    let db = database().await;
    db.sql("CREATE FUNCTION disconnect_writer() RETURNS trigger LANGUAGE plpgsql AS $$ BEGIN PERFORM pg_terminate_backend(pg_backend_pid()); RETURN NEW; END $$; CREATE TRIGGER disconnect_writer BEFORE INSERT ON records FOR EACH ROW EXECUTE FUNCTION disconnect_writer()");
    let body = input();
    let (status, headers, result) = request(
        &db.app,
        "POST",
        "/api/buckets/sitemap_raw/records",
        body.clone(),
    )
    .await;
    assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE, "{result}");
    problem(status, &headers, &result);
    let (_, _, result) =
        request(&db.app, "POST", "/graphql", gql_input("sitemap_raw", &body)).await;
    assert_eq!(
        result["errors"][0]["extensions"]["code"],
        "SERVICE_UNAVAILABLE"
    );
    assert_eq!(db.counts(), (0, 0));
    db.sql("DROP TRIGGER disconnect_writer ON records");
    assert_eq!(
        request(&db.app, "POST", "/api/buckets/sitemap_raw/records", body)
            .await
            .0,
        StatusCode::CREATED
    );
    assert_eq!(db.counts(), (1, 1));
}

#[tokio::test]
#[ignore = "requires DATABASE_URL_TEST; run make test-postgres"]
async fn every_non_null_json_shape_and_decoded_payloads_round_trip() {
    let db = database().await;
    for metadata in [
        json!(false),
        json!("metadata"),
        json!([null, true, 12]),
        json!(12345),
        serde_json::from_str("0e131072").unwrap(),
    ] {
        let mut body = input();
        body["metadata"] = metadata.clone();
        let (_, _, result) =
            request(&db.app, "POST", "/graphql", gql_input("sitemap_raw", &body)).await;
        let id = result["data"]["createRecord"]["id"].as_str().unwrap();
        let (_, _, result) = request(
            &db.app,
            "GET",
            &format!("/api/buckets/sitemap_raw/records/{id}"),
            Value::Null,
        )
        .await;
        if metadata == serde_json::from_str::<Value>("0e131072").unwrap() {
            assert_eq!(result["metadata"], json!(0));
        } else {
            assert_eq!(result["metadata"], metadata);
        }
        assert_eq!(
            request(&db.app, "POST", "/api/buckets/sitemap_raw/records", body)
                .await
                .0,
            StatusCode::OK
        );
    }
    let mut body = input();
    body["payload"] = json!("雪");
    let (status, _, _) = request(
        &db.app,
        "POST",
        "/api/buckets/sitemap_raw/records",
        body.clone(),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    let escaped = body.to_string().replace('雪', "\\u96ea");
    let (status, _, _) = raw(
        &db.app,
        "POST",
        "/api/buckets/sitemap_raw/records",
        Some(object_storage::BEARER_TOKEN),
        Some("application/json"),
        escaped,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    body["payload"] = json!("雪\u{0}");
    let (_, _, result) =
        request(&db.app, "POST", "/graphql", gql_input("sitemap_raw", &body)).await;
    assert_eq!(result["errors"][0]["extensions"]["code"], "BAD_USER_INPUT");
    assert_eq!(db.counts(), (6, 6));
}

#[tokio::test]
#[ignore = "requires DATABASE_URL_TEST; run make test-postgres"]
async fn startup_refuses_failed_migrations() {
    let db = database().await;
    // Remove only this isolated schema's migration history: replay must fail
    // against the existing tables, and startup must not continue serving.
    db.sql("DROP TABLE __diesel_schema_migrations");
    let separator = if db.schema.url.contains('?') {
        '&'
    } else {
        '?'
    };
    let scoped = format!(
        "{}{separator}options=-csearch_path%3D{}",
        db.schema.url, db.schema.schema
    );
    let result = tokio::task::spawn_blocking(move || Store::connect(&scoped))
        .await
        .unwrap();
    assert!(matches!(result, Err("Database migrations failed.")));
}

#[tokio::test]
#[ignore = "requires DATABASE_URL_TEST; run make test-postgres"]
async fn reporting_covers_active_schema_estimates_and_storage() {
    let db = database().await;
    let other = database().await;
    other.sql("CREATE TABLE foreign_schema_marker(id integer)");
    let mut connection = db.store.0.get().unwrap();
    connection.batch_execute("CREATE TABLE report_sample(id integer PRIMARY KEY, payload text); ALTER TABLE report_sample ALTER COLUMN payload SET STORAGE EXTERNAL; INSERT INTO report_sample SELECT i, repeat(md5(i::text),1000) FROM generate_series(1,3) i; CREATE VIEW report_view AS SELECT * FROM report_sample; ANALYZE report_sample;").unwrap();
    // Flush only the test fixture statistics before querying the estimate.
    connection
        .batch_execute("SELECT pg_stat_force_next_flush()")
        .unwrap();
    drop(connection);
    let (status, headers, report) = request(
        &db.app,
        "GET",
        "/api/reporting/database-tables",
        Value::Null,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(headers["cache-control"], "no-store");
    assert_eq!(report["schema_name"], db.schema.schema);
    assert_eq!(report["row_count_kind"], "estimated");
    assert!(chrono::DateTime::parse_from_rfc3339(report["generated_at"].as_str().unwrap()).is_ok());
    let tables = report["tables"].as_array().unwrap();
    assert!(
        tables
            .windows(2)
            .all(|w| w[0]["table_name"].as_str() < w[1]["table_name"].as_str())
    );
    assert!(
        tables
            .iter()
            .any(|t| t["table_name"] == "__diesel_schema_migrations")
    );
    assert!(tables.iter().any(|t| t["table_name"] == "data"));
    assert!(!tables.iter().any(|t|t["table_name"]=="foreign_schema_marker" || t["table_name"]=="report_view"));
    let sample = tables
        .iter()
        .find(|t| t["table_name"] == "report_sample")
        .unwrap();
    assert_eq!(sample["row_count"], 3);
    assert!(sample["table_bytes"].as_u64().unwrap() > 96000);
    assert!(sample["index_bytes"].as_u64().unwrap() > 0);
    for table in tables {
        assert_eq!(table.as_object().unwrap().len(), 5);
        assert_eq!(
            table["total_bytes"].as_u64().unwrap(),
            table["table_bytes"].as_u64().unwrap() + table["index_bytes"].as_u64().unwrap()
        );
    }
    // Empty the isolated schema only, retaining its connection search path.
    db.sql("DROP VIEW report_view; DROP TABLE report_sample,records,data,__diesel_schema_migrations CASCADE");
    let (status, _, report) = request(
        &db.app,
        "GET",
        "/api/reporting/database-tables",
        Value::Null,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(report["tables"], json!([]));
}
