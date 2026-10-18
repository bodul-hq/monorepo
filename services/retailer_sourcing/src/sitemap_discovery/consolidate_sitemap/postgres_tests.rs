use crate::platform::db::Json;
use crate::platform::object_storage::{
    CONSOLIDATED_SITEMAPS_BUCKET, ObjectStorage, ObjectStorageMetadata, PROCESSED_SITEMAPS_BUCKET,
};
use crate::platform::test_objects::Server;
use diesel::connection::SimpleConnection;
use diesel::prelude::*;
use diesel::sql_types::{Jsonb, Uuid as SqlUuid};
use diesel_migrations::{EmbeddedMigrations, MigrationHarness, embed_migrations};
use serde_json::{Value, json};
use sitemap::{GroupedLinks, SitemapDocument};
use uuid::Uuid;

use super::entity::ConsolidatedSitemap;
use super::handler::Store;
use super::store::{PgStore, StoreError};
use crate::platform::test_db::Database;

const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

#[derive(QueryableByName)]
struct JsonRow {
    #[diesel(sql_type = Jsonb)]
    value: Value,
}
fn json_query(conn: &mut PgConnection, sql: &str) -> Value {
    diesel::sql_query(sql)
        .get_result::<JsonRow>(conn)
        .unwrap()
        .value
}

fn seed_processed(conn: &mut PgConnection) -> (Uuid, Uuid, Value) {
    let id = Uuid::now_v7();
    let retrieval_id = Uuid::now_v7();
    let document = serde_json::to_value(SitemapDocument::default()).unwrap();
    diesel::sql_query("INSERT INTO sitemaps_processed (id,retrieval_id,retailer_code,storage,url_count) VALUES ($1,$2,'ugreenca',jsonb_build_object('content_size',0),0)")
        .bind::<SqlUuid,_>(id).bind::<SqlUuid,_>(retrieval_id).execute(conn).unwrap();
    diesel::sql_query("INSERT INTO processed_sitemaps_data (id,document) VALUES ($1,$2)")
        .bind::<SqlUuid, _>(id)
        .bind::<Jsonb, _>(&document)
        .execute(conn)
        .unwrap();
    (id, retrieval_id, document)
}

fn seed_object_processed(conn: &mut PgConnection, objects: &ObjectStorage) -> (Uuid, Uuid, Value) {
    let id = Uuid::now_v7();
    let retrieval_id = Uuid::now_v7();
    let document = serde_json::to_value(SitemapDocument::default()).unwrap();
    let body = document.to_string();
    let object_storage_id = objects
        .create(PROCESSED_SITEMAPS_BUCKET, id, &body)
        .unwrap();
    diesel::sql_query("INSERT INTO sitemaps_processed (id,retrieval_id,retailer_code,storage,url_count) VALUES ($1,$2,'ugreenca',$3,0)")
        .bind::<SqlUuid,_>(id).bind::<SqlUuid,_>(retrieval_id)
        .bind::<Jsonb,_>(Json(ObjectStorageMetadata { content_size: body.len() as u64, object_storage_id }))
        .execute(conn).unwrap();
    (id, retrieval_id, document)
}

