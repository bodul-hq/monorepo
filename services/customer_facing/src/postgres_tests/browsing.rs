use super::*;

async fn rest(client: &TestClient<poem::Route>, suffix: &str) -> Value {
    let response = client.get(format!("/api/offers{suffix}")).send().await;
    response.assert_status_is_ok();
    response.json().await.value().deserialize()
}

#[tokio::test]
#[ignore = "requires DATABASE_URL_TEST with CREATE SCHEMA permission"]
async fn browse_projection_order_values_updates_and_reversible_index() {
    let (_schema, pool) = database(true);
    let (kernel, state) = start_mulac(pool.clone()).unwrap();
    let client = TestClient::new(crate::app(state));
    for index in 1..=31 {
        let amount = match index {
            1 => i64::MIN,
            2 => i64::MAX,
            _ => 1999,
        };
        let payload = json!({"id":Uuid::from_u128(index),"retailer_code":"AnkerAu",
            "title":format!(" \tČaj 🔌 {index}\n"),"price":{"amount":amount,"currency":"AUD"},
            "url":format!("https://shop.example/Charger?variant={index}#details"),
            "created_at":if index==31 {"2026-10-23T12:00:00Z"}else{"2026-10-24T14:00:00.123456+02:00"}});
        client.post("/api/inbox").body_json(&json!({"message_id":Uuid::now_v7(),"source":"offers","routing_key":"OfferCreated","payload":payload})).send().await.assert_status(StatusCode::ACCEPTED);
    }
    assert_eq!(rest(&client, "").await["items"], json!([]));
    assert_eq!(
        run(&client, "inbox", 100).await,
        json!({"processed":31,"errors":[]})
    );
    assert_eq!(rest(&client, "").await["items"], json!([]));
    assert_eq!(
        run(&client, "commands", 100).await,
        json!({"processed":31,"errors":[]})
    );
    let first = rest(&client, "").await;
    assert_eq!(first["items"].as_array().unwrap().len(), 30);
    assert_eq!(first["has_more"], true);
    for (position, index) in (1..=30).rev().enumerate() {
        assert_eq!(
            first["items"][position]["id"],
            Uuid::from_u128(index).to_string()
        );
    }
    let last = &first["items"][29];
    assert_eq!(last["title"], " \tČaj 🔌 1\n");
    assert_eq!(last["price"]["amount"], i64::MIN.to_string());
    assert_eq!(first["items"][28]["price"]["amount"], i64::MAX.to_string());
    assert_eq!(last["created_at"], "2026-10-24T12:00:00.123456Z");
    assert_eq!(
        last["url"],
        "https://shop.example/Charger?variant=1#details"
    );
    assert_eq!(last["retailer_code"], "ankerau");
    let second = rest(&client, "?offset=30").await;
    assert_eq!(second["items"].as_array().unwrap().len(), 1);
    assert_eq!(second["items"][0]["id"], Uuid::from_u128(31).to_string());
    assert_eq!(second["has_more"], false);
    assert_eq!(rest(&client, "?offset=31").await["items"], json!([]));
    let response = client.post("/graphql").body_json(&json!({"query":"{ offers(limit: 2, offset: 28) { items { id title price { amount currency } createdAt } hasMore } }"})).send().await;
    let graphql: Value = response.json().await.value().deserialize();
    assert_eq!(
        graphql["data"]["offers"]["items"][0]["price"]["amount"],
        i64::MAX.to_string()
    );
    assert_eq!(
        graphql["data"]["offers"]["items"][1]["price"],
        last["price"]
    );
    assert_eq!(graphql["data"]["offers"]["hasMore"], true);
    for (kind, payload) in [
        (
            "OfferTitleChanged",
            json!({"id":Uuid::from_u128(30),"title":"Updated title"}),
        ),
        (
            "OfferPriceChangedEvent",
            json!({"id":Uuid::from_u128(30),"price":{"amount":123,"currency":"JPY"}}),
        ),
    ] {
        client.post("/api/inbox").body_json(&json!({"message_id":Uuid::now_v7(),"source":"offers","routing_key":kind,"payload":payload})).send().await.assert_status(StatusCode::ACCEPTED);
    }
    assert_eq!(
        run(&client, "inbox", 100).await,
        json!({"processed":2,"errors":[]})
    );
    assert_eq!(
        run(&client, "commands", 100).await,
        json!({"processed":2,"errors":[]})
    );
    let updated = rest(&client, "").await;
    assert_eq!(updated["items"][0]["title"], "Updated title");
    assert_eq!(
        updated["items"][0]["price"],
        json!({"amount":"123","currency":"JPY"})
    );
    assert_eq!(
        updated["items"][0]["created_at"],
        first["items"][0]["created_at"]
    );
    let index_query = "SELECT to_jsonb(count(*)) AS data FROM pg_indexes WHERE schemaname=current_schema() AND indexname='offers_created_at_id_idx'";
    assert_eq!(snapshot(&pool, index_query), 1);
    pool.get()
        .unwrap()
        .revert_last_migration(bootstrap::MIGRATIONS)
        .unwrap();
    assert_eq!(snapshot(&pool, index_query), 0);
    assert_eq!(rest(&client, "").await, updated);
    bootstrap::migrate(&pool).unwrap();
    assert_eq!(snapshot(&pool, index_query), 1);
    assert_eq!(rest(&client, "").await, updated);
    kernel.shutdown();
}
