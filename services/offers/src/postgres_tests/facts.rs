use diesel::connection::SimpleConnection;
use diesel_migrations::MigrationHarness;
use poem::{http::StatusCode, test::TestClient};
use serde_json::{Value, json};
use testimony_infra::prelude::DbPool;

use super::{inbox_test_database, inbox_test_snapshot as snapshot, run_queue};
use crate::bootstrap::MIGRATIONS;

fn before_fact_migrations(pool: &DbPool) {
    let mut connection = pool.get().unwrap();
    while connection
        .revert_last_migration(MIGRATIONS)
        .unwrap()
        .to_string()
        != "20261027000002"
    {}
}

#[test]
#[ignore = "requires DATABASE_URL_TEST with CREATE SCHEMA permission"]
fn fact_migrations_preserve_legacy_rows_and_pending_work_through_round_trip() {
    let (_schema, pool) = inbox_test_database();
    before_fact_migrations(&pool);
    pool.get().unwrap().batch_execute(
        "INSERT INTO read_offers (id,retailer_code,url,title,price_amount,currency,created_at,gtin) VALUES ('00000000-0000-0000-0000-000000000001','ankerau','https://example.com/legacy','Legacy charger',1999,'AUD','2026-10-26T12:00:00Z','0012345678905'); INSERT INTO command_entries(id,command_type,payload) VALUES ('00000000-0000-0000-0000-000000000002','LegacyPending','{}')"
    ).unwrap();
    let legacy = snapshot(&pool, "SELECT to_jsonb(o) AS data FROM read_offers o");
    let pending = snapshot(&pool, "SELECT to_jsonb(c) AS data FROM command_entries c");
    pool.get()
        .unwrap()
        .run_pending_migrations(MIGRATIONS)
        .unwrap();
    let mut upgraded = snapshot(&pool, "SELECT to_jsonb(o) AS data FROM read_offers o");
    let content = json!([{"language":null,"title":"Legacy charger","description":null}]);
    assert_eq!(
        upgraded.as_object_mut().unwrap().remove("content"),
        Some(content.clone())
    );
    for column in [
        "was_price_amount",
        "was_price_currency",
        "availability",
        "item_condition",
        "description",
    ] {
        assert_eq!(
            upgraded.as_object_mut().unwrap().remove(column),
            Some(Value::Null)
        );
    }
    assert_eq!(upgraded, legacy);
    pool.get().unwrap().batch_execute("UPDATE read_offers SET was_price_amount=2999,was_price_currency='AUD',availability='{\"other\":\"http://schema.org/OutOfStock\"}',item_condition='\"new\"',description=' <p>Exact ž &amp; source</p> '").unwrap();
    before_fact_migrations(&pool);
    assert_eq!(
        snapshot(&pool, "SELECT to_jsonb(o) AS data FROM read_offers o"),
        legacy
    );
    assert_eq!(
        snapshot(&pool, "SELECT to_jsonb(c) AS data FROM command_entries c"),
        pending
    );
    pool.get()
        .unwrap()
        .run_pending_migrations(MIGRATIONS)
        .unwrap();
    let mut restored = legacy;
    restored["content"] = content;
    for column in [
        "was_price_amount",
        "was_price_currency",
        "availability",
        "item_condition",
        "description",
    ] {
        restored[column] = Value::Null;
    }
    assert_eq!(
        snapshot(&pool, "SELECT to_jsonb(o) AS data FROM read_offers o"),
        restored
    );
    assert_eq!(
        snapshot(&pool, "SELECT to_jsonb(c) AS data FROM command_entries c"),
        pending
    );
}

fn fact_projection(pool: &DbPool) -> Value {
    snapshot(
        pool,
        "SELECT jsonb_build_object('price_amount',price_amount,'currency',currency,'was_price_amount',was_price_amount,'was_price_currency',was_price_currency,'availability',availability,'item_condition',item_condition,'description',description) AS data FROM read_offers",
    )
}

async fn import(client: &TestClient<poem::Route>, input: &Value, expected_events: usize) {
    client
        .post("/api/tmp-import")
        .body_json(input)
        .send()
        .await
        .assert_status(StatusCode::ACCEPTED);
    run_queue(client, "inbox", 100, 1).await;
    run_queue(client, "commands", 100, 1).await;
    run_queue(client, "events", 100, expected_events).await;
}

