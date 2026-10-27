use std::collections::HashSet;

use super::*;
use crate::offer_discovery::prelude::export_testing::{assert_fact_payloads, fact_document};
use crate::platform::object_storage::ObjectStorageMetadata;

fn source_with_facts(objects: &ObjectStorage) -> ExportSource {
    super::super::prelude::testing::stored_source(objects, &fact_document().to_string())
}

fn row(db: &Database, id: Uuid) -> serde_json::Value {
    rows(db)
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["id"] == json!(id))
        .unwrap()
        .clone()
}

fn assert_uploaded_body(objects: &ObjectStorage, db: &Database, export: &OfferExport, body: &str) {
    let record = row(db, export.id);
    assert_eq!(record["status"], "completed");
    assert!(record["prepared_body"].is_null());
    let storage: ObjectStorageMetadata = serde_json::from_value(record["storage"].clone()).unwrap();
    assert_eq!(storage.content_size, body.len() as u64);
    assert_eq!(
        objects
            .get(OFFER_EXPORT_BUCKET, export.id, &storage)
            .unwrap(),
        body
    );
}

/// Also runs against the configured real object-storage service.
pub(super) fn verify_current_round_trip(db: &Database, objects: ObjectStorage) {
    let source = source_with_facts(&objects);
    let store = PgStore::new(db.pool.clone(), objects.clone());
    let prepared = store
        .prepare(source.clone(), super::super::entity::SCHEMA_VERSION)
        .unwrap();
    let expected = prepared.snapshot.clone();
    let body = prepared.body.clone();
    assert_fact_payloads(&expected, &fact_document());
    assert_eq!(expected.source, source);
    assert_eq!(row(db, expected.id)["prepared_body"], body);
    assert!(store.load(expected.id).unwrap().is_none());
    drop(store);
    let restarted = PgStore::new(db.pool.clone(), objects.clone());
    let retry = restarted
        .prepare(source.clone(), expected.schema_version)
        .unwrap();
    assert_eq!(retry.body, body);
    assert_eq!(retry.snapshot, expected);
    assert_eq!(restarted.complete(retry).unwrap(), expected);
    assert_uploaded_body(&objects, db, &expected, &body);
    drop(restarted);
    let restarted = PgStore::new(db.pool.clone(), objects);
    assert_eq!(restarted.load(expected.id).unwrap(), Some(expected.clone()));
    let retry = restarted.prepare(source, expected.schema_version).unwrap();
    assert_eq!(retry.body, body);
    assert_eq!(retry.snapshot, expected);
    assert!(retry.storage.is_some());
    assert_eq!(restarted.complete(retry).unwrap(), expected);
    assert_eq!(rows(db).as_array().unwrap().len(), 1);
}

