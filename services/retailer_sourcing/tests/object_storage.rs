mod support;
use retailer_sourcing::object_storage::{
    Error, ObjectStorage, ObjectStorageMetadata, RAW_SITEMAPS_BUCKET,
};
use serde_json::json;
use support::Server;
use uuid::Uuid;

#[test]
fn uploads_exact_input_and_validates_every_part_of_the_reference() {
    let server = Server::new();
    let client = server.client();
    let local_id = Uuid::now_v7();
    let body = "<urlset>ž 🔎\n</urlset>";
    let object_id = client.create(RAW_SITEMAPS_BUCKET, local_id, body).unwrap();
    let metadata = ObjectStorageMetadata {
        content_size: body.len() as u64,
        object_storage_id: object_id,
    };
    assert_ne!(object_id, local_id);
    assert_eq!(
        client.create(RAW_SITEMAPS_BUCKET, local_id, body).unwrap(),
        object_id
    );
    assert_eq!(
        client
            .get(RAW_SITEMAPS_BUCKET, local_id, &metadata)
            .unwrap(),
        body
    );
    let original = {
        let data = server.data.lock().unwrap();
        assert_eq!(
            data.uploads[0],
            (
                "sitemaps_raw".into(),
                json!({"idempotency_key":local_id,"payload":body,"metadata":{}})
            )
        );
        assert_eq!(data.reads[0], ("sitemaps_raw".into(), object_id));
        data.records[&object_id].clone()
    };
    for (field, value) in [
        ("id", json!(Uuid::now_v7())),
        ("bucket", json!("sitemap_raw")),
        ("idempotency_key", json!(Uuid::now_v7())),
        ("payload", json!("short")),
    ] {
        let mut bad = original.clone();
        bad[field] = value;
        server.data.lock().unwrap().get_response = Some((200, bad.to_string()));
        assert!(matches!(
            client.get(RAW_SITEMAPS_BUCKET, local_id, &metadata),
            Err(Error::InvalidRecord)
        ));
    }
    for (status, body) in [(404, "{}"), (503, "{}"), (200, "not JSON")] {
        server.data.lock().unwrap().get_response = Some((status, body.into()));
        assert!(matches!(
            client.get(RAW_SITEMAPS_BUCKET, local_id, &metadata),
            Err(Error::Client(_))
        ));
    }
}

#[test]
fn configuration_rejects_invalid_url_or_token() {
    let server = Server::new();
    assert!(ObjectStorage::new("not a URL", "token", server.runtime.handle().clone()).is_err());
    assert!(ObjectStorage::new(&server.url, "", server.runtime.handle().clone()).is_err());
}

#[test]
fn pipeline_configuration_is_checked_before_connecting_or_migrating() {
    let logs = std::env::temp_dir().join(format!("sourcing-startup-{}", Uuid::now_v7()));
    for missing in ["OBJECT_STORAGE_URL", "OBJECT_STORAGE_TOKEN"] {
        let output = std::process::Command::new(env!("CARGO_BIN_EXE_retailer_sourcing"))
            .env("DATABASE_URL", "not a database URL")
            .env("OBJECT_STORAGE_URL", "http://127.0.0.1:1")
            .env("OBJECT_STORAGE_TOKEN", "test-token")
            .env_remove(missing)
            .env("LOG_DIR", &logs)
            .output()
            .unwrap();
        assert!(!output.status.success());
        let stderr = String::from_utf8(output.stderr).unwrap();
        assert!(stderr.contains(missing), "{stderr}");
        assert!(!stderr.contains("database connection failed"), "{stderr}");
    }
    std::fs::remove_dir_all(logs).unwrap();
}

#[test]
fn offer_replacements_get_distinct_external_ids_and_validate_typed_metadata() {
    use retailer_sourcing::object_storage::{OfferStorageMetadata, RAW_OFFERS_BUCKET};
    let server = Server::new();
    let client = server.client();
    let first = client.create_offer(RAW_OFFERS_BUCKET, "first ž").unwrap();
    let second = client.create_offer(RAW_OFFERS_BUCKET, "second 🔎").unwrap();
    assert_ne!(first.external_id, second.external_id);
    assert_ne!(first.object_storage_id, second.object_storage_id);
    assert_eq!(
        client.get_offer(RAW_OFFERS_BUCKET, &first).unwrap(),
        "first ž"
    );
    assert_eq!(
        client.get_offer(RAW_OFFERS_BUCKET, &second).unwrap(),
        "second 🔎"
    );
    let wrong_key = OfferStorageMetadata {
        external_id: first.external_id,
        ..second
    };
    assert!(matches!(
        client.get_offer(RAW_OFFERS_BUCKET, &wrong_key),
        Err(Error::InvalidRecord)
    ));
    let json = serde_json::to_value(second).unwrap();
    for field in ["content_size", "object_storage_id", "external_id"] {
        let mut missing = json.clone();
        missing.as_object_mut().unwrap().remove(field);
        assert!(serde_json::from_value::<OfferStorageMetadata>(missing).is_err());
        for invalid in [
            json!(null),
            json!(-1),
            json!(1.5),
            json!("invalid"),
            json!({}),
        ] {
            let mut bad = json.clone();
            bad[field] = invalid;
            assert!(serde_json::from_value::<OfferStorageMetadata>(bad).is_err());
        }
    }
    let mut extra = json;
    extra["body_size"] = json!(7);
    assert!(serde_json::from_value::<OfferStorageMetadata>(extra).is_err());
}
