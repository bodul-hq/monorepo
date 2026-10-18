use super::entity::RawOffer;
use super::handler::Store;
use super::store::PgStore;
use crate::platform::object_storage::{ObjectStorage, OfferStorageMetadata, RAW_OFFERS_BUCKET};
use crate::platform::test_db::{Database, json_query};
use crate::platform::test_objects::Server;
use bodul_shared::retailer::RetailerCode;
use diesel::connection::SimpleConnection;
use uuid::Uuid;

const ROW: &str = "SELECT to_jsonb(o) AS value FROM offer_details_raw o";
fn entity() -> RawOffer {
    RawOffer {
        offer_id: Uuid::now_v7(),
        retailer_code: RetailerCode::AnkerAu,
        url: "https://shop.test/a".into(),
        body: "ž 🔎".into(),
    }
}

pub(super) fn verify_round_trip(db: &Database, objects: ObjectStorage) {
    let store = PgStore::new(db.pool.clone(), objects.clone());
    let mut conn = db.pool.get().unwrap();
    let mut entity = entity();
    let stored = store.store_raw_offer(&entity).unwrap();
    assert_eq!(stored.entity.body, entity.body);
    let before = json_query(&mut conn, ROW);
    assert_eq!(
        before["storage"]["object_storage_id"],
        stored.object_storage_id.to_string()
    );
    let storage: OfferStorageMetadata = serde_json::from_value(before["storage"].clone()).unwrap();
    assert_eq!(
        objects.get_offer(RAW_OFFERS_BUCKET, &storage).unwrap(),
        entity.body
    );
    entity.body = "replacement 🔎".into();
    entity.retailer_code = RetailerCode::UgreenCa;
    store.store_raw_offer(&entity).unwrap();
    let after = json_query(&mut conn, ROW);
    assert_eq!(after["id"], before["id"]);
    assert_eq!(after["offer_id"], entity.offer_id.to_string());
    assert_eq!(after["retailer_code"], "ugreenca");
    assert_eq!(after["url"], entity.url);
    assert_ne!(
        after["storage"]["external_id"],
        before["storage"]["external_id"]
    );
    assert_ne!(
        after["storage"]["object_storage_id"],
        before["storage"]["object_storage_id"]
    );
    let storage: OfferStorageMetadata = serde_json::from_value(after["storage"].clone()).unwrap();
    assert_eq!(storage.content_size, entity.body.len() as u64);
    assert_eq!(
        objects.get_offer(RAW_OFFERS_BUCKET, &storage).unwrap(),
        entity.body
    );
}

#[test]
#[ignore = "requires DATABASE_URL_TEST; run make test-postgres"]
fn postgres_download_entity_preserves_upsert_identity_and_atomicity() {
    let db = Database::new();
    let server = Server::new();
    verify_round_trip(&db, server.client());
}

#[test]
#[ignore = "requires DATABASE_URL_TEST; run make test-postgres"]
fn postgres_failures_preserve_local_state() {
    let db = Database::new();
    let server = Server::new();
    let store = PgStore::new(db.pool.clone(), server.client());
    let mut conn = db.pool.get().unwrap();
    let entity = entity();
    for exists in [false, true] {
        if exists {
            store.store_raw_offer(&entity).unwrap();
        }
        let snapshot =
            "SELECT coalesce(jsonb_agg(to_jsonb(o)),'[]') AS value FROM offer_details_raw o";
        let before = json_query(&mut conn, snapshot);
        {
            let mut data = server.data.lock().unwrap();
            data.fail_create_at = Some(data.uploads.len() + 1);
        }
        assert!(store.store_raw_offer(&entity).is_err());
        assert_eq!(json_query(&mut conn, snapshot), before);
        server.data.lock().unwrap().fail_create_at = None;
        conn.batch_execute(
            "ALTER TABLE offer_details_raw ADD CONSTRAINT reject_write CHECK(false) NOT VALID",
        )
        .unwrap();
        assert!(store.store_raw_offer(&entity).is_err());
        assert_eq!(json_query(&mut conn, snapshot), before);
        conn.batch_execute("ALTER TABLE offer_details_raw DROP CONSTRAINT reject_write")
            .unwrap();
    }
}

#[test]
#[ignore = "requires DATABASE_URL_TEST; run make test-postgres"]
fn postgres_concurrent_updates_keep_identity_and_complete_references() {
    let db = Database::new();
    let server = Server::new();
    let objects = server.client();
    let store = PgStore::new(db.pool.clone(), objects.clone());
    let entity = entity();
    store.store_raw_offer(&entity).unwrap();
    let before = json_query(&mut db.pool.get().unwrap(), ROW);
    let barrier = std::sync::Barrier::new(2);
    std::thread::scope(|scope| {
        for value in ["left", "right"] {
            let mut entity = entity.clone();
            entity.body = value.to_string();
            let store = &store;
            let barrier = &barrier;
            scope.spawn(move || {
                barrier.wait();
                store.store_raw_offer(&entity).unwrap();
            });
        }
    });
    let after = json_query(&mut db.pool.get().unwrap(), ROW);
    assert_eq!(before["id"], after["id"]);
    let storage: OfferStorageMetadata = serde_json::from_value(after["storage"].clone()).unwrap();
    let body = objects.get_offer(RAW_OFFERS_BUCKET, &storage).unwrap();
    assert!(["left", "right"].contains(&body.as_str()));
    assert_eq!(server.data.lock().unwrap().records.len(), 3);
}
