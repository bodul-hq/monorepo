//! Compatibility with rows written using the original Mulac queue schema.
use std::sync::{Arc, Mutex};

use axum::Router;
use diesel::prelude::*;
use diesel::sql_types::{Jsonb, Nullable, Text, Uuid as SqlUuid};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use uuid::Uuid;

use super::tests::run;
use crate::platform::bus::{Bus, Command, Event, Handle};
use crate::platform::state::AppState;
use crate::platform::test_db::{Database, json_query};

#[derive(Serialize, Deserialize)]
struct Ping {
    value: i32,
}
impl Command for Ping {
    const NAME: &'static str = "UpgradePing";
}

#[derive(Serialize, Deserialize)]
struct Pong {
    value: i32,
}
impl Event for Pong {
    const NAME: &'static str = "UpgradePong";
}

#[derive(Serialize, Deserialize)]
struct Finish {
    value: i32,
}
impl Command for Finish {
    const NAME: &'static str = "UpgradeFinish";
}

struct PingHandler;
impl Handle for PingHandler {
    type Command = Ping;
    type Event = Pong;
    type Error = &'static str;

    fn handle(&self, command: Ping) -> Result<Vec<Pong>, Self::Error> {
        if command.value == 0 {
            return Err("fixture failure");
        }
        Ok(vec![Pong {
            value: command.value,
        }])
    }
}

struct FinishHandler(Arc<Mutex<Vec<i32>>>);
impl Handle for FinishHandler {
    type Command = Finish;
    type Event = Pong;
    type Error = &'static str;

    fn handle(&self, command: Finish) -> Result<Vec<Pong>, Self::Error> {
        self.0.lock().unwrap().push(command.value);
        Ok(Vec::new())
    }
}

fn app(db: &Database, finished: Arc<Mutex<Vec<i32>>>) -> Router {
    let mut bus = Bus::default();
    bus.handle(PingHandler);
    bus.handle(FinishHandler(finished));
    bus.on::<Pong, Finish>(|event| Some(Finish { value: event.value }));
    crate::app(AppState {
        bus: Some(bus.start(db.pool.clone()).unwrap()),
        db: Some(db.pool.clone()),
        object_storage: None,
        outbox: None,
    })
}

fn rows(db: &Database, table: &str) -> Vec<Value> {
    json_query(
        &mut db.pool.get().unwrap(),
        &format!(
            "SELECT coalesce(jsonb_agg(to_jsonb(t) ORDER BY id), '[]') AS value FROM {table} t"
        ),
    )
    .as_array()
    .unwrap()
    .clone()
}

// Construct the original persisted wire shape directly, rather than serializing
// through the upgraded queue recorder. Both SQL NULL metadata and populated
// metadata/extra_info must remain readable after the dependency changes.
fn seed_command(db: &Database, payload: &str, meta: Option<Value>) -> Uuid {
    let id = Uuid::now_v7();
    diesel::sql_query(
        "INSERT INTO command_entries \
         (id, command_type, payload, meta, status, attempts, scheduled_at, received_at, updated_at, extra_info) \
         VALUES ($1, 'UpgradePing', $2, $3, 4, 2, now() - interval '1 day', \
         '2026-10-01T00:00:00Z', '2026-10-01T00:00:00Z', '{\"errors\":[\"earlier failure\"]}')",
    )
    .bind::<SqlUuid, _>(id)
    .bind::<Text, _>(payload)
    .bind::<Nullable<Jsonb>, _>(meta)
    .execute(&mut db.pool.get().unwrap()).unwrap();
    id
}

fn seed_event(db: &Database, payload: &str, meta: Option<Value>) -> Uuid {
    let id = Uuid::now_v7();
    diesel::sql_query(
        "INSERT INTO event_entries \
         (id, event_type, payload, meta, status, attempts, scheduled_at, received_at, updated_at, extra_info) \
         VALUES ($1, 'UpgradePong', $2, $3, 4, 2, now() - interval '1 day', \
         '2026-10-01T00:00:00Z', '2026-10-01T00:00:00Z', '{\"errors\":[\"earlier failure\"]}')",
    )
    .bind::<SqlUuid, _>(id)
    .bind::<Text, _>(payload)
    .bind::<Nullable<Jsonb>, _>(meta)
    .execute(&mut db.pool.get().unwrap()).unwrap();
    id
}

fn assert_completed(before: &Value, after: &Value) {
    for field in [
        "id",
        "command_type",
        "event_type",
        "payload",
        "meta",
        "received_at",
        "scheduled_at",
        "extra_info",
    ] {
        assert_eq!(after[field], before[field], "{field}");
    }
    assert_eq!(after["status"], 5);
    assert_eq!(after["attempts"], 3);
    assert!(after["reservation_id"].is_null());
    assert!(after["reserved_at"].is_null());
    assert!(after["processed_at"].is_string());
}

