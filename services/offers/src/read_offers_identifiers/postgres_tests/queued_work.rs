use diesel::{
    RunQueryDsl,
    connection::SimpleConnection,
    sql_types::{Jsonb, Text, Uuid as SqlUuid},
};
use diesel_migrations::MigrationHarness;
use poem::{http::StatusCode, test::TestClient};
use serde_json::{Value, json};
use testimony_infra::prelude::DbPool;
use uuid::Uuid;

use super::{before_identifier_migration, rows, table_snapshot};
use crate::{
    bootstrap::MIGRATIONS,
    infra::records::OfferEventRecord,
    postgres_tests::{inbox_test_database, inbox_test_snapshot as snapshot, run_queue},
};

const LEGACY_ID: Uuid = Uuid::from_u128(101);
const LEGACY_URL: &str = "https://example.com/Legacy?Variant=001#details";

fn seed_legacy_offer(pool: &DbPool) -> Value {
    // v5 predates the fact and identity-column migrations. Store its original shape.
    let record = json!({
        "type":"OfferCreated.v5",
        "data":{
            "id":LEGACY_ID,"retailer_code":"AnkerAu","url":LEGACY_URL,
            "title":"Legacy charger","price":{"amount":1999,"currency":"AUD"},
            "created_at":"2026-10-26T12:00:00Z","scraped_at":null,"images":[],
            "localization":{"target_market":null,"language":null},
            "gtin":"0012345678905","sku":" original ",
            "external_identifiers":{"shopify_product_gid":"001","shopify_product_variation_gid":"002","shopify_handle":" Exact Ž "}
        }
    });
    let _: OfferEventRecord = serde_json::from_value(record.clone()).unwrap();
    diesel::sql_query("INSERT INTO event_store(agg_id,sequence,payload_type,payload,metadata,recorded_on) VALUES ($1,1,$2,$3,'{}','2026-10-26T12:00:00Z')")
        .bind::<Text,_>(LEGACY_ID.to_string())
        .bind::<Text,_>(std::any::type_name::<OfferEventRecord>())
        .bind::<Jsonb,_>(&record)
        .execute(&mut pool.get().unwrap()).unwrap();
    diesel::sql_query("INSERT INTO read_offers(id,retailer_code,url,title,price_amount,currency,created_at,gtin,sku,shopify_product_gid,shopify_product_variation_gid,shopify_handle) SELECT (data->>'id')::uuid,'ankerau',data->>'url',data->>'title',(data->'price'->>'amount')::bigint,data->'price'->>'currency',(data->>'created_at')::timestamptz,data->>'gtin',data->>'sku',data->'external_identifiers'->>'shopify_product_gid',data->'external_identifiers'->>'shopify_product_variation_gid',data->'external_identifiers'->>'shopify_handle' FROM (SELECT $1::jsonb AS data) fixture")
        .bind::<Jsonb,_>(&record["data"])
        .execute(&mut pool.get().unwrap()).unwrap();
    pool.get().unwrap().batch_execute("INSERT INTO read_offer_identities(id,retailer_code,url) SELECT id,retailer_code,url FROM read_offers").unwrap();
    record
}

#[tokio::test]
#[ignore = "requires DATABASE_URL_TEST with CREATE SCHEMA permission"]
async fn queued_identifier_imports_cross_migration_without_losing_legacy_values() {
    // Cover an accepted inbox message and an already-dispatched command separately.
    for dispatched in [false, true] {
        let (_schema, pool) = inbox_test_database();
        before_identifier_migration(&pool);
        let legacy_record = seed_legacy_offer(&pool);
        let mut expected = rows(&pool, "read_offers");
        expected[0]["sku"] = json!("corrected");
        let (kernel, state) = crate::start_mulac(pool.clone()).unwrap();
        let client = TestClient::new(crate::app_with_reporting_token(state, "test".into()));
        let payload = json!({"retailer_code":"AnkerAu","url":LEGACY_URL,"title":"Legacy charger","price":{"amount":1999,"currency":"AUD"},"gtin":null,"sku":"corrected","external_identifiers":{"shopify_product_gid":null}});
        client.post("/api/inbox").body_json(&json!({"message_id":Uuid::new_v4(),"source":"legacy-import","routing_key":"ImportOfferDetails","payload":payload})).send().await.assert_status(StatusCode::ACCEPTED);
        if dispatched {
            run_queue(&client, "inbox", 100, 1).await;
        }
        let inbox = table_snapshot(&pool, "inbox_entries");
        let commands = table_snapshot(&pool, "command_entries");
        let history = table_snapshot(&pool, "event_store");
        kernel.shutdown();
        drop(client);
        drop(kernel);

        pool.get()
            .unwrap()
            .run_pending_migrations(MIGRATIONS)
            .unwrap();
        assert_eq!(table_snapshot(&pool, "inbox_entries"), inbox);
        assert_eq!(table_snapshot(&pool, "command_entries"), commands);
        assert_eq!(table_snapshot(&pool, "event_store"), history);
        let (kernel, state) = crate::start_mulac(pool.clone()).unwrap();
        let client = TestClient::new(crate::app_with_reporting_token(state, "test".into()));
        run_queue(&client, "inbox", 100, usize::from(!dispatched)).await;
        run_queue(&client, "commands", 100, 1).await;
        run_queue(&client, "events", 100, 1).await;
        assert_eq!(rows(&pool, "read_offer_identities"), expected);
        assert_eq!(rows(&pool, "read_offers"), expected);
        assert_eq!(
            snapshot(
                &pool,
                "SELECT payload AS data FROM event_store WHERE sequence=1"
            ),
            legacy_record
        );
        assert_eq!(
            snapshot(
                &pool,
                "SELECT jsonb_agg(sequence ORDER BY sequence) AS data FROM event_store"
            ),
            json!([1, 2])
        );
        assert_eq!(
            snapshot(
                &pool,
                "SELECT jsonb_agg(event_type) AS data FROM event_entries"
            ),
            json!(["OfferIdentifiersChanged"])
        );
        kernel.shutdown();
    }
}