pub(super) fn verify_round_trip(db: &Database, objects: ObjectStorage) {
    let store = PgStore::new(db.pool.clone(), objects.clone());
    let mut conn = db.pool.get().unwrap();
    let (id, retrieval_id, document) = seed_object_processed(&mut conn, &objects);
    let processed = store.processed_sitemap(id).unwrap();
    assert_eq!(processed.retrieval_id, retrieval_id);
    assert_eq!(serde_json::to_value(&processed.document).unwrap(), document);
    let entity = ConsolidatedSitemap::new(
        &processed,
        GroupedLinks {
            product: vec!["https://shop.test/ž".into()],
            ..Default::default()
        },
    );
    let stored = store.store_consolidated(&entity).unwrap();
    assert_eq!(stored.entity.id, entity.id);
    let row = json_query(
        &mut conn,
        "SELECT to_jsonb(c) AS value FROM sitemaps_consolidated c",
    );
    assert_eq!(
        row["references"],
        json!({"processed_sitemap_id":id,"retrieval_id":retrieval_id,"retailer_code":"ugreenca"})
    );
    assert_eq!(
        row["stats"],
        json!({"product_count":1,"catalog_count":0,"content_count":0,"not_interested_count":0,"unknown_count":0})
    );
    let storage: ObjectStorageMetadata = serde_json::from_value(row["storage"].clone()).unwrap();
    assert_eq!(storage.object_storage_id, stored.object_storage_id);
    assert_eq!(
        objects
            .get(CONSOLIDATED_SITEMAPS_BUCKET, entity.id, &storage)
            .unwrap(),
        serde_json::to_string(entity.links()).unwrap()
    );
    let duplicate = ConsolidatedSitemap::new(&processed, GroupedLinks::default());
    assert!(matches!(
        store.store_consolidated(&duplicate),
        Err(StoreError::Database(_))
    ));
    let exported = json_query(
        &mut conn,
        &format!(
            "SELECT to_jsonb(crawl) AS value FROM ({}) crawl",
            include_str!("../../bin/export_sitemaps.sql")
        ),
    );
    assert_eq!(exported["id"], id.to_string());
    assert_eq!(exported["consolidated_id"], entity.id.to_string());
    assert_eq!(exported["consolidated_storage"], row["storage"]);
    assert_eq!(
        exported["storage"]["object_storage_id"],
        processed.object_storage_id.to_string()
    );
}

#[test]
#[ignore = "requires DATABASE_URL_TEST; run make test-postgres"]
fn postgres_store_round_trip_and_export_query() {
    let db = Database::new();
    let server = Server::new();
    verify_round_trip(&db, server.client());
}

#[test]
#[ignore = "requires DATABASE_URL_TEST; run make test-postgres"]
fn postgres_failed_objects_and_local_inserts() {
    let db = Database::new();
    let server = Server::new();
    let objects = server.client();
    let store = PgStore::new(db.pool.clone(), objects.clone());
    let mut conn = db.pool.get().unwrap();
    let (id, _, _) = seed_object_processed(&mut conn, &objects);
    let processed = store.processed_sitemap(id).unwrap();
    let entity = ConsolidatedSitemap::new(&processed, GroupedLinks::default());
    {
        let mut data = server.data.lock().unwrap();
        data.fail_create_at = Some(data.uploads.len() + 1);
    }
    assert!(store.store_consolidated(&entity).is_err());
    server.data.lock().unwrap().fail_create_at = None;
    conn.batch_execute(
        "ALTER TABLE sitemaps_consolidated ADD CONSTRAINT reject_write CHECK(false)",
    )
    .unwrap();
    assert!(store.store_consolidated(&entity).is_err());
    assert_eq!(
        json_query(
            &mut conn,
            "SELECT to_jsonb(count(*)) AS value FROM sitemaps_consolidated"
        ),
        json!(0)
    );
    conn.batch_execute("ALTER TABLE sitemaps_consolidated DROP CONSTRAINT reject_write")
        .unwrap();
    store.store_consolidated(&entity).unwrap();
    for (status, body) in [(404, "{}"), (503, "{}"), (200, "malformed")] {
        server.data.lock().unwrap().get_response = Some((status, body.into()));
        assert!(store.processed_sitemap(id).is_err());
    }
    server.data.lock().unwrap().get_response = None;
    conn.batch_execute("UPDATE sitemaps_processed SET retailer_code='invalid'")
        .unwrap();
    assert!(matches!(
        store.processed_sitemap(id),
        Err(StoreError::InvalidRetailerCode(_))
    ));
    assert!(matches!(
        store.processed_sitemap(Uuid::now_v7()),
        Err(StoreError::Database(diesel::result::Error::NotFound))
    ));
}

