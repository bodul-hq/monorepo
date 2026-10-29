use super::projection::{advance, apply, created, offer, projection_counts, replay_inbox, submit};
use super::*;

async fn browse(client: &TestClient<poem::Route>) -> Value {
    let response = client.get("/api/offers").send().await;
    response.assert_status_is_ok();
    let result: Value = response.json().await.value().deserialize();
    result["items"][0].clone()
}

async fn assert_reads(client: &TestClient<poem::Route>, expected: &Value) {
    let actual = browse(client).await;
    assert_eq!(&actual, expected);
    let response = client.get("/api/offers/search?q=charger").send().await;
    response.assert_status_is_ok();
    let search: Value = response.json().await.value().deserialize();
    assert_eq!(search["items"], json!([expected]));
    let mut facts = json!({
        "price":expected["price"],"was_price":expected["was_price"],
        "availability":expected["availability"],"item_condition":expected["item_condition"],
        "content":with_urls(expected["content"].clone(), &expected["url"])
    });
    for field in ["availability", "item_condition"] {
        if let Some(kind) = facts[field].get_mut("kind") {
            *kind = json!(kind.as_str().unwrap().to_uppercase());
        }
    }
    for field in ["offers", "searchOffers(q:\"charger\")"] {
        let query = format!(
            "{{ result: {field} {{ items {{ price {{amount currency}} was_price: wasPrice {{amount currency}} availability {{kind raw}} item_condition: itemCondition {{kind raw}} content {{language title description url}} }} }} }}"
        );
        let response = client
            .post("/graphql")
            .body_json(&json!({"query":query}))
            .send()
            .await;
        response.assert_status_is_ok();
        response
            .assert_json(json!({"data":{"result":{"items":[facts]}}}))
            .await;
    }
}

fn populated(id: Uuid) -> Value {
    let mut payload = created(id);
    payload["title"] = json!("USB charger");
    payload["price"] = json!({"amount":1999,"currency":"AUD"});
    payload["was_price"] = json!({"amount":2999,"currency":"AUD"});
    payload["availability"] = json!({"other":"http://schema.org/OutOfStock"});
    payload["item_condition"] = json!({"other":"http://schema.org/NewCondition"});
    payload["content"] = json!({"description":" <p>Original ž &amp; source</p>\n"});
    payload["images"] = json!(["https://example.com/charger.png"]);
    payload["gtin"] = json!("0012345678905");
    payload["sku"] = json!("sku-001");
    payload["localization"] =
        json!({"language":"en","target_market":{"kind":"country","code":"AU"}});
    payload
}

#[tokio::test]
#[ignore = "requires DATABASE_URL_TEST and pg_trgm in customer_facing_extensions"]
async fn independent_status_and_content_updates_preserve_neighbors_and_historical_other_values() {
    let (_schema, pool) = database(true);
    let (kernel, state) = start_mulac(pool.clone()).unwrap();
    let client = TestClient::new(crate::app(state));
    let id = Uuid::now_v7();
    let payload = populated(id);
    apply(&client, "OfferCreated", &payload).await;
    let mut stored = offer(&pool);
    let mut expected = browse(&client).await;
    assert_eq!(stored["availability"], payload["availability"]);
    assert_eq!(stored["item_condition"], payload["item_condition"]);
    assert_eq!(
        expected["availability"],
        json!({"kind":"other","raw":"http://schema.org/OutOfStock"})
    );
    assert_eq!(
        expected["item_condition"],
        json!({"kind":"other","raw":"http://schema.org/NewCondition"})
    );
    assert_reads(&client, &expected).await;
    for (kind, field, states) in [
        (
            "OfferAvailabilityChanged",
            "availability",
            vec![
                json!("available"),
                json!("unavailable"),
                json!("in_stock"),
                json!("out_of_stock"),
                json!("sold_out"),
                json!({"other":"http://schema.org/OutOfStock"}),
                json!({"other":" Stock status ž \t"}),
                Value::Null,
                json!("in_stock"),
            ],
        ),
        (
            "OfferItemConditionChanged",
            "item_condition",
            vec![
                json!("new"),
                json!({"other":"http://schema.org/NewCondition"}),
                json!({"other":" Refurbished ž "}),
                Value::Null,
                json!("new"),
            ],
        ),
    ] {
        for value in states {
            let mut change = json!({"id":id});
            change[field] = value.clone();
            apply(&client, kind, &change).await;
            stored[field] = value.clone();
            expected[field] = match &value {
                Value::Null => Value::Null,
                Value::String(kind) => json!({"kind":kind,"raw":null}),
                _ => json!({"kind":"other","raw":value["other"]}),
            };
            assert_eq!(offer(&pool), stored);
            assert_reads(&client, &expected).await;
        }
    }
    for description in [
        Some(""),
        Some(" \t\r\n"),
        Some(" <p>ž &amp; <script>source()</script></p>\n"),
        None,
    ] {
        let content = json!({"description":description});
        apply(
            &client,
            "OfferContentChanged",
            &json!({"id":id,"content":content}),
        )
        .await;
        stored["description"] = json!(description);
        stored["content"][0]["description"] = json!(description);
        expected["content"] = with_urls(stored["content"].clone(), &expected["url"]);
        assert_eq!(offer(&pool), stored);
        assert_reads(&client, &expected).await;
    }
    kernel.shutdown();
}

