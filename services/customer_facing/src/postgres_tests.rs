//! Opt-in tests use a fresh schema per case and never reset a database.

use std::sync::{Arc, Mutex};

use diesel::{
    Connection, PgConnection, RunQueryDsl, connection::SimpleConnection, r2d2::ConnectionManager,
};
use diesel_migrations::MigrationHarness;
use kernel::{
    ApplicationEvent, CommandError, CommandHandlerPort, EventError, EventSubscriberPort,
    KernelConfig, NewEventEnvelope, io::DbPool,
};
use poem::{http::StatusCode, test::TestClient};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use uuid::Uuid;

use crate::{
    bootstrap,
    orchestration::{AppState, start_mulac},
};

struct TestSchema {
    connection: PgConnection,
    name: String,
}

impl Drop for TestSchema {
    fn drop(&mut self) {
        self.connection
            .batch_execute(&format!("DROP SCHEMA {} CASCADE", self.name))
            .expect("remove the isolated test schema");
    }
}

fn database(migrate: bool) -> (TestSchema, DbPool) {
    let database_url = std::env::var("DATABASE_URL_TEST").expect("DATABASE_URL_TEST is required");
    let connection = PgConnection::establish(&database_url)
        .unwrap_or_else(|_| panic!("could not connect to the test database"));
    let mut schema = TestSchema {
        connection,
        name: format!("customer_facing_test_{}", Uuid::new_v4().simple()),
    };
    schema
        .connection
        .batch_execute(&format!("CREATE SCHEMA {}", schema.name))
        .expect("create isolated schema");
    let mut url = url::Url::parse(&database_url)
        .unwrap_or_else(|_| panic!("DATABASE_URL_TEST must be a PostgreSQL URL"));
    url.query_pairs_mut()
        .append_pair("options", &format!("-csearch_path={}", schema.name));
    let pool = DbPool::builder()
        .max_size(3)
        .build(ConnectionManager::<PgConnection>::new(url.to_string()))
        .unwrap_or_else(|_| panic!("could not create the test pool"));
    if migrate {
        bootstrap::migrate(&pool).unwrap();
    }
    (schema, pool)
}

#[derive(diesel::QueryableByName)]
struct Snapshot {
    #[diesel(sql_type = diesel::sql_types::Jsonb)]
    data: Value,
}

fn snapshot(pool: &DbPool, sql: &str) -> Value {
    diesel::sql_query(sql)
        .get_result::<Snapshot>(&mut pool.get().unwrap())
        .unwrap()
        .data
}

fn counts(pool: &DbPool) -> Value {
    snapshot(
        pool,
        "SELECT jsonb_build_object(
        'inbox', (SELECT count(*) FROM inbox_entries),
        'commands', (SELECT count(*) FROM command_entries),
        'events', (SELECT count(*) FROM event_entries)
    ) AS data",
    )
}

async fn run(client: &TestClient<poem::Route>, queue: &str, limit: usize) -> Value {
    let response = client
        .post(format!("/api/{queue}/run"))
        .body_json(&json!({"limit":limit}))
        .send()
        .await;
    response.assert_status_is_ok();
    response.json().await.value().deserialize()
}

#[derive(Clone, Deserialize, Serialize)]
struct Probe {
    value: String,
}

impl ApplicationEvent for Probe {
    fn event_type(&self) -> &'static str {
        "DevelopmentProbeFinished"
    }
}

struct ProbeHandler;

impl CommandHandlerPort<Probe, Probe> for ProbeHandler {
    fn execute(&self, command: Probe) -> Result<Vec<Probe>, CommandError> {
        Ok(vec![command])
    }
}

struct ProbeSubscriber(Arc<Mutex<Vec<String>>>);

impl EventSubscriberPort for ProbeSubscriber {
    fn handle(&self, event: &NewEventEnvelope) -> Result<(), EventError> {
        self.0.lock().unwrap().push(event.payload.clone());
        Ok(())
    }
}