fn seed_legacy_consolidated(conn: &mut PgConnection, processed: Uuid, retrieval: Uuid) -> Uuid {
    let id = Uuid::now_v7();
    diesel::sql_query("INSERT INTO sitemaps_consolidated (id,stats,\"references\",storage) VALUES ($1,$2,$3,$4)")
        .bind::<SqlUuid,_>(id)
        .bind::<Jsonb,_>(json!({"product_count":0,"catalog_count":0,"content_count":0,"not_interested_count":0,"unknown_count":0}))
        .bind::<Jsonb,_>(json!({"processed_sitemap_id":processed,"retrieval_id":retrieval,"retailer_code":"ugreenca"}))
        .bind::<Jsonb,_>(json!({"content_size":2})).execute(conn).unwrap();
    diesel::sql_query("INSERT INTO consolidated_sitemaps_data (id,content) VALUES ($1,'{}')")
        .bind::<SqlUuid, _>(id)
        .execute(conn)
        .unwrap();
    id
}

#[test]
#[ignore = "requires DATABASE_URL_TEST; run make test-postgres"]
fn postgres_migration_preserves_populated_rows_through_up_down_up() {
    let db = Database::new();
    let mut conn = db.pool.get().unwrap();
    // Test the stats migration independently of the later references/storage migration.
    conn.revert_last_migration(MIGRATIONS).unwrap(); // Undo remaining object storage.
    conn.revert_last_migration(MIGRATIONS).unwrap(); // Undo raw object storage.
    conn.revert_last_migration(MIGRATIONS).unwrap(); // Undo pipeline storage.
    conn.revert_last_migration(MIGRATIONS).unwrap(); // Undo table renames.
    conn.revert_last_migration(MIGRATIONS).unwrap();
    conn.revert_last_migration(MIGRATIONS).unwrap();
    for counts in [[0, 0, 0, 0, 0], [1, 2, 3, 4, i32::MAX]] {
        let (processed_id, retrieval_id, _) = seed_processed_legacy(&mut conn);
        let id = Uuid::now_v7();
        diesel::sql_query(format!("INSERT INTO consolidated_sitemaps (id,processed_sitemap_id,retrieval_id,retailer_code,content_size,product_count,catalog_count,content_count,not_interested_count,unknown_count,consolidated_at) VALUES ($1,$2,$3,'ugreenca',123,{},{},{},{},{},'2026-10-14T12:34:56.123456Z')", counts[0],counts[1],counts[2],counts[3],counts[4]))
            .bind::<SqlUuid,_>(id).bind::<SqlUuid,_>(processed_id).bind::<SqlUuid,_>(retrieval_id).execute(&mut conn).unwrap();
        diesel::sql_query("INSERT INTO consolidated_sitemaps_data VALUES ($1, $2)")
            .bind::<SqlUuid, _>(id)
            .bind::<Jsonb, _>(json!({"product":["ž 🔎"],"unknown":[]}))
            .execute(&mut conn)
            .unwrap();
    }
    let original = json_query(&mut conn, SNAPSHOT_LEGACY);
    conn.run_next_migration(MIGRATIONS).unwrap();
    let upgraded = json_query(&mut conn, SNAPSHOT_LEGACY);
    for (before, after) in original["records"]
        .as_array()
        .unwrap()
        .iter()
        .zip(upgraded["records"].as_array().unwrap())
    {
        for name in [
            "product_count",
            "catalog_count",
            "content_count",
            "not_interested_count",
            "unknown_count",
        ] {
            assert_eq!(before[name], after["stats"][name]);
            assert!(after.get(name).is_none());
        }
        for name in [
            "id",
            "processed_sitemap_id",
            "retrieval_id",
            "retailer_code",
            "content_size",
            "consolidated_at",
        ] {
            assert_eq!(before[name], after[name]);
        }
    }
    assert_eq!(upgraded["payloads"], original["payloads"]);
    assert_eq!(upgraded["processed"], original["processed"]);
    assert_eq!(upgraded["documents"], original["documents"]);
    conn.revert_last_migration(MIGRATIONS).unwrap();
    assert_eq!(json_query(&mut conn, SNAPSHOT_LEGACY), original);
    conn.run_next_migration(MIGRATIONS).unwrap();
    assert_eq!(json_query(&mut conn, SNAPSHOT_LEGACY), upgraded);
    // Verify the existing uniqueness and foreign key constraints still hold.
    assert!(
        conn.batch_execute(
            "INSERT INTO consolidated_sitemaps SELECT * FROM consolidated_sitemaps LIMIT 1"
        )
        .is_err()
    );
    assert!(conn.batch_execute("UPDATE consolidated_sitemaps SET processed_sitemap_id='00000000-0000-0000-0000-000000000000'").is_err());
    assert!(
        conn.batch_execute("UPDATE consolidated_sitemaps SET stats=NULL")
            .is_err()
    );
}