#[tokio::test]
#[ignore = "requires DATABASE_URL_TEST and pg_trgm in customer_facing_extensions"]
async fn exact_pricing_pairs_change_independently_and_legacy_prices_only_hide_comparisons() {
    let (_schema, pool) = database(true);
    let (kernel, state) = start_mulac(pool.clone()).unwrap();
    let client = TestClient::new(crate::app(state));
    let id = Uuid::now_v7();
    apply(&client, "OfferCreated", &populated(id)).await;
    let mut stored = offer(&pool);
    let mut expected = browse(&client).await;
    for (amount, comparison, currency) in [
        (1999_i64, Some(3999), "AUD"), // Comparison-only change.
        (1499, Some(2499), "AUD"),
        (9_007_199_254_740_993, Some(i64::MAX), "JPY"),
        (0, Some(1), "VND"),
        (0, None, "VND"),
        (9_007_199_254_740_993, Some(i64::MAX), "JPY"),
    ] {
        let price = json!({"amount":amount,"currency":currency});
        let was_price = comparison.map(|amount| json!({"amount":amount,"currency":currency}));
        apply(
            &client,
            "OfferPricingChanged",
            &json!({"id":id,"pricing":{"price":price,"was_price":was_price}}),
        )
        .await;
        stored["price_amount"] = json!(amount);
        stored["currency"] = json!(currency);
        stored["was_price_amount"] = json!(comparison);
        stored["was_price_currency"] = json!(comparison.map(|_| currency));
        expected["price"] = json!({"amount":amount.to_string(),"currency":currency});
        expected["was_price"] = json!(
            comparison.map(|amount| json!({"amount":amount.to_string(),"currency":currency}))
        );
        assert_eq!(offer(&pool), stored);
        assert_reads(&client, &expected).await;
    }
    for (amount, currency, visible) in [
        (1, "USD", false),
        (i64::MAX, "JPY", false),
        (i64::MAX - 1, "JPY", true),
    ] {
        apply(
            &client,
            "OfferPriceChangedEvent",
            &json!({"id":id,"price":{"amount":amount,"currency":currency}}),
        )
        .await;
        stored["price_amount"] = json!(amount);
        stored["currency"] = json!(currency);
        expected["price"] = json!({"amount":amount.to_string(),"currency":currency});
        expected["was_price"] = if visible {
            json!({"amount":i64::MAX.to_string(),"currency":"JPY"})
        } else {
            Value::Null
        };
        assert_eq!(offer(&pool), stored);
        assert_reads(&client, &expected).await;
    }
    kernel.shutdown();
}

