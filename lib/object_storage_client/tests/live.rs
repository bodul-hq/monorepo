use object_storage_client::{Client, CreateRecord, ErrorKind, Transport, Uuid};

/// Run only against a disposable server/database: immutable artifacts remain.
#[tokio::test]
#[ignore = "requires OBJECT_STORAGE_URL and OBJECT_STORAGE_TOKEN; run make test-live"]
async fn created_record_round_trips_all_input_fields() {
    let url = std::env::var("OBJECT_STORAGE_URL").expect("OBJECT_STORAGE_URL is required");
    let token = std::env::var("OBJECT_STORAGE_TOKEN").expect("OBJECT_STORAGE_TOKEN is required");
    let rest = Client::new(&url, &token, Transport::Rest).unwrap();
    let graphql = rest.with_transport(Transport::Graphql);
    let bucket = "sitemap_raw";

    for writer in [&rest, &graphql] {
        let input = CreateRecord {
            idempotency_key: Uuid::now_v7(),
            payload: "<urlset>\n  Živjo 雪 🌍 & \"quotes\" \\ slash\n</urlset>\n".into(),
            metadata: serde_json::from_str(
                r#"{
                    "source": "client round-trip integration",
                    "nested": {"enabled": true, "missing": null, "text": "Živjo 雪 🌍"},
                    "array": [1, "two", false, null, {"empty": []}],
                    "integer": 123456789012345678901234567890,
                    "decimal": 0.123456789012345678901234567890
                }"#,
            )
            .unwrap(),
        };

        let id = writer.create(bucket, &input).await.unwrap();
        assert_eq!(id.get_version_num(), 7);
        let record = writer.get(bucket, id).await.unwrap();
        assert_eq!(record.id, id);
        assert_eq!(record.bucket, bucket);
        assert_eq!(record.idempotency_key, input.idempotency_key);
        assert_eq!(record.payload, input.payload);
        assert_eq!(record.metadata, input.metadata);
        assert!(record.created_at.timestamp() > 0);

        // Both APIs must expose the same persisted record, including created_at.
        for reader in [&rest, &graphql] {
            assert_eq!(reader.get(bucket, id).await.unwrap(), record);
        }
    }
}

/// Run only against a disposable server/database: immutable artifacts remain.
#[tokio::test]
#[ignore = "requires OBJECT_STORAGE_URL and OBJECT_STORAGE_TOKEN; run make test-live"]
async fn both_transports_share_creation_retries_and_retrieval() {
    let url = std::env::var("OBJECT_STORAGE_URL").expect("OBJECT_STORAGE_URL is required");
    let token = std::env::var("OBJECT_STORAGE_TOKEN").expect("OBJECT_STORAGE_TOKEN is required");
    let rest = Client::new(&url, &token, Transport::Rest).unwrap();
    let graphql = rest.with_transport(Transport::Graphql);
    rest.health().await.unwrap();
    graphql.health().await.unwrap();
    for (writer, reader) in [(&rest, &graphql), (&graphql, &rest)] {
        let input = CreateRecord {
            idempotency_key: Uuid::now_v7(), payload: "Client integration: Živjo 雪 🌍".into(),
            metadata: serde_json::from_str(r#"{"number":123456789012345678901234567890,"decimal":0.123456789012345678901234567890}"#).unwrap(),
        };
        let id = writer.create("sitemap_raw", &input).await.unwrap();
        assert_eq!(reader.create("sitemap_raw", &input).await.unwrap(), id);
        let record = reader.get("sitemap_raw", id).await.unwrap();
        assert_eq!(record.id, id);
        assert_eq!(record.idempotency_key, input.idempotency_key);
        assert_eq!(record.payload, input.payload);
        assert_eq!(record.metadata, input.metadata);
        let mut conflict = input.clone();
        conflict.payload.push('!');
        assert_eq!(
            writer
                .create("sitemap_raw", &conflict)
                .await
                .unwrap_err()
                .kind(),
            ErrorKind::Conflict
        );
        assert_eq!(
            reader
                .get("offer_details_raw", id)
                .await
                .unwrap_err()
                .kind(),
            ErrorKind::NotFound
        );
        assert_eq!(
            reader
                .get("sitemap_raw", Uuid::now_v7())
                .await
                .unwrap_err()
                .kind(),
            ErrorKind::NotFound
        );
    }
    for transport in [Transport::Rest, Transport::Graphql] {
        let bad_auth = Client::new(&url, "incorrect", transport).unwrap();
        assert_eq!(
            bad_auth
                .get("sitemap_raw", Uuid::now_v7())
                .await
                .unwrap_err()
                .kind(),
            ErrorKind::Unauthorized
        );
    }
}
