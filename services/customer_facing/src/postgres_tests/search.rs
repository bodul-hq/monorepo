use super::*;
use crate::offer_read::Offers;
use diesel::sql_types::{Text, Uuid as SqlUuid};

fn insert(pool: &DbPool, id: u128, title: &str) {
    diesel::sql_query("INSERT INTO offers(id,retailer_code,price_amount,currency,created_at,content) VALUES ($1,'ankerau',1999,'AUD','2026-10-26T12:00:00Z',jsonb_build_object('selected_language',NULL,'entries',jsonb_build_array(jsonb_build_object('language',NULL,'title',$2::text,'description',NULL,'url',$3::text))))")
        .bind::<SqlUuid,_>(Uuid::from_u128(id)).bind::<Text,_>(title)
        .bind::<Text,_>(format!("https://example.com/{id}"))
        .execute(&mut pool.get().unwrap()).unwrap();
}

#[tokio::test]
#[ignore = "requires DATABASE_URL_TEST and pg_trgm in customer_facing_extensions"]
async fn relevance_pagination_mapping_updates_and_index_round_trip() {
    let (_schema, pool) = database(true);
    let offers = Offers::postgres(pool.clone());
    for (id, title) in [
        (1, "Charger"),
        (2, "USB charger 65W"),
        (3, "USB charge 65W"),
        (4, "Winter boots"),
        (5, "Čaj"),
        (6, "USB charger 65W"),
    ] {
        insert(&pool, id, title);
    }
    let found = offers.search("CHARGER", 100, 0).await.unwrap();
    let titles: Vec<_> = found.items.iter().map(|item| item.title.as_str()).collect();
    assert_eq!(
        titles,
        [
            "USB charger 65W",
            "USB charger 65W",
            "Charger",
            "USB charge 65W"
        ]
    );
    assert_eq!(found.items[0].id, Uuid::from_u128(6).to_string());
    for query in [
        "chargre",
        "charg",
        "USB charger",
        "c",
        "ch",
        "ČAJ",
        "100%_charger' OR 1=1 --",
    ] {
        // Punctuation is input text, never SQL or wildcard syntax.
        let page = offers.search(query, 100, 0).await.unwrap();
        assert!(
            page.items.iter().all(|offer| offer.title != "Winter boots"),
            "{query}"
        );
        if query == "chargre" || query == "ČAJ" {
            assert!(!page.items.is_empty(), "{query}");
        }
    }
    pool.get().unwrap().batch_execute("INSERT INTO offer_images(offer_id,position,image_url) VALUES ('00000000-0000-0000-0000-000000000001',1,'https://example.com/b'),('00000000-0000-0000-0000-000000000001',0,'https://example.com/a')").unwrap();
    let searched = offers.search("charger", 1, 2).await.unwrap();
    let browsed = offers
        .list(100, 0)
        .await
        .unwrap()
        .items
        .into_iter()
        .find(|offer| offer.id == searched.items[0].id)
        .unwrap();
    assert_eq!(
        serde_json::to_value(&browsed).unwrap(),
        serde_json::to_value(&searched.items[0]).unwrap()
    );
    assert_eq!(
        searched.items[0].images,
        ["https://example.com/a", "https://example.com/b"]
    );
    for id in 10..41 {
        insert(&pool, id, "Unique adapter");
    }
    assert!(
        offers
            .search("Unique adapter", 30, 0)
            .await
            .unwrap()
            .has_more
    );
    assert_eq!(
        offers
            .search("Unique adapter", 30, 30)
            .await
            .unwrap()
            .items
            .len(),
        1
    );
    assert!(
        offers
            .search("Unique adapter", 30, i32::MAX)
            .await
            .unwrap()
            .items
            .is_empty()
    );
    let (kernel, state) = start_mulac(pool.clone()).unwrap();
    let client = TestClient::new(crate::app(state));
    let message = json!({"message_id":Uuid::now_v7(),"source":"offers","routing_key":"OfferTitleChanged","payload":{"id":Uuid::from_u128(1),"title":"Telescope"}});
    for _ in 0..2 {
        client
            .post("/api/inbox")
            .body_json(&message)
            .send()
            .await
            .assert_status(StatusCode::ACCEPTED);
        run(&client, "inbox", 100).await;
        run(&client, "commands", 100).await;
    }
    assert_eq!(
        offers.search("telescope", 30, 0).await.unwrap().items.len(),
        1
    );
    assert!(
        offers
            .search("charger", 100, 0)
            .await
            .unwrap()
            .items
            .iter()
            .all(|item| item.id != Uuid::from_u128(1).to_string())
    );
    let before = snapshot(
        &pool,
        "SELECT jsonb_agg(to_jsonb(o) ORDER BY id) AS data FROM offers o",
    );
    pool.get()
        .unwrap()
        .batch_execute("DROP INDEX offers_content_title_search_idx")
        .unwrap();
    assert_eq!(
        snapshot(
            &pool,
            "SELECT to_jsonb(count(*)) AS data FROM pg_indexes WHERE schemaname=current_schema() AND indexname='offers_content_title_search_idx'"
        ),
        0
    );
    pool.get()
        .unwrap()
        .batch_execute("CREATE INDEX offers_content_title_search_idx ON offers USING gin (lower(offer_selected_content(content)->>'title') customer_facing_extensions.gin_trgm_ops)")
        .unwrap();
    assert_eq!(
        snapshot(
            &pool,
            "SELECT jsonb_agg(to_jsonb(o) ORDER BY id) AS data FROM offers o"
        ),
        before
    );
    assert_eq!(
        offers.search("telescope", 30, 0).await.unwrap().items.len(),
        1
    );
    kernel.shutdown();
}

#[tokio::test]
#[ignore = "requires DATABASE_URL_TEST and pg_trgm in customer_facing_extensions"]
async fn search_settings_are_local_on_success_and_failure() {
    let (_schema, pool) = database(true);
    // Hold every connection to initialize and later inspect the same pool.
    let mut connections = (0..3).map(|_| pool.get().unwrap()).collect::<Vec<_>>();
    for connection in &mut connections {
        connection
            .batch_execute("SET pg_trgm.word_similarity_threshold=0.9; SET statement_timeout='7s'")
            .unwrap();
    }
    drop(connections);
    let offers = Offers::postgres(pool.clone());
    offers.search("charger", 30, 0).await.unwrap();
    pool.get()
        .unwrap()
        .batch_execute("DROP TABLE offer_images")
        .unwrap();
    assert!(offers.search("charger", 30, 0).await.is_err());
    let mut connections = (0..3).map(|_| pool.get().unwrap()).collect::<Vec<_>>();
    for connection in &mut connections {
        let row = diesel::sql_query("SELECT jsonb_build_array(current_setting('pg_trgm.word_similarity_threshold'),current_setting('statement_timeout')) AS data")
            .get_result::<Snapshot>(connection).unwrap();
        assert_eq!(row.data, json!(["0.9", "7s"]));
    }
}