#[tokio::test]
#[ignore = "requires DATABASE_URL_TEST and pg_trgm in customer_facing_extensions"]
async fn facts_create_change_clear_and_replay_with_browse_search_transport_parity() {
    let (_schema, pool) = database(true);
    let (kernel, state) = start_mulac(pool.clone()).unwrap();
    let client = TestClient::new(crate::app(state));
    let id = Uuid::now_v7();
    let mut payload = created(id);
    payload["title"] = json!("USB charger");
    payload["price"] = json!({"amount":1999,"currency":"AUD"});
    payload["was_price"] = json!({"amount":2999,"currency":"AUD"});
    payload["availability"] = json!("available");
    payload["item_condition"] = json!({"other":" Refurbished ž "});
    payload["content"] = json!({"description":"<p>Raw &amp; description</p>"});
    let creation = apply(&client, "OfferCreated", &payload).await;
    let item = browse(&client).await;
    assert_eq!(item["was_price"], json!({"amount":"2999","currency":"AUD"}));
    assert_eq!(item["availability"], json!({"kind":"available","raw":null}));
    assert_eq!(
        item["item_condition"],
        json!({"kind":"other","raw":" Refurbished ž "})
    );
    assert_eq!(
        item["content"],
        json!([{ "language": null, "title": payload["title"], "description": payload["content"]["description"], "url":payload["url"] }])
    );
    let response = client.get("/api/offers/search?q=charger").send().await;
    response.assert_status_is_ok();
    let search: Value = response.json().await.value().deserialize();
    assert_eq!(search["items"][0], item);
    for field in ["offers", "searchOffers(q:\"charger\")"] {
        let query = format!(
            "{{ {field} {{ items {{ price {{amount currency}} wasPrice {{amount currency}} availability {{kind raw}} itemCondition {{kind raw}} content {{language title description url}} }} }} }}"
        );
        let response = client
            .post("/graphql")
            .body_json(&json!({"query":query}))
            .send()
            .await;
        response.assert_status_is_ok();
        let result: Value = response.json().await.value().deserialize();
        let key = if field == "offers" {
            "offers"
        } else {
            "searchOffers"
        };
        assert_eq!(
            result["data"][key]["items"][0],
            json!({
                "price":item["price"],"wasPrice":item["was_price"],
                "availability":{"kind":"AVAILABLE","raw":null},
                "itemCondition":{"kind":"OTHER","raw":" Refurbished ž "},
                "content":item["content"]
            }),
            "{result}"
        );
    }
    let mut messages = vec![creation];
    for (kind, change) in [
        (
            "OfferPricingChanged",
            json!({"id":id,"pricing":{"price":{"amount":1499,"currency":"AUD"},"was_price":{"amount":3999,"currency":"AUD"}}}),
        ),
        (
            "OfferAvailabilityChanged",
            json!({"id":id,"availability":{"other":" https://schema.org/PreOrder "}}),
        ),
        (
            "OfferItemConditionChanged",
            json!({"id":id,"item_condition":"new"}),
        ),
        (
            "OfferContentChanged",
            json!({"id":id,"content":{"description":"<div>Updated ž</div>"}}),
        ),
    ] {
        messages.push(apply(&client, kind, &change).await);
    }
    let updated = offer(&pool);
    assert_eq!(updated["price_amount"], 1499);
    assert_eq!(updated["was_price_amount"], 3999);
    assert_eq!(
        updated["availability"],
        json!({"other":" https://schema.org/PreOrder "})
    );
    assert_eq!(updated["item_condition"], "new");
    assert_eq!(updated["description"], "<div>Updated ž</div>");
    for message in messages {
        replay_inbox(&pool, message);
        advance(&client, "inbox", 1).await;
        advance(&client, "commands", 1).await;
        assert_eq!(offer(&pool), updated);
    }
    assert_eq!(projection_counts(&pool), json!({"offers":1,"receipts":5}));

    // Legacy events change only current price; stale comparisons remain stored
    // but never appear in either read contract.
    for (amount, currency, visible) in [
        (1499, "USD", false),
        (3999, "AUD", false),
        (4000, "AUD", false),
        (1499, "AUD", true),
    ] {
        apply(
            &client,
            "OfferPriceChangedEvent",
            &json!({"id":id,"price":{"amount":amount,"currency":currency}}),
        )
        .await;
        assert_eq!(offer(&pool)["was_price_amount"], 3999);
        assert_eq!(browse(&client).await["was_price"].is_object(), visible);
    }
    for (kind, change) in [
        (
            "OfferPricingChanged",
            json!({"id":id,"pricing":{"price":{"amount":1499,"currency":"AUD"},"was_price":null}}),
        ),
        (
            "OfferAvailabilityChanged",
            json!({"id":id,"availability":null}),
        ),
        (
            "OfferItemConditionChanged",
            json!({"id":id,"item_condition":null}),
        ),
        (
            "OfferContentChanged",
            json!({"id":id,"content":{"description":null}}),
        ),
    ] {
        apply(&client, kind, &change).await;
    }
    let cleared = browse(&client).await;
    for field in ["was_price", "availability", "item_condition"] {
        assert!(cleared[field].is_null());
    }
    assert_eq!(
        cleared["content"],
        json!([{"language":null,"title":"USB charger","description":null}])
    );
    assert_eq!(cleared["title"], "USB charger");
    kernel.shutdown();
}