#[tokio::test]
#[ignore = "requires DATABASE_URL_TEST with CREATE SCHEMA permission"]
async fn pending_complete_change_keeps_bytes_and_redelivery_keeps_one_outbox_identity() {
    let (_schema, pool) = inbox_test_database();
    before_identifier_migration(&pool);
    seed_legacy_offer(&pool);
    let mut expected = rows(&pool, "read_offers");
    for field in ["gtin", "shopify_product_gid", "shopify_handle"] {
        expected[0][field] = Value::Null;
    }
    expected[0]["sku"] = json!(" replacement ");
    expected[0]["shopify_product_variation_gid"] = json!("0009");
    let change = json!({"id":LEGACY_ID,"gtin":null,"sku":" replacement ","external_identifiers":{"shopify_product_gid":null,"shopify_product_variation_gid":"0009","shopify_handle":null}});
    let record = json!({"type":"OfferIdentifiersChanged.v1","data":change});
    let _: OfferEventRecord = serde_json::from_value(record.clone()).unwrap();
    let payload = format!("{}\n", serde_json::to_string_pretty(&change).unwrap());
    let event_id = Uuid::new_v4();
    let metadata = json!({"event_id":event_id,"sequence":2,"correlation_id":Uuid::new_v4(),"causation_id":Uuid::new_v4(),"source":"legacy-export"});
    diesel::sql_query("INSERT INTO event_store(agg_id,sequence,payload_type,payload,metadata,recorded_on) VALUES ($1,2,$2,$3,$4,'2026-10-26T12:01:00Z')")
        .bind::<Text,_>(LEGACY_ID.to_string())
        .bind::<Text,_>(std::any::type_name::<OfferEventRecord>())
        .bind::<Jsonb,_>(&record).bind::<Jsonb,_>(&metadata)
        .execute(&mut pool.get().unwrap()).unwrap();
    diesel::sql_query("INSERT INTO event_entries(id,event_type,payload,meta) VALUES ($1,'OfferIdentifiersChanged',$2,$3)")
        .bind::<SqlUuid,_>(event_id).bind::<Text,_>(&payload).bind::<Jsonb,_>(&metadata)
        .execute(&mut pool.get().unwrap()).unwrap();
    let queued = table_snapshot(&pool, "event_entries");
    let history = table_snapshot(&pool, "event_store");
    pool.get()
        .unwrap()
        .run_pending_migrations(MIGRATIONS)
        .unwrap();
    assert_eq!(table_snapshot(&pool, "event_entries"), queued);
    assert_eq!(table_snapshot(&pool, "event_store"), history);
    let (kernel, state) = crate::start_mulac(pool.clone()).unwrap();
    let client = TestClient::new(crate::app_with_reporting_token(state, "test".into()));
    run_queue(&client, "events", 100, 1).await;
    assert_eq!(rows(&pool, "read_offer_identities"), expected);
    assert_eq!(rows(&pool, "read_offers"), expected);
    let outbox = table_snapshot(&pool, "outbox_entries");
    assert_eq!(outbox.as_array().unwrap().len(), 1);
    assert_eq!(outbox[0]["id"], json!(event_id));
    assert_eq!(outbox[0]["payload"], payload);
    let mut expected_metadata = metadata.clone();
    expected_metadata["message_id"] = json!(event_id);
    expected_metadata["event_type"] = json!("OfferIdentifiersChanged");
    expected_metadata["routing_key"] = json!("customer_facing");
    expected_metadata["content_type"] = json!("application/json");
    assert_eq!(outbox[0]["meta"], expected_metadata);

    // Only this complete change is redelivered; creation idempotency is unrelated.
    diesel::sql_query(
        "UPDATE event_entries SET status=0,attempts=0,scheduled_at=now() WHERE id=$1",
    )
    .bind::<SqlUuid, _>(event_id)
    .execute(&mut pool.get().unwrap())
    .unwrap();
    run_queue(&client, "events", 100, 1).await;
    assert_eq!(rows(&pool, "read_offer_identities"), expected);
    assert_eq!(rows(&pool, "read_offers"), expected);
    assert_eq!(table_snapshot(&pool, "outbox_entries"), outbox);
    assert_eq!(table_snapshot(&pool, "event_store"), history);
    let delivered = snapshot(
        &pool,
        "SELECT jsonb_build_object('payload',payload,'meta',meta,'status',status) AS data FROM event_entries",
    );
    assert_eq!(
        delivered,
        json!({"payload":payload,"meta":metadata,"status":5})
    );
    kernel.shutdown();
}
