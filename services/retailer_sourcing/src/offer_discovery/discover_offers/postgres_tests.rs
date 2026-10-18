use super::handler::Store;
use super::store::{PgStore, StoreError};
use crate::platform::db::Json;
use crate::platform::object_storage::{CONSOLIDATED_SITEMAPS_BUCKET, ObjectStorageMetadata};
use crate::platform::test_db::Database;
use crate::platform::test_objects::Server;
use diesel::prelude::*;
use diesel::sql_types::{Jsonb, Uuid as SqlUuid};
use serde_json::json;
use uuid::Uuid;

#[test]
#[ignore = "requires DATABASE_URL_TEST; run make test-postgres"]
fn postgres_discovery_returns_entities_and_rejects_bad_content() {
    let db = Database::new();
    let server = Server::new();
    let objects = server.client();
    let store = PgStore::new(db.pool.clone(), objects.clone());
    let mut conn = db.pool.get().unwrap();
    assert!(matches!(
        store.consolidated_sitemap(Uuid::now_v7()),
        Err(StoreError::Database(diesel::result::Error::NotFound))
    ));
    for (content, valid) in [
        (
            json!({"product":["https://shop.test/ž"],"unknown":[]}),
            true,
        ),
        (json!({"product":[]}), true),
        (json!({"product":42}), false),
    ] {
        let id = Uuid::now_v7();
        let body = content.to_string();
        let object_storage_id = objects
            .create(CONSOLIDATED_SITEMAPS_BUCKET, id, &body)
            .unwrap();
        diesel::sql_query("INSERT INTO sitemaps_consolidated (id,stats,\"references\",storage) VALUES ($1,'{}','{}',$2)")
            .bind::<SqlUuid,_>(id).bind::<Jsonb,_>(Json(ObjectStorageMetadata { content_size: body.len() as u64, object_storage_id }))
            .execute(&mut conn).unwrap();
        let result = store.consolidated_sitemap(id);
        if valid {
            let entity = result.unwrap();
            assert_eq!(entity.id, id);
            assert_eq!(entity.object_storage_id, object_storage_id);
            assert_eq!(json!(entity.product_urls), content["product"]);
        } else {
            assert!(matches!(result, Err(StoreError::Content(_))));
        }
    }
}
