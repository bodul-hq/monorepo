use super::*;
use diesel::sql_types::{Jsonb, Uuid as SqlUuid};

fn insert(pool: &DbPool, id: u128, content: Value) {
    diesel::sql_query("INSERT INTO offers(id,retailer_code,price_amount,currency,created_at,content) VALUES ($1,'ankerau',1999,'AUD','2026-10-29T00:00:00Z',$2)")
        .bind::<SqlUuid,_>(Uuid::from_u128(id)).bind::<Jsonb,_>(content)
        .execute(&mut pool.get().unwrap()).unwrap();
}

fn content(id: u128, title: &str, description: Option<&str>) -> Value {
    json!({"selected_language":"en","entries":[
        {"language":"en","title":title,"description":description,"url":format!("https://example.com/{id}")},
        {"language":"fr","title":"Caméra","description":"Appareil photo","url":format!("https://example.com/fr/{id}")}
    ]})
}

#[tokio::test]
#[ignore = "requires DATABASE_URL_TEST and pg_trgm in customer_facing_extensions"]
async fn selected_content_search_weights_and_language_switches() {
    let (_schema, pool) = database(true);
    insert(&pool, 1, content(1, "Charger", None));
    insert(&pool, 2, content(2, "Boot", Some("Charger")));
    insert(&pool, 3, content(3, "Charger", Some("Charger")));
    let offers = crate::offer_read::Offers::postgres(pool.clone());
    let results = offers.search("charger", 100, 0).await.unwrap();
    assert_eq!(
        results
            .items
            .iter()
            .map(|o| o.id.clone())
            .collect::<Vec<_>>(),
        vec![3, 1, 2]
            .into_iter()
            .map(|id| Uuid::from_u128(id).to_string())
            .collect::<Vec<_>>()
    );
    assert!(
        offers
            .search("caméra", 100, 0)
            .await
            .unwrap()
            .items
            .is_empty()
    );
    let first = offers.search("charger", 1, 0).await.unwrap();
    assert!(first.has_more);
    assert_eq!(first.items[0].id, Uuid::from_u128(3).to_string());
    pool.get().unwrap().batch_execute("UPDATE offers SET content=jsonb_set(content,'{selected_language}','\"fr\"') WHERE id='00000000-0000-0000-0000-000000000003'").unwrap();
    let selected = offers.search("caméra", 100, 0).await.unwrap();
    assert_eq!(selected.items.len(), 1);
    assert_eq!(selected.items[0].url, "https://example.com/fr/3");
    assert_eq!(selected.items[0].title, "Caméra");
    assert_eq!(
        offers.search("charger", 100, 0).await.unwrap().items.len(),
        2
    );
}

#[test]
#[ignore = "requires DATABASE_URL_TEST and pg_trgm in customer_facing_extensions"]
fn content_migration_preserves_multilingual_data_and_refuses_lossy_downgrade() {
    let (_schema, pool) = database(true);
    revert_to(&pool, "20261027000009");
    pool.get().unwrap().batch_execute("INSERT INTO offers(id,retailer_code,title,description,language,url,price_amount,currency,created_at,content) VALUES ('00000000-0000-0000-0000-000000000001','ankerau','Selected',NULL,'en','https://example.com/1',10,'AUD',now(),'[{\"language\":null,\"title\":\"Unknown\",\"description\":\"original\"},{\"language\":\"en\",\"title\":\"Selected\",\"description\":null}]')").unwrap();
    let before = snapshot(&pool, "SELECT to_jsonb(o) AS data FROM offers o");
    bootstrap::migrate(&pool).unwrap();
    let after = snapshot(&pool, "SELECT to_jsonb(o) AS data FROM offers o");
    for field in ["title", "description", "language", "url"] {
        assert!(after.get(field).is_none());
    }
    assert_eq!(after["content"]["selected_language"], "en");
    assert_eq!(
        after["content"]["entries"],
        with_urls(before["content"].clone(), &before["url"])
    );
    assert_eq!(logical_offer(after.clone()), before);
    revert_to(&pool, "20261027000009");
    assert_eq!(
        snapshot(&pool, "SELECT to_jsonb(o) AS data FROM offers o"),
        before
    );
    bootstrap::migrate(&pool).unwrap();
    assert_eq!(
        snapshot(&pool, "SELECT to_jsonb(o) AS data FROM offers o"),
        after
    );
    pool.get().unwrap().batch_execute("UPDATE offers SET content=jsonb_set(content,'{entries,0,url}','\"https://example.com/unknown\"')").unwrap();
    assert!(
        pool.get()
            .unwrap()
            .revert_last_migration(bootstrap::MIGRATIONS)
            .is_err()
    );
    assert_eq!(
        snapshot(
            &pool,
            "SELECT content->'entries'->0->'url' AS data FROM offers"
        ),
        "https://example.com/unknown"
    );
}