#[test]
#[ignore = "requires DATABASE_URL_TEST; run make test-postgres"]
fn postgres_downgrade_rejects_invalid_stats_atomically() {
    let db = Database::new();
    let mut conn = db.pool.get().unwrap();
    conn.revert_last_migration(MIGRATIONS).unwrap(); // Undo remaining object storage.
    let (id, retrieval, _) = seed_processed(&mut conn);
    seed_legacy_consolidated(&mut conn, id, retrieval);
    conn.revert_last_migration(MIGRATIONS).unwrap(); // Undo raw object storage.
    conn.revert_last_migration(MIGRATIONS).unwrap(); // Undo pipeline storage.
    conn.revert_last_migration(MIGRATIONS).unwrap(); // Undo table renames.
    // The entity was written using the latest schema; revert only references/storage first.
    conn.revert_last_migration(MIGRATIONS).unwrap();
    let valid = json_query(
        &mut conn,
        "SELECT stats AS value FROM consolidated_sitemaps",
    );
    for bad in [
        json!({}),
        json!(null),
        json!({"product_count":2147483648_i64}),
        json!({"product_count":"bad"}),
    ] {
        diesel::sql_query("UPDATE consolidated_sitemaps SET stats=$1")
            .bind::<Jsonb, _>(bad)
            .execute(&mut conn)
            .unwrap();
        let before = json_query(&mut conn, SNAPSHOT_LEGACY);
        assert!(conn.revert_last_migration(MIGRATIONS).is_err());
        assert_eq!(json_query(&mut conn, SNAPSHOT_LEGACY), before);
        assert_eq!(
            conn.applied_migrations().unwrap()[0].to_string(),
            "20261015000007"
        );
    }
    diesel::sql_query("UPDATE consolidated_sitemaps SET stats=$1")
        .bind::<Jsonb, _>(valid)
        .execute(&mut conn)
        .unwrap();
    conn.revert_last_migration(MIGRATIONS).unwrap();
}

