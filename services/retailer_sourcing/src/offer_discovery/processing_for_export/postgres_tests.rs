use diesel::connection::SimpleConnection;
use diesel::prelude::*;
use diesel::sql_types::{Jsonb, Uuid as SqlUuid};
use diesel_migrations::{EmbeddedMigrations, MigrationHarness, embed_migrations};
use serde_json::json;
use uuid::Uuid;

use super::conversion::tests::{ANKER, source};
use super::entity::{ExportSource, OfferExport};
use super::handler::Store;
use super::store::PgStore;
use crate::platform::db::Json;
use crate::platform::object_storage::{
    OFFER_EXPORT_BUCKET, ObjectStorage, STRUCTURALIZED_OFFERS_BUCKET,
};
use crate::platform::test_db::{Database, json_query};
use crate::platform::test_objects::Server;

const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

fn stored_source(objects: &ObjectStorage) -> ExportSource {
    let mut source = source();
    source.storage = objects
        .create_offer(STRUCTURALIZED_OFFERS_BUCKET, ANKER)
        .unwrap();
    source
}

fn rows(db: &Database) -> serde_json::Value {
    json_query(
        &mut db.pool.get().unwrap(),
        "SELECT coalesce(jsonb_agg(to_jsonb(e)), '[]') AS value FROM offer_export e",
    )
}

pub(super) fn verify_round_trip(db: &Database, objects: ObjectStorage) {
    let source = stored_source(&objects);
    let store = PgStore::new(db.pool.clone(), objects);
    let prepared = store.prepare(source.clone()).unwrap();
    let id = prepared.snapshot.id;
    let snapshot = prepared.snapshot.clone();
    assert!(store.load(id).unwrap().is_none());
    assert_eq!(rows(db)[0]["status"], "prepared");
    let completed = store.complete(prepared).unwrap();
    assert_eq!(completed, snapshot);
    assert_eq!(store.load(id).unwrap(), Some(completed.clone()));
    assert_eq!(
        store.complete(store.prepare(source).unwrap()).unwrap(),
        completed
    );
    assert!(store.load(Uuid::now_v7()).unwrap().is_none());
    let rows = rows(db);
    assert_eq!(rows.as_array().unwrap().len(), 1);
    assert_eq!(rows[0]["status"], "completed");
    assert!(rows[0]["prepared_body"].is_null());
    assert!(rows[0]["completed_at"].is_string());
}

#[test]
#[ignore = "requires DATABASE_URL_TEST; run make test-postgres"]
fn postgres_export_round_trip_and_repeat_keep_snapshot_and_entry_ids() {
    let server = Server::new();
    verify_round_trip(&Database::new(), server.client());
    let data = server.data.lock().unwrap();
    assert_eq!(
        data.uploads
            .iter()
            .filter(|(bucket, _)| bucket == OFFER_EXPORT_BUCKET)
            .count(),
        1
    );
}

#[test]
#[ignore = "requires DATABASE_URL_TEST; run make test-postgres"]
fn postgres_upload_then_failed_completion_recovers_exact_bytes_after_restart() {
    let db = Database::new();
    let server = Server::new();
    let source = stored_source(&server.client());
    let store = PgStore::new(db.pool.clone(), server.client());
    let prepared = store.prepare(source.clone()).unwrap();
    let expected = prepared.snapshot.clone();
    let pending = rows(&db);
    db.pool.get().unwrap().batch_execute(
        "ALTER TABLE offer_export ADD CONSTRAINT fail_completion CHECK (status <> 'completed') NOT VALID"
    ).unwrap();
    assert!(store.complete(prepared).is_err());
    assert_eq!(rows(&db), pending);
    db.pool
        .get()
        .unwrap()
        .batch_execute("ALTER TABLE offer_export DROP CONSTRAINT fail_completion")
        .unwrap();
    drop(store);
    let restarted = PgStore::new(db.pool.clone(), server.client());
    let retry = restarted.prepare(source).unwrap();
    assert_eq!(retry.snapshot, expected);
    assert_eq!(restarted.complete(retry).unwrap(), expected);
    let data = server.data.lock().unwrap();
    let uploads: Vec<_> = data
        .uploads
        .iter()
        .filter(|(bucket, _)| bucket == OFFER_EXPORT_BUCKET)
        .collect();
    assert_eq!(uploads.len(), 2);
    assert_eq!(uploads[0], uploads[1]);
    assert_eq!(
        data.records
            .values()
            .filter(|record| record["bucket"] == OFFER_EXPORT_BUCKET)
            .count(),
        1
    );
}

