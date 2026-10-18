use super::entity::ProcessedSitemap;
use super::handler::Store;
use super::store::PgStore;
use super::store::StoreError;
use crate::platform::object_storage::{
    ObjectStorage, ObjectStorageMetadata, PROCESSED_SITEMAPS_BUCKET,
};
use crate::platform::test_db::{Database, json_query};
use bodul_shared::retailer::RetailerCode;
use diesel::connection::SimpleConnection;
use diesel::prelude::*;
use diesel::sql_types::{Jsonb, Uuid as SqlUuid};
use serde_json::json;
use sitemap::SitemapDocument;
use uuid::Uuid;

#[test]
#[ignore = "requires DATABASE_URL_TEST; run make test-postgres"]
fn postgres_processing_reads_entities_and_writes_atomically() {
    let db = Database::new();
    let server = crate::platform::test_objects::Server::new();
    let objects = server.client();
    let store = PgStore::new(db.pool.clone(), objects.clone());
    let mut conn = db.pool.get().unwrap();
    let retrieval_id = Uuid::now_v7();
    assert!(store.raw_documents(retrieval_id).unwrap().is_empty());
    for (second, body) in [(2, "second 🔎"), (1, "first ž")] {
        let id = Uuid::now_v7();
        let object_id = objects.create("sitemaps_raw", id, body).unwrap();
        diesel::sql_query(format!("INSERT INTO sitemaps_raw (id,retrieval_id,url,storage,fetched_at) VALUES ($1,$2,'https://shop.test/{second}',jsonb_build_object('content_size',{},'object_storage_id','{object_id}'),'2026-10-15T00:00:0{second}Z')",body.len()))
            .bind::<SqlUuid,_>(id).bind::<SqlUuid,_>(retrieval_id).execute(&mut conn).unwrap();
    }
    let entities = store.raw_documents(retrieval_id).unwrap();
    assert_eq!(
        entities
            .iter()
            .map(|e| e.document.body.as_str())
            .collect::<Vec<_>>(),
        ["first ž", "second 🔎"]
    );
    assert_eq!(entities[0].document.body_size, "first ž".len());
    assert!(store.raw_documents(Uuid::now_v7()).unwrap().is_empty());
    for (status, body) in [(404, "{}"), (503, "{}"), (200, "malformed")] {
        server.data.lock().unwrap().get_response = Some((status, body.into()));
        assert!(matches!(
            store.raw_documents(retrieval_id),
            Err(StoreError::ObjectStorage(_))
        ));
    }
    server.data.lock().unwrap().get_response = None;
    for storage in [
        json!(null),
        json!([7]),
        json!({}),
        json!({"content_size": -1}),
        json!({"content_size": "7"}),
        json!({"content_size": 1.5}),
        json!({"content_size": 7, "body_size": 7}),
    ] {
        diesel::sql_query("UPDATE sitemaps_raw SET storage=$1")
            .bind::<Jsonb, _>(storage)
            .execute(&mut conn)
            .unwrap();
        assert!(matches!(
            store.raw_documents(retrieval_id),
            Err(StoreError::Database(
                diesel::result::Error::DeserializationError(_)
            ))
        ));
    }
    conn.batch_execute(
        "UPDATE sitemaps_raw SET storage=jsonb_build_object('content_size',2147483648::bigint,'object_storage_id',gen_random_uuid())",
    )
    .unwrap();
    assert!(matches!(
        store.raw_documents(retrieval_id),
        Err(StoreError::InvalidContentSize)
    ));
    let entity = ProcessedSitemap::new(
        retrieval_id,
        RetailerCode::UgreenCa,
        SitemapDocument::default(),
    );
    {
        let mut data = server.data.lock().unwrap();
        data.fail_create_at = Some(data.uploads.len() + 1);
    }
    assert!(store.store_processed(&entity).is_err());
    assert_eq!(
        json_query(
            &mut conn,
            "SELECT to_jsonb(count(*)) AS value FROM sitemaps_processed"
        ),
        json!(0)
    );
    server.data.lock().unwrap().fail_create_at = None;
    conn.batch_execute(
        "ALTER TABLE sitemaps_processed ADD CONSTRAINT reject_payload CHECK (false)",
    )
    .unwrap();
    assert!(store.store_processed(&entity).is_err());
    assert_eq!(
        json_query(
            &mut conn,
            "SELECT to_jsonb(count(*)) AS value FROM sitemaps_processed"
        ),
        json!(0)
    );
    conn.batch_execute("ALTER TABLE sitemaps_processed DROP CONSTRAINT reject_payload")
        .unwrap();
    let stored = store.store_processed(&entity).unwrap();
    let row = json_query(
        &mut conn,
        "SELECT to_jsonb(p) AS value FROM sitemaps_processed p",
    );
    assert_eq!(row["id"], entity.id.to_string());
    assert_eq!(row["retrieval_id"], retrieval_id.to_string());
    assert_eq!(row["url_count"], entity.url_count());
    let storage: ObjectStorageMetadata = serde_json::from_value(row["storage"].clone()).unwrap();
    assert_eq!(storage.object_storage_id, stored.object_storage_id);
    assert_eq!(
        objects
            .get(PROCESSED_SITEMAPS_BUCKET, entity.id, &storage)
            .unwrap(),
        serde_json::to_string(entity.document()).unwrap()
    );
}

pub(super) fn verify_round_trip(db: &Database, objects: ObjectStorage) {
    let store = PgStore::new(db.pool.clone(), objects.clone());
    let entity = ProcessedSitemap::new(
        Uuid::now_v7(),
        RetailerCode::UgreenCa,
        SitemapDocument::default(),
    );
    let stored = store.store_processed(&entity).unwrap();
    assert_eq!(stored.entity.id, entity.id);
    let row = json_query(
        &mut db.pool.get().unwrap(),
        "SELECT to_jsonb(p) AS value FROM sitemaps_processed p",
    );
    let storage: ObjectStorageMetadata = serde_json::from_value(row["storage"].clone()).unwrap();
    assert_eq!(storage.object_storage_id, stored.object_storage_id);
    assert_eq!(
        objects
            .get(PROCESSED_SITEMAPS_BUCKET, entity.id, &storage)
            .unwrap(),
        serde_json::to_string(entity.document()).unwrap()
    );
}