#[tokio::test]
#[ignore = "requires DATABASE_URL_TEST with CREATE SCHEMA permission"]
async fn created_facts_survive_legacy_imports_then_clear_and_publish_once() {
    let (_schema, pool) = inbox_test_database();
    let (kernel, state) = crate::start_mulac(pool.clone()).unwrap();
    let client = TestClient::new(crate::app_with_reporting_token(state, "test".into()));
    let legacy = json!({"retailer_code":"ankerau","title":"Charger","price_amount":1999,"currency":"AUD","url":"https://example.com/facts"});
    let observed = json!({"was_price":{"amount":2999,"currency":"AUD"},"availability":{"other":" exact status "},"item_condition":"new","content":{"description":" <p>Exact &amp; source ž</p>\n"}});
    let mut input = legacy.clone();
    input
        .as_object_mut()
        .unwrap()
        .extend(observed.as_object().unwrap().clone());
    import(&client, &input, 1).await;
    let populated = json!({"price_amount":1999,"currency":"AUD","was_price_amount":2999,"was_price_currency":"AUD","availability":{"other":" exact status "},"item_condition":"new","description":" <p>Exact &amp; source ž</p>\n"});
    assert_eq!(fact_projection(&pool), populated);
    let creation = snapshot(
        &pool,
        "SELECT payload::jsonb AS data FROM outbox_entries WHERE meta->>'event_type'='OfferCreated'",
    );
    for (field, expected) in observed.as_object().unwrap() {
        if field == "content" {
            assert_eq!(
                creation[field],
                json!([{"language":null,"title":"Charger","description":expected["description"]}])
            );
        } else {
            assert_eq!(&creation[field], expected);
        }
    }
    import(&client, &legacy, 0).await;
    assert_eq!(fact_projection(&pool), populated);
    for field in ["was_price", "availability", "item_condition"] {
        input[field] = Value::Null;
    }
    input["content"] = json!({"description":null});
    import(&client, &input, 4).await;
    import(&client, &input, 0).await;
    let cleared = json!({"price_amount":1999,"currency":"AUD","was_price_amount":null,"was_price_currency":null,"availability":null,"item_condition":null,"description":null});
    assert_eq!(fact_projection(&pool), cleared);
    let messages = snapshot(
        &pool,
        "SELECT jsonb_object_agg(meta->>'event_type',payload::jsonb) AS data FROM outbox_entries",
    );
    assert_eq!(messages.as_object().unwrap().len(), 5);
    assert_eq!(
        messages["OfferPricingChanged"]["pricing"],
        json!({"price":{"amount":1999,"currency":"AUD"},"was_price":null})
    );
    assert_eq!(
        messages["OfferAvailabilityChanged"].get("availability"),
        Some(&Value::Null)
    );
    assert_eq!(
        messages["OfferItemConditionChanged"].get("item_condition"),
        Some(&Value::Null)
    );
    assert_eq!(
        messages["OfferContentsChanged"]["content"],
        json!([{"language":null,"title":"Charger","description":null}])
    );
    assert_eq!(
        snapshot(
            &pool,
            "SELECT jsonb_agg((meta->>'sequence')::bigint ORDER BY (meta->>'sequence')::bigint) AS data FROM outbox_entries"
        ),
        json!([1, 2, 3, 4, 5])
    );
    let outbox = snapshot(
        &pool,
        "SELECT jsonb_agg(to_jsonb(o) ORDER BY id) AS data FROM outbox_entries o",
    );
    pool.get().unwrap().batch_execute("UPDATE event_entries SET status=0,attempts=0,scheduled_at=now() WHERE event_type <> 'OfferCreated'").unwrap();
    run_queue(&client, "events", 100, 4).await;
    assert_eq!(fact_projection(&pool), cleared);
    assert_eq!(
        snapshot(
            &pool,
            "SELECT jsonb_agg(to_jsonb(o) ORDER BY id) AS data FROM outbox_entries o"
        ),
        outbox
    );
    assert_eq!(
        snapshot(&pool, "SELECT to_jsonb(count(*)) AS data FROM event_store"),
        5
    );
    kernel.shutdown();
}

