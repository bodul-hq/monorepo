use super::entity::ExtractedOffer;
use super::handler::Store;
use super::store::PgStore;
use crate::platform::db::Json;
use crate::platform::object_storage::{
    EXTRACTED_OFFERS_BUCKET, ObjectStorage, OfferStorageMetadata, RAW_OFFERS_BUCKET,
};
use crate::platform::test_db::{Database, json_query};
use crate::platform::test_objects::Server;
use diesel::connection::SimpleConnection;
use diesel::prelude::*;
use diesel::sql_types::{Jsonb, Uuid as SqlUuid};
use serde_json::json;
use uuid::Uuid;

const ROW: &str = "SELECT to_jsonb(o) AS value FROM offer_details_extracted o";

pub(super) fn verify_round_trip(db: &Database, objects: ObjectStorage) {
    let store = PgStore::new(db.pool.clone(), objects.clone());
    let mut conn = db.pool.get().unwrap();
    let offer_id = Uuid::now_v7();
    assert!(store.raw_offer(offer_id).unwrap().is_none());
    let input_value = json!("<html>ž 🔎</html>");
    let input_body = input_value.as_str().unwrap().to_owned();
    let input_storage = objects
        .create_offer(RAW_OFFERS_BUCKET, &input_body)
        .unwrap();
    diesel::sql_query("INSERT INTO offer_details_raw (id,offer_id,storage,url,retailer_code) VALUES ($1,$2,$3,'https://shop.test/a','ankerau')")
        .bind::<SqlUuid,_>(Uuid::now_v7()).bind::<SqlUuid,_>(offer_id)
        .bind::<Jsonb,_>(Json(input_storage)).execute(&mut conn).unwrap();
    let input = store.raw_offer(offer_id).unwrap().unwrap();
    assert_eq!(input.offer_id, offer_id);
    assert_eq!(input.object_storage_id, input_storage.object_storage_id);
    assert_eq!(json!(input.body), input_value);
    let mut entity = ExtractedOffer {
        offer_id,
        document: json!({"value":"first ž"}),
    };
    let mut previous: Option<serde_json::Value> = None;
    for value in [
        json!({"value":"first ž"}),
        json!({"value":"updated 🔎"}),
        json!(null),
        json!([1, "ž"]),
    ] {
        entity.document = value;
        let stored = store.store_extracted_offer(&entity).unwrap();
        assert_eq!(stored.entity, entity);
        let row = json_query(&mut conn, ROW);
        let storage: OfferStorageMetadata = serde_json::from_value(row["storage"].clone()).unwrap();
        assert_eq!(stored.object_storage_id, storage.object_storage_id);
        assert_eq!(row["offer_id"], offer_id.to_string());
        assert_eq!(
            storage.content_size,
            entity.document.to_string().len() as u64
        );
        assert_eq!(
            objects
                .get_offer(EXTRACTED_OFFERS_BUCKET, &storage)
                .unwrap(),
            entity.document.to_string()
        );
        if let Some(before) = previous {
            assert_eq!(row["id"], before["id"]);
            assert_ne!(
                row["storage"]["external_id"],
                before["storage"]["external_id"]
            );
            assert_ne!(
                row["storage"]["object_storage_id"],
                before["storage"]["object_storage_id"]
            );
        }
        previous = Some(row);
    }
    assert_eq!(
        json_query(
            &mut conn,
            "SELECT to_jsonb(count(*)) AS value FROM offer_details_extracted"
        ),
        json!(1)
    );
}

#[test]
#[ignore = "requires DATABASE_URL_TEST; run make test-postgres"]
fn postgres_entity_reads_and_atomic_upserts() {
    let db = Database::new();
    let server = Server::new();
    verify_round_trip(&db, server.client());
}

#[test]
#[ignore = "requires DATABASE_URL_TEST; run make test-postgres"]
fn postgres_failures_preserve_local_state() {
    let db = Database::new();
    let server = Server::new();
    let objects = server.client();
    let store = PgStore::new(db.pool.clone(), objects.clone());
    let mut conn = db.pool.get().unwrap();
    let entity = ExtractedOffer {
        offer_id: Uuid::now_v7(),
        document: json!({"a":"ž"}),
    };
    for exists in [false, true] {
        if exists {
            store.store_extracted_offer(&entity).unwrap();
        }
        let snapshot = || "SELECT coalesce(jsonb_agg(to_jsonb(o)),'[]') AS value FROM offer_details_extracted o";
        let before = json_query(&mut conn, snapshot());
        {
            let mut data = server.data.lock().unwrap();
            data.fail_create_at = Some(data.uploads.len() + 1);
        }
        assert!(store.store_extracted_offer(&entity).is_err());
        assert_eq!(json_query(&mut conn, snapshot()), before);
        server.data.lock().unwrap().fail_create_at = None;
        conn.batch_execute("ALTER TABLE offer_details_extracted ADD CONSTRAINT reject_write CHECK(false) NOT VALID").unwrap();
        assert!(store.store_extracted_offer(&entity).is_err());
        assert_eq!(json_query(&mut conn, snapshot()), before);
        conn.batch_execute("ALTER TABLE offer_details_extracted DROP CONSTRAINT reject_write")
            .unwrap();
    }
    let storage = objects.create_offer(RAW_OFFERS_BUCKET, "null").unwrap();
    diesel::sql_query("INSERT INTO offer_details_raw (id,offer_id,storage,url,retailer_code) VALUES ($1,$2,$3,'https://shop.test/a','ankerau')")
        .bind::<SqlUuid,_>(Uuid::now_v7()).bind::<SqlUuid,_>(entity.offer_id)
        .bind::<Jsonb,_>(Json(storage)).execute(&mut conn).unwrap();
    for (status, body) in [(404, "{}"), (503, "{}"), (200, "malformed")] {
        server.data.lock().unwrap().get_response = Some((status, body.into()));
        assert!(store.raw_offer(entity.offer_id).is_err());
    }
    server.data.lock().unwrap().get_response = None;
    conn.batch_execute("UPDATE offer_details_raw SET storage='{}'")
        .unwrap();
    assert!(store.raw_offer(entity.offer_id).is_err());
}

#[test]
#[ignore = "requires DATABASE_URL_TEST; run make test-postgres"]
fn postgres_concurrent_updates_keep_identity_and_complete_references() {
    let db = Database::new();
    let server = Server::new();
    let objects = server.client();
    let store = PgStore::new(db.pool.clone(), objects.clone());
    let entity = ExtractedOffer {
        offer_id: Uuid::now_v7(),
        document: json!({"initial":true}),
    };
    store.store_extracted_offer(&entity).unwrap();
    let before = json_query(&mut db.pool.get().unwrap(), ROW);
    let barrier = std::sync::Barrier::new(2);
    std::thread::scope(|scope| {
        for value in ["left", "right"] {
            let mut entity = entity.clone();
            entity.document = json!({"value":value});
            let store = &store;
            let barrier = &barrier;
            scope.spawn(move || {
                barrier.wait();
                store.store_extracted_offer(&entity).unwrap();
            });
        }
    });
    let after = json_query(&mut db.pool.get().unwrap(), ROW);
    assert_eq!(before["id"], after["id"]);
    let storage: OfferStorageMetadata = serde_json::from_value(after["storage"].clone()).unwrap();
    let body = objects
        .get_offer(EXTRACTED_OFFERS_BUCKET, &storage)
        .unwrap();
    assert!([r#"{"value":"left"}"#, r#"{"value":"right"}"#].contains(&body.as_str()));
    assert_eq!(server.data.lock().unwrap().records.len(), 3);
}
