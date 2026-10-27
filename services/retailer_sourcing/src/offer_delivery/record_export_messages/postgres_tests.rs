use std::sync::{Arc, Barrier};

use diesel::connection::SimpleConnection;
use diesel::prelude::*;
use diesel::sql_types::{Jsonb, Nullable, Text, Uuid as SqlUuid};
use serde_json::{Value, json};
use tower::ServiceExt;
use uuid::Uuid;

use super::listener::{Listener, ListenerError, StoreError};
use super::store::PgStore;
use crate::offer_discovery::prelude::export_testing::{
    ANKER, ExportSource, completed, event, pending, stored_source,
};
use crate::offer_discovery::prelude::{OfferExport, OfferExportWasPrepared};
use crate::platform::bus::{Bus, Event, EventContext, Listen};
use crate::platform::outbox::{self, OutgoingMessage, RecordError};
use crate::platform::state::AppState;
use crate::platform::test_db::{Database, json_query};
use crate::platform::test_objects::Server;
use crate::platform::{Platform, http};

const IMPORT_FIXTURE: &str =
    include_str!("../../../tests/fixtures/offer_export/import_payload.json");
/// One priced variation, one without a price and one priced but unavailable.
const MIXED: &str = r#"{"variations":[
    {"title":"Charger","url":"https://shop.example/Charger?variant=1","price":{"amount":1999,"currency":"AUD"}},
    {"title":"Charger","url":"https://shop.example/Charger?variant=2","price":null},
    {"title":"Charger","url":"https://shop.example/Charger?variant=3","price":{"amount":2999,"currency":"AUD"},"available":false}
]}"#;
const EMPTY: &str = r#"{"variations":[]}"#;

fn outbox_rows(db: &Database) -> Value {
    json_query(
        &mut db.pool.get().unwrap(),
        "SELECT coalesce(jsonb_agg(to_jsonb(t) ORDER BY id), '[]') AS value FROM outbox_entries t",
    )
}

fn event_rows(db: &Database) -> Value {
    json_query(
        &mut db.pool.get().unwrap(),
        "SELECT coalesce(jsonb_agg(to_jsonb(t) ORDER BY id), '[]') AS value FROM event_entries t",
    )
}

fn context() -> EventContext {
    EventContext {
        event_id: Uuid::now_v7(),
        correlation_id: Some(Uuid::now_v7()),
        causation_id: Some(Uuid::now_v7()),
        source: Some("event:OfferWasStructuralized".into()),
    }
}

fn listener(db: &Database, server: &Server) -> Listener<PgStore> {
    Listener::new(PgStore::new(db.pool.clone(), server.client()))
}

fn export(db: &Database, server: &Server, document: &str) -> OfferExport {
    completed(
        &db.pool,
        server.client(),
        stored_source(&server.client(), document),
    )
}

/// Queues `payload` as an event, as Mulac stores one a handler produced.
fn seed_event(db: &Database, event_type: &str, payload: &Value, meta: Option<Value>) -> Uuid {
    let id = Uuid::now_v7();
    diesel::sql_query(
        "INSERT INTO event_entries (id, event_type, payload, meta, status) VALUES ($1, $2, $3, $4, 0)",
    )
    .bind::<SqlUuid, _>(id)
    .bind::<Text, _>(event_type)
    .bind::<Text, _>(payload.to_string())
    .bind::<Nullable<Jsonb>, _>(meta)
    .execute(&mut db.pool.get().unwrap())
    .unwrap();
    id
}

fn meta(context: &EventContext) -> Value {
    json!({
        "event_id": context.event_id,
        "correlation_id": context.correlation_id,
        "causation_id": context.causation_id,
        "source": context.source,
    })
}

fn seed_export_event(db: &Database, export: &OfferExport, context: &EventContext) -> Uuid {
    seed_event(
        db,
        OfferExportWasPrepared::NAME,
        &serde_json::to_value(event(export)).unwrap(),
        Some(meta(context)),
    )
}

/// The service's routes over a bus with only this slice registered.
fn app(db: &Database, server: &Server) -> axum::Router {
    let platform = Platform {
        db: db.pool.clone(),
        http: http::Client,
        object_storage: server.client(),
    };
    let mut bus = Bus::default();
    super::prelude::register(&mut bus, &platform);
    let entered = server.runtime.enter();
    let bus = bus.start(db.pool.clone()).unwrap();
    drop(entered);
    crate::app(AppState {
        bus: Some(bus),
        db: Some(db.pool.clone()),
        object_storage: Some(server.client()),
        outbox: None,
    })
}