#[tokio::test]
#[ignore = "requires DATABASE_URL_TEST with CREATE SCHEMA permission"]
async fn fact_updates_wait_for_creation_and_invalid_commands_leave_no_receipt() {
    let (_schema, pool) = database(true);
    let (kernel, state) = start_mulac(pool.clone()).unwrap();
    let client = TestClient::new(crate::app(state));
    let id = Uuid::now_v7();
    for (kind, payload) in [
        (
            "OfferPricingChanged",
            json!({"id":id,"pricing":{"price":{"amount":10,"currency":"USD"},"was_price":{"amount":20,"currency":"USD"}}}),
        ),
        (
            "OfferAvailabilityChanged",
            json!({"id":id,"availability":"unavailable"}),
        ),
        (
            "OfferItemConditionChanged",
            json!({"id":id,"item_condition":"new"}),
        ),
        (
            "OfferContentChanged",
            json!({"id":id,"content":{"description":"source text"}}),
        ),
    ] {
        submit(&client, kind, &payload, Uuid::now_v7()).await;
    }
    advance(&client, "inbox", 4).await;
    assert_eq!(
        run(&client, "commands", 100).await["errors"]
            .as_array()
            .unwrap()
            .len(),
        4
    );
    assert_eq!(projection_counts(&pool), json!({"offers":0,"receipts":0}));
    apply(&client, "OfferCreated", &created(id)).await;
    pool.get()
        .unwrap()
        .batch_execute("UPDATE command_entries SET scheduled_at=now() WHERE status=4")
        .unwrap();
    advance(&client, "commands", 4).await;
    assert_eq!(projection_counts(&pool), json!({"offers":1,"receipts":5}));
    let before = offer(&pool);
    for (kind, payload) in [
        (
            "OfferPricingChanged",
            json!({"id":id,"pricing":{"price":{"amount":11,"currency":"USD"}}}),
        ),
        (
            "OfferAvailabilityChanged",
            json!({"id":id,"availability":{"other":" "}}),
        ),
        ("OfferItemConditionChanged", json!({"id":id})),
        ("OfferContentChanged", json!({"id":id,"content":{}})),
    ] {
        submit(&client, kind, &payload, Uuid::now_v7()).await;
    }
    advance(&client, "inbox", 4).await;
    assert_eq!(
        run(&client, "commands", 100).await["errors"]
            .as_array()
            .unwrap()
            .len(),
        4
    );
    assert_eq!(projection_counts(&pool), json!({"offers":1,"receipts":5}));
    assert_eq!(offer(&pool), before);
    kernel.shutdown();
}

