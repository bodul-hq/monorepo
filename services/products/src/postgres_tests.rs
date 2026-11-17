use crate::app;
use crate::start_mulac;
use diesel::sql_query;
use std::env::var;
use url::Url;
use uuid::Uuid;

use crate::{
    bootstrap::MIGRATIONS,
    infra::{DbPool, PostgresEventStore},
};
use diesel::{
    Connection, PgConnection, RunQueryDsl, connection::SimpleConnection, r2d2::ConnectionManager,
};
use diesel_migrations::MigrationHarness;
use poem::{http::StatusCode, test::TestClient};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use testimony::prelude::{EventEnvelope, EventStore, HasAggregateId};

struct TestSchema {
    connection: PgConnection,
    name: String,
}
impl Drop for TestSchema {
    fn drop(&mut self) {
        self.connection
            .batch_execute(&format!("DROP SCHEMA {} CASCADE", self.name))
            .unwrap();
    }
}
fn database() -> (TestSchema, DbPool) {
    let url = var("DATABASE_URL_TEST").expect("DATABASE_URL_TEST");
    let mut schema = TestSchema {
        connection: PgConnection::establish(&url).unwrap(),
        name: format!("foundation_{}", Uuid::new_v4().simple()),
    };
    schema
        .connection
        .batch_execute(&format!("CREATE SCHEMA {}", schema.name))
        .unwrap();
    let mut url = Url::parse(&url).unwrap();
    url.query_pairs_mut()
        .append_pair("options", &format!("-csearch_path={}", schema.name));
    let pool = DbPool::builder()
        .max_size(2)
        .build(ConnectionManager::<PgConnection>::new(url.to_string()))
        .unwrap();
    pool.get()
        .unwrap()
        .run_pending_migrations(MIGRATIONS)
        .unwrap();
    (schema, pool)
}
#[derive(diesel::QueryableByName)]
struct Snapshot {
    #[diesel(sql_type=diesel::sql_types::Jsonb)]
    data: Value,
}
fn snapshot(pool: &DbPool, sql: &str) -> Value {
    sql_query(sql)
        .get_result::<Snapshot>(&mut pool.get().unwrap())
        .unwrap()
        .data
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct Fixture {
    id: String,
    value: String,
}
impl HasAggregateId for Fixture {
    fn aggregate_id(&self) -> String {
        self.id.clone()
    }
}
impl From<&Fixture> for Fixture {
    fn from(value: &Fixture) -> Self {
        value.clone()
    }
}

#[test]
#[ignore = "requires DATABASE_URL_TEST"]
fn migrations_and_testimony_round_trip() {
    let (_schema, pool) = database();
    let mut conn = pool.get().unwrap();
    conn.revert_all_migrations(MIGRATIONS).unwrap();
    assert_eq!(
        snapshot(
            &pool,
            "SELECT coalesce(to_jsonb(to_regclass('event_store')), 'null'::jsonb) AS data"
        ),
        Value::Null
    );
    conn.run_pending_migrations(MIGRATIONS).unwrap();
    drop(conn);
    let event = Fixture {
        id: "fixture".into(),
        value: "persisted".into(),
    };
    let envelope = EventEnvelope::new(event.clone(), 1);
    let timestamp = envelope.recorded_at();
    let store = PostgresEventStore::<String, Fixture, Fixture>::new(pool.clone());
    store.append(vec![envelope]).unwrap();
    let loaded = store.load("fixture".into()).unwrap();
    assert_eq!(loaded[0].event(), &event);
    assert_eq!(loaded[0].sequence(), 1);
    assert_eq!(loaded[0].recorded_at(), timestamp);
    assert!(store.append(vec![EventEnvelope::new(event, 1)]).is_err());
    assert_eq!(store.load("fixture".into()).unwrap().len(), 1);
    assert!(snapshot(&pool, "SELECT metadata AS data FROM event_store").is_object());
}

#[tokio::test]
#[ignore = "requires DATABASE_URL_TEST"]
async fn durable_inbox_queues_and_reporting() {
    let (_schema, pool) = database();
    let (kernel, state) = start_mulac(pool.clone()).unwrap();
    let client = TestClient::new(app(state, "test".into()));
    for queue in ["inbox", "commands", "events"] {
        client
            .post(format!("/api/{queue}/run"))
            .body_json(&json!({"limit":10}))
            .send()
            .await
            .assert_json(json!({"processed":0,"errors":[]}))
            .await;
    }
    let id = Uuid::new_v4();
    let request = json!({"message_id":id,"payload":{"value":1},"routing_key":"UnknownCommand","source":"test"});
    client
        .post("/api/inbox")
        .body_json(&request)
        .send()
        .await
        .assert_status(StatusCode::ACCEPTED);
    let before = snapshot(&pool, "SELECT to_jsonb(i) AS data FROM inbox_entries i");
    let mut duplicate = request.clone();
    duplicate["payload"] = json!({"value":2});
    client
        .post("/api/inbox")
        .body_json(&duplicate)
        .send()
        .await
        .assert_status(StatusCode::ACCEPTED);
    assert_eq!(
        before,
        snapshot(&pool, "SELECT to_jsonb(i) AS data FROM inbox_entries i")
    );
    kernel.shutdown();
    let (kernel, state) = start_mulac(pool.clone()).unwrap();
    let client = TestClient::new(app(state, "test".into()));
    client
        .post("/api/inbox/run")
        .body_json(&json!({"limit":10}))
        .send()
        .await
        .assert_json(json!({"processed":1,"errors":[]}))
        .await;
    let response = client
        .post("/api/commands/run")
        .body_json(&json!({"limit":10}))
        .send()
        .await;
    response.assert_status_is_ok();
    let body: Value = response.json().await.value().deserialize();
    assert_eq!(body["errors"].as_array().unwrap().len(), 1);
    sql_query("INSERT INTO outbox_entries(id,payload,meta) VALUES (gen_random_uuid(),'{}','{}')")
        .execute(&mut pool.get().unwrap())
        .unwrap();
    let before = snapshot(
        &pool,
        "SELECT coalesce(jsonb_agg(to_jsonb(o)), '[]') AS data FROM outbox_entries o",
    );
    client
        .post("/api/outbox/run")
        .body_json(&json!({"limit":10}))
        .send()
        .await
        .assert_status(StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(
        before,
        snapshot(
            &pool,
            "SELECT coalesce(jsonb_agg(to_jsonb(o)), '[]') AS data FROM outbox_entries o"
        )
    );
    for path in ["database-tables", "entry-statuses"] {
        let response = client
            .get(format!("/api/reporting/{path}"))
            .header("Authorization", "Bearer test")
            .send()
            .await;
        response.assert_status_is_ok();
        assert_eq!(response.0.headers()["cache-control"], "no-store");
    }
    kernel.shutdown();
}