#[tokio::test]
#[ignore = "requires DATABASE_URL_TEST with CREATE SCHEMA permission"]
async fn queues_advance_only_through_their_own_runner_and_preserve_metadata() {
    let (_schema, pool) = database(true);
    let handled = Arc::new(Mutex::new(Vec::new()));
    let kernel = kernel::boot(KernelConfig::default())
        .command_handler("DevelopmentProbe", Arc::new(ProbeHandler))
        .event_subscriber(
            "DevelopmentProbeFinished",
            "probe",
            Arc::new(ProbeSubscriber(handled.clone())),
        )
        .start_persistent(pool.clone(), 0)
        .unwrap();
    let client = TestClient::new(crate::app(AppState::from_kernel(pool.clone(), &kernel)));
    let message_id = Uuid::now_v7();
    let correlation_id = Uuid::now_v7();
    let payload = json!({"value":"unchanged"});
    let input = json!({
        "message_id": message_id, "correlation_id": correlation_id,
        "routing_key":"DevelopmentProbe", "source":"test", "payload":payload,
    });
    let response = client.post("/api/inbox").body_json(&input).send().await;
    response.assert_status(StatusCode::ACCEPTED);
    response.assert_json(json!({"message_id":message_id})).await;
    let original = snapshot(&pool, "SELECT to_jsonb(i) AS data FROM inbox_entries i");
    let mut duplicate = input.clone();
    duplicate["payload"] = json!({"value":"must not overwrite"});
    duplicate["source"] = json!("different");
    client
        .post("/api/inbox")
        .body_json(&duplicate)
        .send()
        .await
        .assert_status(StatusCode::ACCEPTED);
    assert_eq!(
        snapshot(&pool, "SELECT to_jsonb(i) AS data FROM inbox_entries i"),
        original
    );
    assert_eq!(counts(&pool), json!({"inbox":1,"commands":0,"events":0}));
    assert_eq!(
        run(&client, "commands", 10).await,
        json!({"processed":0,"errors":[]})
    );
    assert_eq!(
        run(&client, "inbox", 1).await,
        json!({"processed":1,"errors":[]})
    );
    assert_eq!(counts(&pool), json!({"inbox":1,"commands":1,"events":0}));
    assert!(handled.lock().unwrap().is_empty());
    let command = snapshot(&pool, "SELECT to_jsonb(c) AS data FROM command_entries c");
    assert_eq!(command["command_type"], "DevelopmentProbe");
    assert_eq!(
        serde_json::from_str::<Value>(command["payload"].as_str().unwrap()).unwrap(),
        payload
    );
    assert_eq!(
        command["meta"]["correlation_id"],
        correlation_id.to_string()
    );
    assert_eq!(command["meta"]["causation_id"], message_id.to_string());
    assert_eq!(command["meta"]["source"], "test");
    assert_eq!(
        run(&client, "events", 10).await,
        json!({"processed":0,"errors":[]})
    );
    assert_eq!(
        run(&client, "commands", 1).await,
        json!({"processed":1,"errors":[]})
    );
    assert_eq!(counts(&pool), json!({"inbox":1,"commands":1,"events":1}));
    assert!(handled.lock().unwrap().is_empty());
    let event = snapshot(&pool, "SELECT to_jsonb(e) AS data FROM event_entries e");
    assert_eq!(event["event_type"], "DevelopmentProbeFinished");
    assert_eq!(event["meta"]["correlation_id"], correlation_id.to_string());
    assert_eq!(
        run(&client, "events", 1).await,
        json!({"processed":1,"errors":[]})
    );
    let observed: Vec<Value> = handled
        .lock()
        .unwrap()
        .iter()
        .map(|body| serde_json::from_str(body).unwrap())
        .collect();
    assert_eq!(observed, vec![payload]);
    for queue in ["inbox", "commands", "events"] {
        assert_eq!(
            run(&client, queue, 10).await,
            json!({"processed":0,"errors":[]})
        );
    }
    // A duplicate after processing still does not queue a second command.
    client
        .post("/api/inbox")
        .body_json(&input)
        .send()
        .await
        .assert_status(StatusCode::ACCEPTED);
    assert_eq!(
        run(&client, "inbox", 10).await,
        json!({"processed":0,"errors":[]})
    );
    kernel.shutdown();
}