#[tokio::test]
#[ignore = "requires DATABASE_URL_TEST with CREATE SCHEMA permission"]
async fn fact_migrations_preserve_old_rows_and_reject_incomplete_prices_and_status_shapes() {
    let (_schema, pool) = database(true);
    revert_to(&pool, "20261026000005");
    pool.get().unwrap().batch_execute("INSERT INTO offers(id,retailer_code,title,price_amount,currency,url,created_at) VALUES ('00000000-0000-0000-0000-000000000001','ankerau','Charger',0,'AUD','https://example.com/1',now()); INSERT INTO applied_offer_messages(message_id) VALUES ('00000000-0000-0000-0000-000000000002')").unwrap();
    let before = offer(&pool);
    pool.get().unwrap().batch_execute("INSERT INTO offer_images (offer_id,position,image_url) VALUES ('00000000-0000-0000-0000-000000000001',0,'https://example.com/image.png')").unwrap();
    let (kernel, state) = start_mulac(pool.clone()).unwrap();
    let client = TestClient::new(crate::app(state));
    let id = Uuid::from_u128(1);
    for (kind, payload) in [
        (
            "OfferTitleChanged",
            json!({"id":id,"title":"Pending charger"}),
        ),
        (
            "OfferPricingChanged",
            json!({"id":id,"pricing":{"price":{"amount":99,"currency":"JPY"},"was_price":{"amount":123,"currency":"JPY"}}}),
        ),
        (
            "OfferAvailabilityChanged",
            json!({"id":id,"availability":{"other":"http://schema.org/OutOfStock"}}),
        ),
        (
            "OfferItemConditionChanged",
            json!({"id":id,"item_condition":"new"}),
        ),
        (
            "OfferContentChanged",
            json!({"id":id,"content":{"description":" \t<p>Pending ž &amp; text</p>\n"}}),
        ),
    ] {
        submit(&client, kind, &payload, Uuid::now_v7()).await;
    }
    advance(&client, "inbox", 5).await;
    let auxiliary = |pool: &DbPool| {
        snapshot(
            pool,
            "SELECT jsonb_build_object(
        'images',(SELECT jsonb_agg(to_jsonb(i) ORDER BY position) FROM offer_images i),
        'commands',(SELECT jsonb_agg(to_jsonb(c) ORDER BY id) FROM command_entries c),
        'inbox',(SELECT jsonb_agg(to_jsonb(i) ORDER BY id) FROM inbox_entries i)) AS data",
        )
    };
    let pending = auxiliary(&pool);
    let receipts = snapshot(
        &pool,
        "SELECT jsonb_agg(to_jsonb(r)) AS data FROM applied_offer_messages r",
    );
    bootstrap::migrate(&pool).unwrap();
    let migrated = offer(&pool);
    for field in [
        "was_price_amount",
        "was_price_currency",
        "availability",
        "item_condition",
        "description",
    ] {
        assert!(migrated[field].is_null());
    }
    for sql in [
        "UPDATE offers SET was_price_amount=10",
        "UPDATE offers SET was_price_currency='AUD'",
        "UPDATE offers SET availability='null'",
        "UPDATE offers SET availability='{}'",
        "UPDATE offers SET availability='{\"other\":10}'",
        "UPDATE offers SET availability='{\"other\":\"raw\",\"extra\":null}'",
        "UPDATE offers SET item_condition='\"used\"'",
        "UPDATE offers SET item_condition='null'",
    ] {
        assert!(pool.get().unwrap().batch_execute(sql).is_err(), "{sql}");
    }
    pool.get().unwrap().batch_execute("UPDATE offers SET was_price_amount=20,was_price_currency='AUD',availability='\"in_stock\"',item_condition='{\"other\":\"Refurbished\"}',content=jsonb_set(content,'{entries,0,description}','\"<p>Text</p>\"')").unwrap();
    revert_to(&pool, "20261026000005");
    assert_eq!(offer(&pool), before);
    assert_eq!(
        snapshot(
            &pool,
            "SELECT jsonb_agg(to_jsonb(r)) AS data FROM applied_offer_messages r"
        ),
        receipts
    );
    bootstrap::migrate(&pool).unwrap();
    assert_eq!(offer(&pool), migrated);
    assert_eq!(auxiliary(&pool), pending);
    advance(&client, "commands", 5).await;
    let processed = offer(&pool);
    for (field, value) in [
        ("title", json!("Pending charger")),
        ("price_amount", json!(99)),
        ("currency", json!("JPY")),
        ("was_price_amount", json!(123)),
        ("was_price_currency", json!("JPY")),
        (
            "availability",
            json!({"other":"http://schema.org/OutOfStock"}),
        ),
        ("item_condition", json!("new")),
        ("description", json!(" \t<p>Pending ž &amp; text</p>\n")),
    ] {
        assert_eq!(processed[field], value, "{field}");
    }
    assert_eq!(projection_counts(&pool), json!({"offers":1,"receipts":6}));
    assert_eq!(auxiliary(&pool)["images"], pending["images"]);
    kernel.shutdown();
}