#[test]
#[ignore = "requires DATABASE_URL_TEST; run make test-postgres"]
fn current_facts_round_trip_preserves_exact_values_and_bytes() {
    let server = Server::new();
    verify_current_round_trip(&Database::new(), server.client());
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
fn fact_versions_coexist_without_rewriting_legacy_pending_or_completed_exports() {
    let db = Database::new();
    let server = Server::new();
    let objects = server.client();
    let source = source_with_facts(&objects);
    let store = PgStore::new(db.pool.clone(), objects.clone());
    let legacy_pending = store.prepare(source.clone(), 1).unwrap();
    let legacy_pending_body = legacy_pending.body.clone();
    let legacy_pending_snapshot = legacy_pending.snapshot.clone();
    let legacy_completed = store.prepare(source.clone(), 5).unwrap();
    let legacy_completed_body = legacy_completed.body.clone();
    let legacy_completed = store.complete(legacy_completed).unwrap();
    let pending_row = row(&db, legacy_pending_snapshot.id);
    let completed_row = row(&db, legacy_completed.id);
    let mut identities = HashSet::new();
    for export in [&legacy_pending_snapshot, &legacy_completed] {
        assert_fact_payloads(export, &fact_document());
        assert!(identities.insert(export.id));
        for entry in &export.entries {
            assert!(identities.insert(entry.id));
        }
    }
    for version in [6, 7, 8] {
        let prepared = store.prepare(source.clone(), version).unwrap();
        let expected = prepared.snapshot.clone();
        let body = prepared.body.clone();
        assert_eq!(expected.schema_version, version);
        assert_eq!(expected.source, source);
        assert_fact_payloads(&expected, &fact_document());
        assert!(identities.insert(expected.id));
        for entry in &expected.entries {
            assert!(identities.insert(entry.id));
        }
        assert_eq!(row(&db, expected.id)["prepared_body"], body);
        let restarted = PgStore::new(db.pool.clone(), objects.clone());
        let retry = restarted.prepare(source.clone(), version).unwrap();
        assert_eq!(retry.body, body);
        assert_eq!(retry.snapshot, expected);
        assert_eq!(restarted.complete(retry).unwrap(), expected);
        assert_uploaded_body(&objects, &db, &expected, &body);
        assert_eq!(restarted.load(expected.id).unwrap(), Some(expected.clone()));
        let completed_retry = restarted.prepare(source.clone(), version).unwrap();
        assert_eq!(completed_retry.body, body);
        assert_eq!(completed_retry.snapshot, expected);
        assert_eq!(row(&db, legacy_pending_snapshot.id), pending_row);
        assert_eq!(row(&db, legacy_completed.id), completed_row);
        assert_eq!(
            restarted.prepare(source.clone(), 1).unwrap().body,
            legacy_pending_body
        );
        assert_eq!(
            restarted.prepare(source.clone(), 5).unwrap().body,
            legacy_completed_body
        );
    }
    assert_eq!(
        store.complete(legacy_pending).unwrap(),
        legacy_pending_snapshot
    );
    assert_uploaded_body(
        &objects,
        &db,
        &legacy_pending_snapshot,
        &legacy_pending_body,
    );
    assert_uploaded_body(&objects, &db, &legacy_completed, &legacy_completed_body);
    assert_eq!(rows(&db).as_array().unwrap().len(), 5);
}

#[test]
#[ignore = "requires DATABASE_URL_TEST; run make test-postgres"]
fn fact_versions_recover_identical_bytes_after_upload_then_completion_failure() {
    let db = Database::new();
    let server = Server::new();
    for version in [6, 7, 8] {
        let source = source_with_facts(&server.client());
        let store = PgStore::new(db.pool.clone(), server.client());
        let prepared = store.prepare(source.clone(), version).unwrap();
        let expected = prepared.snapshot.clone();
        let body = prepared.body.clone();
        let pending = row(&db, expected.id);
        db.pool.get().unwrap().batch_execute("ALTER TABLE offer_export ADD CONSTRAINT fail_fact_completion CHECK (status <> 'completed') NOT VALID").unwrap();
        assert!(store.complete(prepared).is_err());
        assert_eq!(row(&db, expected.id), pending);
        assert!(store.load(expected.id).unwrap().is_none());
        db.pool
            .get()
            .unwrap()
            .batch_execute("ALTER TABLE offer_export DROP CONSTRAINT fail_fact_completion")
            .unwrap();
        drop(store);
        // Recovery must use durable prepared bytes, even if the original input is unavailable.
        server
            .data
            .lock()
            .unwrap()
            .records
            .remove(&source.storage.object_storage_id)
            .unwrap();
        let restarted = PgStore::new(db.pool.clone(), server.client());
        let retry = restarted.prepare(source.clone(), version).unwrap();
        assert_eq!(retry.body, body);
        assert_eq!(retry.snapshot, expected);
        assert_fact_payloads(&retry.snapshot, &fact_document());
        assert_eq!(restarted.complete(retry).unwrap(), expected);
        assert_uploaded_body(&server.client(), &db, &expected, &body);
        assert_eq!(restarted.prepare(source, version).unwrap().body, body);
        let data = server.data.lock().unwrap();
        let uploads: Vec<_> = data
            .uploads
            .iter()
            .filter(|(bucket, request)| {
                bucket == OFFER_EXPORT_BUCKET && request["idempotency_key"] == json!(expected.id)
            })
            .collect();
        assert_eq!(uploads.len(), 2);
        assert_eq!(uploads[0], uploads[1]);
        assert_eq!(uploads[0].1["payload"], body);
        assert_eq!(
            data.records
                .values()
                .filter(|record| record["bucket"] == OFFER_EXPORT_BUCKET
                    && record["idempotency_key"] == json!(expected.id))
                .count(),
            1
        );
    }
}

#[test]
#[ignore = "requires DATABASE_URL_TEST; run make test-postgres"]
fn malformed_observations_never_persist_an_export_preparation() {
    let db = Database::new();
    let server = Server::new();
    let store = PgStore::new(db.pool.clone(), server.client());
    for (version, field, value) in [
        (6, "was_price", json!({"amount":1,"currency":"JPY"})),
        (
            6,
            "was_price",
            json!({"amount":9_007_199_254_740_994_i64,"currency":"USD"}),
        ),
        (7, "availability", json!({"other":" "})),
        (7, "item_condition", json!("guessed_new")),
        (8, "content", json!(null)),
        (8, "content", json!({})),
        (8, "content", json!({"description":"bad\u{0}"})),
    ] {
        let mut document = fact_document();
        document["variations"][0][field] = value;
        let source =
            super::super::prelude::testing::stored_source(&server.client(), &document.to_string());
        assert!(store.prepare(source, version).is_err());
        assert_eq!(rows(&db), json!([]));
    }
    assert!(
        server
            .data
            .lock()
            .unwrap()
            .uploads
            .iter()
            .all(|(bucket, _)| bucket != OFFER_EXPORT_BUCKET)
    );
}