#[test]
#[ignore = "requires DATABASE_URL_TEST and pg_trgm in customer_facing_extensions"]
fn envelope_constraints_and_selected_url_uniqueness_reject_invalid_changes() {
    let (_schema, pool) = database(true);
    insert(&pool, 1, content(1, "Charger", None));
    insert(&pool, 2, content(2, "Charger", None));
    let before = snapshot(
        &pool,
        "SELECT jsonb_agg(to_jsonb(o) ORDER BY id) AS data FROM offers o",
    );
    for value in [
        json!(null),
        json!([]),
        json!({}),
        json!({"selected_language":"en","entries":[]}),
        json!({"selected_language":"missing","entries":content(1,"Charger",None)["entries"]}),
    ] {
        assert!(
            diesel::sql_query("UPDATE offers SET content=$1 WHERE id=$2")
                .bind::<Jsonb, _>(value)
                .bind::<SqlUuid, _>(Uuid::from_u128(1))
                .execute(&mut pool.get().unwrap())
                .is_err()
        );
    }
    assert!(pool.get().unwrap().batch_execute("UPDATE offers SET content=jsonb_set(content,'{entries,0,url}','\"https://example.com/1\"') WHERE id='00000000-0000-0000-0000-000000000002'").is_err());
    assert_eq!(
        snapshot(
            &pool,
            "SELECT jsonb_agg(to_jsonb(o) ORDER BY id) AS data FROM offers o"
        ),
        before
    );
}

#[test]
#[ignore = "requires DATABASE_URL_TEST and pg_trgm in customer_facing_extensions"]
fn snapshots_preserve_urls_and_legacy_language_moves_carry_selected_url() {
    use commanding::io::ErasedCommandHandler;
    let (_schema, pool) = database(true);
    insert(&pool, 1, content(1, "Charger", None));
    let projection = crate::offer_projection::Projection(pool.clone());
    let id = Uuid::from_u128(1);
    projection
        .execute(&super::projection::envelope(
            Uuid::now_v7(),
            "OfferContentsChanged",
            &json!({
                "id":id,"language":"en","content":[
                    {"language":"en","title":"Updated charger","description":null},
                    {"language":"fr","title":"Caméra","description":null},
                    {"language":"de","title":"Ladegerät","description":null}
                ]
            }),
        ))
        .unwrap();
    let row = snapshot(&pool, "SELECT content AS data FROM offers");
    let entries = row["entries"].as_array().unwrap();
    assert_eq!(
        entries.iter().find(|e| e["language"] == "fr").unwrap()["url"],
        "https://example.com/fr/1"
    );
    assert_eq!(
        entries.iter().find(|e| e["language"] == "de").unwrap()["url"],
        "https://example.com/1"
    );
    projection
        .execute(&super::projection::envelope(
            Uuid::now_v7(),
            "OfferLocalizationChanged",
            &json!({"id":id,"localization":{"language":"fr","target_market":null}}),
        ))
        .unwrap();
    let row = snapshot(&pool, "SELECT content AS data FROM offers");
    assert_eq!(row["selected_language"], "fr");
    let selected = row["entries"]
        .as_array()
        .unwrap()
        .iter()
        .find(|e| e["language"] == "fr")
        .unwrap();
    assert_eq!(selected["url"], "https://example.com/1");
    assert_eq!(selected["title"], "Updated charger");
    assert!(
        !row["entries"]
            .as_array()
            .unwrap()
            .iter()
            .any(|e| e["language"] == "en")
    );
}
