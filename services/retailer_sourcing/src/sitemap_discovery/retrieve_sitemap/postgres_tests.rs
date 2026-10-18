use super::entity::RetrievedSitemapDocument;
use super::handler::Store;
use super::store::{PgStore, StoreError};
use crate::platform::object_storage::{ObjectStorage, ObjectStorageMetadata, RAW_SITEMAPS_BUCKET};
use crate::platform::test_db::{Database, json_query};
use crate::platform::test_objects::Server;
use diesel::connection::SimpleConnection;
use serde_json::json;
use sitemap::RawSitemapDocument;
use uuid::Uuid;

fn documents() -> Vec<RetrievedSitemapDocument> {
    RetrievedSitemapDocument::from_documents(
        Uuid::now_v7(),
        ["first ž", "second 🔎"]
            .into_iter()
            .map(|body| RawSitemapDocument {
                url: format!("https://shop.test/{body}"),
                last_modified: None,
                body_size: body.len(),
                body: body.into(),
            })
            .collect(),
    )
}

pub(super) fn verify_round_trip(db: &Database, objects: ObjectStorage) {
    let store = PgStore::new(db.pool.clone(), objects.clone());
    let input = documents();
    let stored = store.store_documents(&input).unwrap();
    let mut conn = db.pool.get().unwrap();
    let rows = json_query(
        &mut conn,
        "SELECT jsonb_agg(to_jsonb(r) ORDER BY id) AS value FROM sitemaps_raw r",
    );
    assert_eq!(rows.as_array().unwrap().len(), input.len());
    for (input, stored) in input.iter().zip(&stored) {
        let row = rows
            .as_array()
            .unwrap()
            .iter()
            .find(|row| row["id"] == input.id.to_string())
            .unwrap();
        assert_eq!(stored.sitemap.id, input.id);
        assert_ne!(stored.object_storage_id, input.id);
        assert_eq!(row["retrieval_id"], input.retrieval_id.to_string());
        assert_eq!(row["url"], input.document.url);
        assert_eq!(
            row["storage"],
            json!({"content_size": input.document.body_size,"object_storage_id":stored.object_storage_id})
        );
        let metadata: ObjectStorageMetadata =
            serde_json::from_value(row["storage"].clone()).unwrap();
        assert_eq!(
            objects
                .get(RAW_SITEMAPS_BUCKET, input.id, &metadata)
                .unwrap(),
            input.document.body
        );
    }
    assert_eq!(
        json_query(
            &mut conn,
            "SELECT to_jsonb(to_regclass('raw_sitemap_documents_data') IS NULL) AS value"
        ),
        json!(true)
    );
    // A successful HTTP retry uses the original local ID, even if the local
    // unique constraint subsequently rejects the duplicate batch.
    assert!(store.store_documents(&input).is_err());
}

#[test]
#[ignore = "requires DATABASE_URL_TEST; run make test-postgres"]
fn postgres_raw_sitemaps_round_trip_through_the_client() {
    let db = Database::new();
    let server = Server::new();
    verify_round_trip(&db, server.client());
    let data = server.data.lock().unwrap();
    assert_eq!(data.records.len(), 2);
    assert!(
        data.uploads
            .iter()
            .all(|(bucket, input)| bucket == RAW_SITEMAPS_BUCKET && input["metadata"] == json!({}))
    );
}

#[test]
#[ignore = "requires DATABASE_URL_TEST; run make test-postgres"]
fn postgres_raw_sitemap_batch_failures_leave_no_local_rows() {
    let db = Database::new();
    let server = Server::new();
    let store = PgStore::new(db.pool.clone(), server.client());
    let mut input = documents();
    input[1].document.body_size = i32::MAX as usize + 1;
    assert!(matches!(
        store.store_documents(&input),
        Err(StoreError::BodyTooLarge(_))
    ));
    assert!(server.data.lock().unwrap().uploads.is_empty());
    input[1].document.body_size = 0;
    assert!(matches!(
        store.store_documents(&input),
        Err(StoreError::InvalidBodySize)
    ));
    assert!(server.data.lock().unwrap().uploads.is_empty());
    input[1].document.body_size = input[1].document.body.len();
    server.data.lock().unwrap().fail_create_at = Some(2);
    assert!(matches!(
        store.store_documents(&input),
        Err(StoreError::ObjectStorage(_))
    ));
    let mut conn = db.pool.get().unwrap();
    assert_eq!(
        json_query(
            &mut conn,
            "SELECT to_jsonb(count(*)) AS value FROM sitemaps_raw"
        ),
        json!(0)
    );
    assert_eq!(server.data.lock().unwrap().records.len(), 1);
    server.data.lock().unwrap().fail_create_at = None;
    conn.batch_execute("ALTER TABLE sitemaps_raw ADD CONSTRAINT reject_batch CHECK (false)")
        .unwrap();
    assert!(matches!(
        store.store_documents(&input),
        Err(StoreError::Database(_))
    ));
    assert_eq!(
        json_query(
            &mut conn,
            "SELECT to_jsonb(count(*)) AS value FROM sitemaps_raw"
        ),
        json!(0)
    );
    assert_eq!(server.data.lock().unwrap().records.len(), 2);
    conn.batch_execute("ALTER TABLE sitemaps_raw DROP CONSTRAINT reject_batch")
        .unwrap();
    assert_eq!(store.store_documents(&input).unwrap().len(), 2);
    assert_eq!(server.data.lock().unwrap().records.len(), 2);
    assert!(store.store_documents(&[]).unwrap().is_empty());
}