#[tokio::test]
#[ignore = "requires DATABASE_URL_TEST with CREATE SCHEMA permission"]
async fn independent_fact_imports_preserve_neighbors_and_replay_exact_outbox_messages() {
    let (_schema, pool) = inbox_test_database();
    let (kernel, state) = crate::start_mulac(pool.clone()).unwrap();
    let client = TestClient::new(crate::app_with_reporting_token(state, "test".into()));
    let mut legacy = json!({
        "retailer_code":"ankerau","title":"Charger","price_amount":1999,"currency":"AUD",
        "url":"https://example.com/independent-facts","images":["https://example.com/image.png"],
        "gtin":"0012345678905","sku":" exact ž ","external_identifiers":{"shopify_handle":"charger"},
        "scraped_at":"2026-10-27T12:00:00Z","localization":{"target_market":{"kind":"country","code":"AU"},"language":"en"}
    });
    let mut initial = legacy.clone();
    initial.as_object_mut().unwrap().extend(
        json!({
            "was_price":{"amount":2999,"currency":"AUD"},"availability":"available",
            "item_condition":"new","content":{"description":"Original description"}
        })
        .as_object()
        .unwrap()
        .clone(),
    );
    import(&client, &initial, 1).await;
    let read_offer = || snapshot(&pool, "SELECT to_jsonb(o) AS data FROM read_offers o");
    let neighbors = || {
        snapshot(
            &pool,
            "SELECT jsonb_build_object('identities',(SELECT jsonb_agg(to_jsonb(i)) FROM read_offer_identities i),'images',(SELECT jsonb_agg(to_jsonb(i) ORDER BY position) FROM read_offer_images i)) AS data",
        )
    };
    let original_neighbors = neighbors();
    let mut expected = read_offer();
    let id = expected["id"].clone();
    for (index, (kind, fields, columns, event_fields)) in [
        (
            "OfferAvailabilityChanged",
            json!({"availability":{"other":"http://schema.org/OutOfStock"}}),
            json!({"availability":{"other":"http://schema.org/OutOfStock"}}),
            json!({"availability":{"other":"http://schema.org/OutOfStock"}}),
        ),
        (
            "OfferItemConditionChanged",
            json!({"item_condition":{"other":" Refurbished ž "}}),
            json!({"item_condition":{"other":" Refurbished ž "}}),
            json!({"item_condition":{"other":" Refurbished ž "}}),
        ),
        (
            "OfferContentsChanged",
            json!({"content":{"description":""}}),
            json!({"description":"","content":[{"language":"en","title":"Charger","description":""}]}),
            json!({"content":[{"language":"en","title":"Charger","description":""}],"language":"en"}),
        ),
        (
            "OfferContentsChanged",
            json!({"content":{"description":" \t\n"}}),
            json!({"description":" \t\n","content":[{"language":"en","title":"Charger","description":" \t\n"}]}),
            json!({"content":[{"language":"en","title":"Charger","description":" \t\n"}],"language":"en"}),
        ),
        (
            "OfferPricingChanged",
            json!({"was_price":{"amount":3999,"currency":"AUD"}}),
            json!({"was_price_amount":3999}),
            json!({"pricing":{"price":{"amount":1999,"currency":"AUD"},"was_price":{"amount":3999,"currency":"AUD"}}}),
        ),
        (
            "OfferPricingChanged",
            json!({"price_amount":i64::MAX-1,"currency":"JPY","was_price":{"amount":i64::MAX,"currency":"JPY"}}),
            json!({"price_amount":i64::MAX-1,"currency":"JPY","was_price_amount":i64::MAX,"was_price_currency":"JPY"}),
            json!({"pricing":{"price":{"amount":i64::MAX-1,"currency":"JPY"},"was_price":{"amount":i64::MAX,"currency":"JPY"}}}),
        ),
    ].into_iter().enumerate() {
        let mut input = legacy.clone();
        input.as_object_mut().unwrap().extend(fields.as_object().unwrap().clone());
        import(&client, &input, 1).await;
        expected.as_object_mut().unwrap().extend(columns.as_object().unwrap().clone());
        assert_eq!(read_offer(), expected);
        assert_eq!(neighbors(), original_neighbors);
        let mut event = event_fields;
        event["id"] = id.clone();
        assert_eq!(snapshot(&pool, "SELECT jsonb_build_object('type',meta->>'event_type','payload',payload::jsonb,'sequence',(meta->>'sequence')::bigint) AS data FROM outbox_entries ORDER BY (meta->>'sequence')::bigint DESC LIMIT 1"), json!({"type":kind,"payload":event,"sequence":index+2}));
        let outbox = snapshot(&pool, "SELECT jsonb_agg(to_jsonb(o) ORDER BY id) AS data FROM outbox_entries o");

        // Identical and legacy observations add no events; a changed payable
        // price remains the current price when omitting the optional facts.
        legacy["price_amount"] = input["price_amount"].clone();
        legacy["currency"] = input["currency"].clone();
        import(&client, &input, 0).await;
        import(&client, &legacy, 0).await;
        pool.get().unwrap().batch_execute("UPDATE event_entries SET status=0,attempts=0,scheduled_at=now() WHERE (meta->>'sequence')::bigint=(SELECT max((meta->>'sequence')::bigint) FROM event_entries)").unwrap();
        run_queue(&client, "events", 100, 1).await;
        assert_eq!(read_offer(), expected);
        assert_eq!(neighbors(), original_neighbors);
        assert_eq!(snapshot(&pool, "SELECT jsonb_agg(to_jsonb(o) ORDER BY id) AS data FROM outbox_entries o"), outbox);
        assert_eq!(snapshot(&pool, "SELECT to_jsonb(count(*)) AS data FROM event_store"), json!(index+2));
    }
    kernel.shutdown();
}