fn run_events(server: &Server, app: &axum::Router) -> Value {
    let request = axum::http::Request::post("/api/events/run")
        .header("content-type", "application/json")
        .body(axum::body::Body::from(r#"{"limit":10}"#))
        .unwrap();
    server.runtime.block_on(async {
        let response = app.clone().oneshot(request).await.unwrap();
        assert_eq!(response.status(), axum::http::StatusCode::OK);
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        serde_json::from_slice(&body).unwrap()
    })
}

/// Makes failed events due again instead of waiting for their backoff.
fn retry_now(db: &Database) {
    db.pool
        .get()
        .unwrap()
        .batch_execute("UPDATE event_entries SET scheduled_at = now() WHERE status = 4")
        .unwrap();
}

#[test]
#[ignore = "requires DATABASE_URL_TEST; run make test-postgres"]
fn postgres_queued_event_records_each_entry_once_with_its_envelope_metadata() {
    let db = Database::new();
    let server = Server::new();
    let export = export(&db, &server, MIXED);
    assert_eq!(export.entries.len(), 2);
    assert_eq!(export.exclusions.len(), 1);
    let app = app(&db, &server);
    let first = context();
    seed_export_event(&db, &export, &first);
    seed_event(
        &db,
        "SomethingUnrelated",
        &json!({"export_id": export.id}),
        Some(meta(&context())),
    );

    assert_eq!(
        run_events(&server, &app),
        json!({"processed":2,"errors":[]})
    );

    let rows = outbox_rows(&db);
    let rows = rows.as_array().unwrap();
    assert_eq!(rows.len(), 2);
    let mut entries = export.entries.clone();
    entries.sort_by_key(|entry| entry.id);
    for (row, entry) in rows.iter().zip(&entries) {
        assert_eq!(row["id"], json!(entry.id));
        assert_eq!(row["status"], 0);
        assert_eq!(row["attempts"], 0);
        assert!(row["reservation_id"].is_null());
        assert_eq!(
            row["payload"],
            json!(serde_json::to_string(&entry.payload).unwrap())
        );
        assert_eq!(
            row["meta"],
            json!({
                "event_id": first.event_id,
                "message_id": entry.id,
                "correlation_id": first.correlation_id,
                "causation_id": first.causation_id,
                "event_type": "OfferExportWasPrepared",
                "routing_key": "offers",
                "source": first.source,
                "content_type": "application/json",
            })
        );
    }

    // A repeated notification with a new envelope adds no delivery identities.
    let before = outbox_rows(&db);
    seed_export_event(&db, &export, &context());
    assert_eq!(
        run_events(&server, &app),
        json!({"processed":1,"errors":[]})
    );
    assert_eq!(outbox_rows(&db), before);
}

#[test]
#[ignore = "requires DATABASE_URL_TEST; run make test-postgres"]
fn postgres_empty_and_wholly_excluded_exports_succeed_without_messages() {
    let db = Database::new();
    let server = Server::new();
    let listener = listener(&db, &server);
    let excluded =
        r#"{"variations":[{"title":"Charger","url":"https://shop.example/Charger","price":null}]}"#;
    for document in [EMPTY, excluded] {
        let export = export(&db, &server, document);
        assert!(export.entries.is_empty());
        listener.listen(event(&export), &context()).unwrap();
    }
    assert_eq!(outbox_rows(&db), json!([]));
}

#[test]
#[ignore = "requires DATABASE_URL_TEST; run make test-postgres"]
fn postgres_retry_after_partial_recording_completes_the_set_and_keeps_delivered_rows() {
    let db = Database::new();
    let server = Server::new();
    let export = export(&db, &server, MIXED);
    let first = context();
    let delivered = &export.entries[0];
    outbox::record(
        &mut db.pool.get().unwrap(),
        &[OutgoingMessage {
            id: delivered.id,
            payload: serde_json::to_string(&delivered.payload).unwrap(),
            routing_key: "offers",
            event_type: OfferExportWasPrepared::NAME,
        }],
        &first,
    )
    .unwrap();
    // As if the publisher delivered it after one failed attempt.
    diesel::sql_query(
        "UPDATE outbox_entries SET status = 5, attempts = 2, reservation_id = $2,
         reserved_at = now(), processed_at = now(), last_error = 'earlier failure'
         WHERE id = $1",
    )
    .bind::<SqlUuid, _>(delivered.id)
    .bind::<SqlUuid, _>(Uuid::now_v7())
    .execute(&mut db.pool.get().unwrap())
    .unwrap();
    let before = outbox_rows(&db)[0].clone();

    listener(&db, &server)
        .listen(event(&export), &context())
        .unwrap();

    let rows = outbox_rows(&db);
    let rows = rows.as_array().unwrap();
    assert_eq!(rows.len(), 2);
    let kept = rows
        .iter()
        .find(|row| row["id"] == json!(delivered.id))
        .unwrap();
    assert_eq!(kept, &before);
    assert_eq!(kept["meta"]["event_id"], json!(first.event_id));
    let added = rows
        .iter()
        .find(|row| row["id"] == json!(export.entries[1].id))
        .unwrap();
    assert_eq!(added["status"], 0);
}

#[test]
#[ignore = "requires DATABASE_URL_TEST; run make test-postgres"]
fn postgres_conflicting_entry_identity_fails_and_records_nothing() {
    let db = Database::new();
    let server = Server::new();
    let export = export(&db, &server, MIXED);
    let reused = &export.entries[1];
    let mut payload = reused.payload.clone();
    payload.title = "Something else".into();
    outbox::record(
        &mut db.pool.get().unwrap(),
        &[OutgoingMessage {
            id: reused.id,
            payload: serde_json::to_string(&payload).unwrap(),
            routing_key: "offers",
            event_type: OfferExportWasPrepared::NAME,
        }],
        &context(),
    )
    .unwrap();
    let before = outbox_rows(&db);

    let error = listener(&db, &server)
        .listen(event(&export), &context())
        .unwrap_err();

    assert!(
        matches!(
            &error,
            ListenerError::Store(StoreError::Record(RecordError::Conflict(id))) if *id == reused.id
        ),
        "{error}"
    );
    // The transaction rolled back the other entry's insert too.
    assert_eq!(outbox_rows(&db), before);

    let message = OutgoingMessage {
        id: Uuid::now_v7(),
        payload: "{}".into(),
        routing_key: "offers",
        event_type: OfferExportWasPrepared::NAME,
    };
    assert!(matches!(
        outbox::record(
            &mut db.pool.get().unwrap(),
            &[message.clone(), message],
            &context()
        ),
        Err(RecordError::Duplicate(_))
    ));
    assert_eq!(outbox_rows(&db), before);
}

#[test]
#[ignore = "requires DATABASE_URL_TEST; run make test-postgres"]
fn postgres_concurrent_notifications_record_one_row_per_entry() {
    let db = Database::new();
    let server = Server::new();
    let export = export(&db, &server, MIXED);
    let listener = Arc::new(listener(&db, &server));
    let barrier = Arc::new(Barrier::new(2));
    let contexts = [context(), context()];
    std::thread::scope(|scope| {
        for context in &contexts {
            let (listener, barrier, event) = (listener.clone(), barrier.clone(), event(&export));
            scope.spawn(move || {
                barrier.wait();
                listener.listen(event, context).unwrap();
            });
        }
    });

    let rows = outbox_rows(&db);
    let rows = rows.as_array().unwrap();
    assert_eq!(rows.len(), export.entries.len());
    let winner = &rows[0]["meta"]["event_id"];
    assert!(
        contexts
            .iter()
            .any(|context| *winner == json!(context.event_id))
    );
    assert!(rows.iter().all(|row| &row["meta"]["event_id"] == winner));
}

#[test]
#[ignore = "requires DATABASE_URL_TEST; run make test-postgres"]
fn postgres_delayed_event_uses_its_own_snapshot_after_newer_source_data() {
    let db = Database::new();
    let server = Server::new();
    let older = stored_source(&server.client(), ANKER);
    let older_export = completed(&db.pool, server.client(), older.clone());
    let newer = ExportSource {
        storage: stored_source(&server.client(), MIXED).storage,
        ..older
    };
    let newer_export = completed(&db.pool, server.client(), newer);
    assert_ne!(older_export.id, newer_export.id);

    listener(&db, &server)
        .listen(event(&older_export), &context())
        .unwrap();

    let mut expected: Vec<_> = older_export
        .entries
        .iter()
        .map(|entry| json!(entry.id))
        .collect();
    expected.sort_by_key(|id| id.as_str().unwrap().to_string());
    let rows = outbox_rows(&db);
    let ids: Vec<_> = rows
        .as_array()
        .unwrap()
        .iter()
        .map(|row| row["id"].clone())
        .collect();
    assert_eq!(ids, expected);
}

#[test]
#[ignore = "requires DATABASE_URL_TEST; run make test-postgres"]
fn postgres_unavailable_or_invalid_exports_fail_visibly_and_retry() {
    let db = Database::new();
    let server = Server::new();
    let app = app(&db, &server);
    let missing = OfferExportWasPrepared {
        export_id: Uuid::now_v7(),
        ..event(&export(&db, &server, EMPTY))
    };
    let pending = pending(
        &db.pool,
        server.client(),
        stored_source(&server.client(), ANKER),
    );
    let stored = export(&db, &server, ANKER);
    let mut other_source = event(&stored);
    other_source.source.url = "https://shop.example/Other".into();
    let mut unsupported = event(&stored);
    unsupported.schema_version += 1;
    for payload in [missing, event(&pending), other_source, unsupported] {
        seed_event(
            &db,
            OfferExportWasPrepared::NAME,
            &serde_json::to_value(payload).unwrap(),
            Some(meta(&context())),
        );
    }
    seed_event(
        &db,
        OfferExportWasPrepared::NAME,
        &serde_json::to_value(event(&stored)).unwrap(),
        None,
    );

    let report = run_events(&server, &app);
    assert!(report["processed"].is_null(), "{report}");
    let errors = report["errors"].to_string();
    assert_eq!(report["errors"].as_array().unwrap().len(), 5, "{errors}");
    for expected in [
        "is not completed",
        "does not match",
        "unsupported schema version",
        "no envelope metadata",
    ] {
        assert!(errors.contains(expected), "{expected}: {errors}");
    }
    let events = event_rows(&db);
    assert!(
        events
            .as_array()
            .unwrap()
            .iter()
            .all(|row| row["status"] == 4 && row["attempts"] == 1)
    );

    // Object storage fails, then returns a malformed snapshot, then recovers.
    let event_id = seed_export_event(&db, &stored, &context());
    server.data.lock().unwrap().get_response = Some((503, "unavailable".into()));
    let report = run_events(&server, &app);
    assert!(report["processed"].is_null(), "{report}");
    server.data.lock().unwrap().get_response = Some((200, r#"{"not":"an export"}"#.into()));
    retry_now(&db);
    assert!(run_events(&server, &app)["processed"].is_null());
    assert_eq!(outbox_rows(&db), json!([]));

    server.data.lock().unwrap().get_response = None;
    db.pool
        .get()
        .unwrap()
        .batch_execute(&format!(
            "UPDATE event_entries SET scheduled_at = now() WHERE id = '{event_id}'"
        ))
        .unwrap();
    run_events(&server, &app);
    let row = event_rows(&db)
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["id"] == json!(event_id))
        .cloned()
        .unwrap();
    assert_eq!(row["status"], 5, "{row}");
    assert_eq!(
        outbox_rows(&db).as_array().unwrap().len(),
        stored.entries.len()
    );
}

#[test]
#[ignore = "requires DATABASE_URL_TEST; run make test-postgres"]
fn postgres_recorded_messages_are_accepted_by_the_offers_inbox_and_not_reset_by_replay() {
    use crate::platform::outbox_http::{postgres_tests as http, tests::run};

    let db = Database::new();
    let receiver = Database::new();
    let server = Server::new();
    let export = export(&db, &server, ANKER);
    let first = context();
    let listener = listener(&db, &server);
    listener.listen(event(&export), &first).unwrap();

    // Recording is complete here; HTTP acceptance is the separate publisher's.
    let inbox = server.runtime.block_on(http::inbox(&receiver));
    let state = {
        let _entered = server.runtime.enter();
        http::state(&db, &inbox.url)
    };
    assert_eq!(
        server
            .runtime
            .block_on(run(state.clone(), json!({"limit":10})))
            .1,
        json!({"processed":2,"errors":[]})
    );
    let delivered = outbox_rows(&db);
    assert!(
        delivered
            .as_array()
            .unwrap()
            .iter()
            .all(|row| row["status"] == 5)
    );

    let accepted = json_query(
        &mut receiver.pool.get().unwrap(),
        "SELECT jsonb_agg(to_jsonb(t) ORDER BY id) AS value FROM inbox_entries t",
    );
    assert_eq!(accepted.as_array().unwrap().len(), 2);
    let fixture: Value = serde_json::from_str(IMPORT_FIXTURE).unwrap();
    let first_entry = &export.entries[0];
    let row = accepted
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["id"] == json!(first_entry.id))
        .unwrap();
    assert_eq!(
        serde_json::from_str::<Value>(row["payload"].as_str().unwrap()).unwrap(),
        fixture
    );
    assert_eq!(row["meta"]["routing_key"], "ImportOfferDetails");
    assert_eq!(row["meta"]["correlation_id"], json!(first.correlation_id));
    assert_eq!(row["meta"]["source"], json!(first.source));
    // Offers processing needs explicit inbox, command and event runs there.
    assert_eq!(
        json_query(
            &mut receiver.pool.get().unwrap(),
            "SELECT to_jsonb(count(*)) AS value FROM command_entries"
        ),
        0
    );

    listener.listen(event(&export), &context()).unwrap();
    assert_eq!(outbox_rows(&db), delivered);
    assert_eq!(
        server.runtime.block_on(run(state, json!({"limit":10}))).1,
        json!({"processed":0,"errors":[]})
    );
}