#[test]
#[ignore = "requires DATABASE_URL_TEST; run make test-postgres"]
fn postgres_references_storage_migration_preserves_rows_and_restores_constraints() {
    let db = Database::new();
    let mut conn = db.pool.get().unwrap();
    conn.revert_last_migration(MIGRATIONS).unwrap(); // Undo remaining object storage.
    conn.revert_last_migration(MIGRATIONS).unwrap(); // Undo raw object storage.
    conn.revert_last_migration(MIGRATIONS).unwrap(); // Undo pipeline storage.
    conn.revert_last_migration(MIGRATIONS).unwrap(); // Undo table renames.
    conn.revert_last_migration(MIGRATIONS).unwrap();
    for size in [0, i32::MAX] {
        let (processed_id, retrieval_id, _) = seed_processed_legacy(&mut conn);
        let id = Uuid::now_v7();
        diesel::sql_query(format!("INSERT INTO consolidated_sitemaps (id,processed_sitemap_id,retrieval_id,retailer_code,content_size,stats,consolidated_at) VALUES ($1,$2,$3,'ugreenca',{size},'{{\"product_count\":1,\"catalog_count\":0,\"content_count\":0,\"not_interested_count\":0,\"unknown_count\":0}}','2026-10-14T12:34:56.123456Z')"))
            .bind::<SqlUuid,_>(id).bind::<SqlUuid,_>(processed_id).bind::<SqlUuid,_>(retrieval_id).execute(&mut conn).unwrap();
        diesel::sql_query("INSERT INTO consolidated_sitemaps_data VALUES ($1, $2)")
            .bind::<SqlUuid, _>(id)
            .bind::<Jsonb, _>(json!({"product":["ž 🔎"]}))
            .execute(&mut conn)
            .unwrap();
    }
    let original = json_query(&mut conn, SNAPSHOT_LEGACY);
    conn.run_next_migration(MIGRATIONS).unwrap();
    let upgraded = json_query(&mut conn, SNAPSHOT_LEGACY);
    for (before, after) in original["records"]
        .as_array()
        .unwrap()
        .iter()
        .zip(upgraded["records"].as_array().unwrap())
    {
        assert_eq!(
            after["references"],
            json!({
                "processed_sitemap_id":before["processed_sitemap_id"],
                "retrieval_id":before["retrieval_id"],
                "retailer_code":before["retailer_code"],
            })
        );
        assert_eq!(
            after["storage"],
            json!({"content_size":before["content_size"]})
        );
        for name in [
            "processed_sitemap_id",
            "retrieval_id",
            "retailer_code",
            "content_size",
        ] {
            assert!(after.get(name).is_none());
        }
        for name in ["id", "stats", "consolidated_at"] {
            assert_eq!(before[name], after[name]);
        }
    }
    for name in ["payloads", "processed", "documents"] {
        assert_eq!(upgraded[name], original[name]);
    }
    assert!(
        conn.batch_execute("UPDATE consolidated_sitemaps SET \"references\"=NULL")
            .is_err()
    );
    assert!(
        conn.batch_execute("UPDATE consolidated_sitemaps SET storage=NULL")
            .is_err()
    );
    conn.revert_last_migration(MIGRATIONS).unwrap();
    assert_eq!(json_query(&mut conn, SNAPSHOT_LEGACY), original);
    assert!(conn.batch_execute("UPDATE consolidated_sitemaps SET processed_sitemap_id='00000000-0000-0000-0000-000000000000'").is_err());
    assert!(conn.batch_execute("UPDATE consolidated_sitemaps SET processed_sitemap_id=(SELECT id FROM processed_sitemaps LIMIT 1)").is_err());
    conn.run_next_migration(MIGRATIONS).unwrap();
    assert_eq!(json_query(&mut conn, SNAPSHOT_LEGACY), upgraded);
}

#[test]
#[ignore = "requires DATABASE_URL_TEST; run make test-postgres"]
fn postgres_legacy_reference_downgrade_rejects_dangling_parent() {
    let db = Database::new();
    let mut conn = db.pool.get().unwrap();
    conn.revert_last_migration(MIGRATIONS).unwrap(); // Undo remaining object storage.
    let (id, retrieval, _) = seed_processed(&mut conn);
    seed_legacy_consolidated(&mut conn, id, retrieval);
    diesel::sql_query("DELETE FROM sitemaps_processed WHERE id=$1")
        .bind::<SqlUuid, _>(id)
        .execute(&mut conn)
        .unwrap();
    assert_eq!(
        json_query(
            &mut conn,
            "SELECT to_jsonb(count(*)) AS value FROM sitemaps_consolidated"
        ),
        json!(1)
    );
    conn.revert_last_migration(MIGRATIONS).unwrap(); // Undo raw object storage.
    conn.revert_last_migration(MIGRATIONS).unwrap(); // Undo pipeline storage.
    conn.revert_last_migration(MIGRATIONS).unwrap(); // Undo table renames.
    let before = json_query(&mut conn, SNAPSHOT_LEGACY);
    // Restoring the previous FK cannot silently discard a dangling artifact.
    assert!(conn.revert_last_migration(MIGRATIONS).is_err());
    assert_eq!(json_query(&mut conn, SNAPSHOT_LEGACY), before);
    assert_eq!(
        conn.applied_migrations().unwrap()[0].to_string(),
        "20261015000008"
    );
    conn.batch_execute("DELETE FROM consolidated_sitemaps")
        .unwrap();
    assert_eq!(
        json_query(
            &mut conn,
            "SELECT to_jsonb(count(*)) AS value FROM consolidated_sitemaps_data"
        ),
        json!(0)
    );
}