#[tokio::test]
#[ignore = "requires DATABASE_URL_TEST with CREATE SCHEMA permission"]
async fn batch_limits_and_unknown_command_retries_are_durable() {
    let (_schema, pool) = database(true);
    let (kernel, state) = start_mulac(pool.clone()).unwrap();
    let client = TestClient::new(crate::app(state));
    for value in ["one", "two"] {
        client.post("/api/inbox")
            .body_json(&json!({"routing_key":"UnknownDevelopmentProbe","source":"test","payload":{"value":value}}))
            .send().await.assert_status(StatusCode::ACCEPTED);
    }
    assert_eq!(
        run(&client, "inbox", 1).await,
        json!({"processed":1,"errors":[]})
    );
    assert_eq!(counts(&pool), json!({"inbox":2,"commands":1,"events":0}));
    let failed = run(&client, "commands", 1).await;
    assert!(failed["processed"].is_null());
    assert!(
        failed["errors"][0]
            .as_str()
            .unwrap()
            .contains("UnknownDevelopmentProbe")
    );
    let retry = snapshot(
        &pool,
        "SELECT jsonb_build_object(
        'attempts', attempts, 'reservation_id', reservation_id, 'reserved_at', reserved_at,
        'future', scheduled_at > CURRENT_TIMESTAMP, 'processed_at', processed_at,
        'extra_info', extra_info
    ) AS data FROM command_entries",
    );
    assert_eq!(retry["attempts"], 1);
    assert!(retry["reservation_id"].is_null());
    assert!(retry["reserved_at"].is_null());
    assert!(retry["processed_at"].is_null());
    assert_eq!(retry["future"], true);
    assert!(!retry["extra_info"].is_null());
    assert_eq!(
        run(&client, "commands", 1).await,
        json!({"processed":0,"errors":[]})
    );
    assert_eq!(
        run(&client, "inbox", 1).await,
        json!({"processed":1,"errors":[]})
    );
    assert_eq!(counts(&pool), json!({"inbox":2,"commands":2,"events":0}));
    kernel.shutdown();
}

#[tokio::test]
#[ignore = "requires DATABASE_URL_TEST with CREATE SCHEMA permission"]
async fn unconfigured_outbox_and_health_do_not_touch_queue_rows() {
    let (_schema, pool) = database(true);
    pool.get()
        .unwrap()
        .batch_execute(
            "INSERT INTO outbox_entries(id,payload,meta) VALUES
        ('0191d6c4-8f5e-7a3b-9c1d-2e3f4a5b6c7d','{}','{}')",
        )
        .unwrap();
    let before = snapshot(&pool, "SELECT to_jsonb(o) AS data FROM outbox_entries o");
    let (kernel, state) = start_mulac(pool.clone()).unwrap();
    let client = TestClient::new(crate::app(state));
    client
        .post("/api/outbox/run")
        .body_json(&json!({"limit":1}))
        .send()
        .await
        .assert_status(StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(
        snapshot(&pool, "SELECT to_jsonb(o) AS data FROM outbox_entries o"),
        before
    );
    pool.get()
        .unwrap()
        .batch_execute("DROP TABLE inbox_entries")
        .unwrap();
    let response = client
        .post("/api/inbox")
        .body_json(&json!({"routing_key":"probe","source":"test","payload":{"value":1}}))
        .send()
        .await;
    response.assert_status(StatusCode::INTERNAL_SERVER_ERROR);
    response
        .assert_text("inbox message could not be stored")
        .await;
    let health = client.get("/api/health").send().await;
    health.assert_status_is_ok();
    health.assert_text("ok").await;
    kernel.shutdown();
}

#[test]
#[ignore = "requires DATABASE_URL_TEST with CREATE SCHEMA permission"]
fn migrations_round_trip_and_startup_is_idempotent() {
    let (_schema, pool) = database(true);
    bootstrap::migrate(&pool).unwrap();
    assert_eq!(counts(&pool), json!({"inbox":0,"commands":0,"events":0}));
    pool.get()
        .unwrap()
        .revert_all_migrations(bootstrap::MIGRATIONS)
        .unwrap();
    let tables = snapshot(
        &pool,
        "SELECT to_jsonb(count(*)) AS data FROM information_schema.tables
        WHERE table_schema = current_schema() AND table_name IN
        ('inbox_entries','command_entries','event_entries','outbox_entries')",
    );
    assert_eq!(tables, 0);
    bootstrap::migrate(&pool).unwrap();
    assert_eq!(counts(&pool), json!({"inbox":0,"commands":0,"events":0}));
}

#[test]
#[ignore = "requires DATABASE_URL_TEST with CREATE SCHEMA permission"]
fn migration_failure_aborts_bootstrap_with_a_sanitized_error() {
    let (_schema, pool) = database(false);
    pool.get()
        .unwrap()
        .batch_execute("CREATE TABLE inbox_entries (id INT)")
        .unwrap();
    assert_eq!(
        bootstrap::migrate(&pool).unwrap_err().to_string(),
        "customer facing database migrations failed"
    );
}