#[test]
#[ignore = "requires DATABASE_URL_TEST; run make test-postgres"]
fn postgres_source_replacement_and_concurrent_preparation_do_not_change_pinned_export() {
    let db = Database::new();
    let server = Server::new();
    let objects = server.client();
    let source = stored_source(&objects);
    let replacement = objects
        .create_offer(STRUCTURALIZED_OFFERS_BUCKET, r#"{"variations":[]}"#)
        .unwrap();
    diesel::sql_query(
        "INSERT INTO offer_details_structuralized (id,offer_id,storage) VALUES ($1,$2,$3)",
    )
    .bind::<SqlUuid, _>(Uuid::now_v7())
    .bind::<SqlUuid, _>(source.offer_id)
    .bind::<Jsonb, _>(Json(replacement))
    .execute(&mut db.pool.get().unwrap())
    .unwrap();
    let barrier = std::sync::Barrier::new(2);
    let prepared_barrier = std::sync::Barrier::new(2);
    server.data.lock().unwrap().read_barrier = Some((
        STRUCTURALIZED_OFFERS_BUCKET.into(),
        std::sync::Arc::new(tokio::sync::Barrier::new(2)),
    ));
    let completed: Vec<OfferExport> = std::thread::scope(|scope| {
        let handles: Vec<_> = (0..2)
            .map(|_| {
                let barrier = &barrier;
                let prepared_barrier = &prepared_barrier;
                let source = source.clone();
                let store = PgStore::new(db.pool.clone(), objects.clone());
                scope.spawn(move || {
                    barrier.wait();
                    let prepared = store.prepare(source).unwrap();
                    prepared_barrier.wait();
                    store.complete(prepared).unwrap()
                })
            })
            .collect();
        handles
            .into_iter()
            .map(|handle| handle.join().unwrap())
            .collect()
    });
    assert_eq!(completed[0], completed[1]);
    {
        let mut data = server.data.lock().unwrap();
        data.read_barrier = None;
        let uploads: Vec<_> = data
            .uploads
            .iter()
            .filter(|(bucket, _)| bucket == OFFER_EXPORT_BUCKET)
            .collect();
        assert_eq!(uploads.len(), 2);
        assert_eq!(uploads[0], uploads[1]);
    }
    assert_eq!(completed[0].entries.len(), 2);
    assert_eq!(completed[0].source, source);
    let mut new_source = source;
    new_source.storage = replacement;
    let store = PgStore::new(db.pool.clone(), objects);
    let newer = store.complete(store.prepare(new_source).unwrap()).unwrap();
    assert_ne!(newer.id, completed[0].id);
    assert!(newer.entries.is_empty());
    assert_eq!(rows(&db).as_array().unwrap().len(), 2);
}

#[test]
#[ignore = "requires DATABASE_URL_TEST; run make test-postgres"]
fn postgres_failures_preserve_preparation_and_reject_bad_sources() {
    let db = Database::new();
    let server = Server::new();
    let store = PgStore::new(db.pool.clone(), server.client());
    let source = stored_source(&server.client());
    let mut invalid = source.clone();
    invalid.storage.content_size += 1;
    assert!(store.prepare(invalid).is_err());
    assert_eq!(rows(&db), json!([]));
    let prepared = store.prepare(source.clone()).unwrap();
    let pending = rows(&db);
    {
        let mut data = server.data.lock().unwrap();
        data.fail_create_at = Some(data.uploads.len() + 1);
    }
    assert!(store.complete(prepared).is_err());
    assert_eq!(rows(&db), pending);
    server.data.lock().unwrap().fail_create_at = None;
    let mut conflicting = source.clone();
    conflicting.url.push_str("?other");
    assert!(matches!(
        store.prepare(conflicting),
        Err(super::store::StoreError::SourceConflict)
    ));
    let complete = store.complete(store.prepare(source).unwrap()).unwrap();
    server.data.lock().unwrap().get_response = Some((404, "{}".into()));
    assert!(store.load(complete.id).is_err());
}

#[test]
#[ignore = "requires DATABASE_URL_TEST; run make test-postgres"]
fn postgres_export_migration_is_additive_and_reversible() {
    let db = Database::new();
    let mut connection = db.pool.get().unwrap();
    connection.batch_execute(
        "INSERT INTO offer_details_structuralized (id,offer_id,storage) VALUES (gen_random_uuid(),gen_random_uuid(),'{}')"
    ).unwrap();
    let snapshot = |connection: &mut PgConnection| {
        json_query(
            connection,
            "SELECT jsonb_agg(to_jsonb(s)) AS value FROM offer_details_structuralized s",
        )
    };
    let before = snapshot(&mut connection);
    connection.revert_last_migration(MIGRATIONS).unwrap();
    assert_eq!(snapshot(&mut connection), before);
    connection.run_next_migration(MIGRATIONS).unwrap();
    assert_eq!(snapshot(&mut connection), before);
    let error = connection
        .batch_execute(
            "INSERT INTO offer_export(id,offer_id,source_object_id,schema_version,source,stats)
         VALUES(gen_random_uuid(),gen_random_uuid(),gen_random_uuid(),1,'{}','{}')",
        )
        .unwrap_err();
    assert!(matches!(
        error,
        diesel::result::Error::DatabaseError(diesel::result::DatabaseErrorKind::CheckViolation, _)
    ));
}

#[test]
#[ignore = "requires DATABASE_URL_TEST; run make test-postgres"]
fn postgres_manual_queue_batches_prepare_export_without_downstream_services() {
    use crate::offer_discovery::structuralize_offer::prelude::OfferWasStructuralized;
    use crate::platform::{
        Platform,
        bus::{Bus, Command, Handle},
        state::AppState,
    };
    use serde::{Deserialize, Serialize};
    use tower::ServiceExt;

    #[derive(Serialize, Deserialize)]
    struct Trigger;
    impl Command for Trigger {
        const NAME: &'static str = "ExportTestTrigger";
    }
    struct Emit(OfferWasStructuralized);
    impl Handle for Emit {
        type Command = Trigger;
        type Event = OfferWasStructuralized;
        type Error = String;
        fn handle(&self, _: Trigger) -> Result<Vec<Self::Event>, String> {
            Ok(vec![self.0.clone()])
        }
    }

    let db = Database::new();
    let server = Server::new();
    let source = stored_source(&server.client());
    let platform = Platform {
        db: db.pool.clone(),
        http: crate::platform::http::Client,
        object_storage: server.client(),
    };
    let event = OfferWasStructuralized {
        offer_id: source.offer_id,
        consolidated_sitemap_id: source.consolidated_sitemap_id,
        retailer_code: source.retailer_code,
        url: source.url,
        storage: Some(source.storage),
    };
    let mut bus = Bus::default();
    bus.handle(Emit(event));
    super::prelude::register(&mut bus, &platform);
    let entered = server.runtime.enter();
    let bus = bus.start(db.pool.clone()).unwrap();
    drop(entered);
    bus.dispatcher().dispatch(&Trigger, "export-test").unwrap();
    let app = crate::app(AppState {
        bus: Some(bus),
        db: Some(db.pool.clone()),
        object_storage: Some(server.client()),
    });
    let run = |path: &str| {
        let request = axum::http::Request::post(path)
            .header("content-type", "application/json")
            .body(axum::body::Body::from(r#"{"limit":10}"#))
            .unwrap();
        server.runtime.block_on(async {
            let response = app.clone().oneshot(request).await.unwrap();
            assert_eq!(response.status(), axum::http::StatusCode::OK);
            let body = axum::body::to_bytes(response.into_body(), usize::MAX)
                .await
                .unwrap();
            let report: serde_json::Value = serde_json::from_slice(&body).unwrap();
            assert_eq!(report, json!({"processed":1,"errors":[]}));
        });
    };
    run("/api/commands/run");
    assert_eq!(rows(&db), json!([]));
    run("/api/events/run");
    assert_eq!(rows(&db), json!([]));
    run("/api/commands/run");
    assert_eq!(rows(&db)[0]["status"], "completed");
    let events = json_query(
        &mut db.pool.get().unwrap(),
        "SELECT jsonb_agg(payload::jsonb) AS value FROM event_entries WHERE event_type = 'OfferExportWasPrepared'",
    );
    assert_eq!(events.as_array().unwrap().len(), 1);
    assert_eq!(events[0]["export_id"], rows(&db)[0]["id"]);
    let count = json_query(
        &mut db.pool.get().unwrap(),
        "SELECT to_jsonb(count(*)) AS value FROM outbox_entries",
    );
    assert_eq!(count, json!(0));
}