#[test]
#[ignore = "requires DATABASE_URL_TEST; run make test-postgres"]
fn postgres_references_storage_downgrade_rejects_missing_or_invalid_values() {
    let db = Database::new();
    let mut conn = db.pool.get().unwrap();
    conn.revert_last_migration(MIGRATIONS).unwrap(); // Undo remaining object storage.
    let (id, retrieval, _) = seed_processed(&mut conn);
    seed_legacy_consolidated(&mut conn, id, retrieval);
    conn.revert_last_migration(MIGRATIONS).unwrap(); // Undo raw object storage.
    conn.revert_last_migration(MIGRATIONS).unwrap(); // Undo pipeline storage.
    conn.revert_last_migration(MIGRATIONS).unwrap(); // Undo table renames.
    let valid = json_query(
        &mut conn,
        "SELECT to_jsonb(c) AS value FROM consolidated_sitemaps c",
    );
    let mut missing_retrieval = valid["references"].clone();
    missing_retrieval
        .as_object_mut()
        .unwrap()
        .remove("retrieval_id");
    let mut invalid_retrieval = valid["references"].clone();
    invalid_retrieval["retrieval_id"] = json!("invalid-uuid");
    let mut missing_retailer = valid["references"].clone();
    missing_retailer
        .as_object_mut()
        .unwrap()
        .remove("retailer_code");
    for (references, storage) in [
        (json!({}), valid["storage"].clone()),
        (missing_retrieval, valid["storage"].clone()),
        (invalid_retrieval, valid["storage"].clone()),
        (missing_retailer, valid["storage"].clone()),
        (valid["references"].clone(), json!({})),
        (
            valid["references"].clone(),
            json!({"content_size":2147483648_i64}),
        ),
        (
            valid["references"].clone(),
            json!({"content_size":"invalid"}),
        ),
    ] {
        diesel::sql_query("UPDATE consolidated_sitemaps SET \"references\"=$1, storage=$2")
            .bind::<Jsonb, _>(references)
            .bind::<Jsonb, _>(storage)
            .execute(&mut conn)
            .unwrap();
        let before = json_query(&mut conn, SNAPSHOT_LEGACY);
        assert!(conn.revert_last_migration(MIGRATIONS).is_err());
        assert_eq!(json_query(&mut conn, SNAPSHOT_LEGACY), before);
        assert_eq!(
            conn.applied_migrations().unwrap()[0].to_string(),
            "20261015000008"
        );
    }
    diesel::sql_query("UPDATE consolidated_sitemaps SET \"references\"=$1, storage=$2")
        .bind::<Jsonb, _>(&valid["references"])
        .bind::<Jsonb, _>(&valid["storage"])
        .execute(&mut conn)
        .unwrap();
    conn.revert_last_migration(MIGRATIONS).unwrap();
}

fn seed_processed_legacy(conn: &mut PgConnection) -> (Uuid, Uuid, Value) {
    let id = Uuid::now_v7();
    let retrieval_id = Uuid::now_v7();
    let document = serde_json::to_value(SitemapDocument::default()).unwrap();
    diesel::sql_query("INSERT INTO processed_sitemaps (id,retrieval_id,retailer_code,document_size,url_count) VALUES ($1,$2,'ugreenca',0,0)")
        .bind::<SqlUuid,_>(id).bind::<SqlUuid,_>(retrieval_id).execute(conn).unwrap();
    diesel::sql_query("INSERT INTO processed_sitemaps_data (id,document) VALUES ($1,$2)")
        .bind::<SqlUuid, _>(id)
        .bind::<Jsonb, _>(&document)
        .execute(conn)
        .unwrap();
    (id, retrieval_id, document)
}

const SNAPSHOT_LEGACY: &str = "SELECT jsonb_build_object(
    'records', (SELECT jsonb_agg(to_jsonb(c) ORDER BY c.id) FROM consolidated_sitemaps c),
    'payloads', (SELECT jsonb_agg(to_jsonb(d) ORDER BY d.id) FROM consolidated_sitemaps_data d),
    'processed', (SELECT jsonb_agg(to_jsonb(p) ORDER BY p.id) FROM processed_sitemaps p),
    'documents', (SELECT jsonb_agg(to_jsonb(d) ORDER BY d.id) FROM processed_sitemaps_data d)
) AS value";