#[tokio::test]
#[ignore = "requires DATABASE_URL_TEST; run make test-postgres"]
async fn old_command_rows_advance_only_through_separate_queue_runs() {
    let db = Database::new();
    let command_id = Uuid::now_v7();
    let correlation_id = Uuid::now_v7();
    let metadata = json!({
        "command_id":command_id, "correlation_id":correlation_id,
        "causation_id":null, "source":"legacy-fixture"
    });
    seed_command(&db, r#"{"value":7}"#, Some(metadata));
    let original = rows(&db, "command_entries");
    let finished = Arc::new(Mutex::new(Vec::new()));
    let app = app(&db, finished.clone());
    assert_eq!(rows(&db, "command_entries"), original);
    assert!(rows(&db, "event_entries").is_empty());
    assert_eq!(run(&app, "/api/events/run", 1).await["processed"], 0);

    assert_eq!(run(&app, "/api/commands/run", 1).await["processed"], 1);
    assert_completed(&original[0], &rows(&db, "command_entries")[0]);
    let events = rows(&db, "event_entries");
    assert_eq!(events.len(), 1);
    assert_eq!(events[0]["status"], 0);
    assert_eq!(events[0]["meta"]["causation_id"], command_id.to_string());
    assert_eq!(
        events[0]["meta"]["correlation_id"],
        correlation_id.to_string()
    );
    assert_eq!(
        serde_json::from_str::<Value>(events[0]["payload"].as_str().unwrap()).unwrap(),
        json!({"value":7})
    );
    assert!(finished.lock().unwrap().is_empty());
    assert_eq!(run(&app, "/api/commands/run", 1).await["processed"], 0);

    assert_eq!(run(&app, "/api/events/run", 1).await["processed"], 1);
    let commands = rows(&db, "command_entries");
    assert_eq!(commands.len(), 2);
    let next = commands
        .iter()
        .find(|row| row["command_type"] == "UpgradeFinish")
        .unwrap();
    assert_eq!(next["status"], 0);
    assert_eq!(next["meta"]["correlation_id"], correlation_id.to_string());
    assert_eq!(next["meta"]["causation_id"], events[0]["meta"]["event_id"]);
    assert_eq!(next["meta"]["source"], "event:UpgradePong");
    assert!(finished.lock().unwrap().is_empty());
    assert_eq!(run(&app, "/api/events/run", 1).await["processed"], 0);
    assert_eq!(run(&app, "/api/commands/run", 1).await["processed"], 1);
    assert_eq!(*finished.lock().unwrap(), [7]);
}

#[tokio::test]
#[ignore = "requires DATABASE_URL_TEST; run make test-postgres"]
async fn legacy_null_metadata_and_event_metadata_remain_compatible() {
    let db = Database::new();
    seed_command(&db, r#"{"value":8}"#, None);
    let command = rows(&db, "command_entries").remove(0);
    seed_event(&db, r#"{"value":9}"#, None);
    let event_id = Uuid::now_v7();
    let correlation_id = Uuid::now_v7();
    seed_event(
        &db,
        r#"{"value":10}"#,
        Some(json!({
            "event_id":event_id, "correlation_id":correlation_id,
            "causation_id":null, "source":"legacy-fixture"
        })),
    );
    let original_events = rows(&db, "event_entries");
    let app = app(&db, Arc::new(Mutex::new(Vec::new())));

    // One request reserves only one legacy event, even with more due rows.
    assert_eq!(run(&app, "/api/events/run", 1).await["processed"], 1);
    assert_eq!(
        rows(&db, "event_entries")
            .iter()
            .filter(|row| row["status"] == 5)
            .count(),
        1
    );
    assert_eq!(run(&app, "/api/events/run", 1).await["processed"], 1);
    for (before, after) in original_events.iter().zip(rows(&db, "event_entries")) {
        assert_completed(before, &after);
    }
    let commands = rows(&db, "command_entries");
    let children: Vec<_> = commands
        .iter()
        .filter(|row| row["command_type"] == "UpgradeFinish")
        .collect();
    assert_eq!(children.len(), 2);
    let null_child = children
        .iter()
        .find(|row| row["payload"] == r#"{"value":9}"#)
        .unwrap();
    assert!(null_child["meta"]["causation_id"].is_null());
    assert!(null_child["meta"]["correlation_id"].is_null());
    let linked_child = children
        .iter()
        .find(|row| row["payload"] == r#"{"value":10}"#)
        .unwrap();
    assert_eq!(linked_child["meta"]["causation_id"], event_id.to_string());
    assert_eq!(
        linked_child["meta"]["correlation_id"],
        correlation_id.to_string()
    );

    assert_eq!(run(&app, "/api/commands/run", 1).await["processed"], 1);
    let after = rows(&db, "command_entries")
        .into_iter()
        .find(|row| row["id"] == command["id"])
        .unwrap();
    assert_completed(&command, &after);
}

#[tokio::test]
#[ignore = "requires DATABASE_URL_TEST; run make test-postgres"]
async fn partial_batches_complete_successes_and_reschedule_failures() {
    let db = Database::new();
    let failing_command = seed_command(&db, r#"{"value":0}"#, None);
    seed_command(&db, r#"{"value":1}"#, None);
    // A failed subscriber deserialization exercises the event retry path.
    let failing_event = seed_event(&db, r#"{"value":"invalid"}"#, None);
    let app = app(&db, Arc::new(Mutex::new(Vec::new())));
    for path in ["/api/commands/run", "/api/events/run"] {
        let report = run(&app, path, 10).await;
        assert!(report["processed"].is_null());
        assert_eq!(report["errors"].as_array().unwrap().len(), 1);
    }
    for (table, failed_id) in [
        ("command_entries", failing_command),
        ("event_entries", failing_event),
    ] {
        let rows = rows(&db, table);
        assert!(rows.iter().any(|row| row["status"] == 5));
        let failed = rows
            .iter()
            .find(|row| row["id"] == failed_id.to_string())
            .unwrap();
        assert_eq!(failed["status"], 4);
        assert_eq!(failed["attempts"], 3);
        assert!(failed["reservation_id"].is_null());
        assert!(failed["reserved_at"].is_null());
        assert!(failed["processed_at"].is_null());
        assert_eq!(failed["extra_info"]["errors"][0], "earlier failure");
        assert_eq!(failed["extra_info"]["errors"].as_array().unwrap().len(), 2);
        let future = json_query(
            &mut db.pool.get().unwrap(),
            &format!(
                "SELECT to_jsonb(scheduled_at > now()) AS value FROM {table} WHERE id = '{failed_id}'"
            ),
        );
        assert_eq!(future, true);
    }
}
