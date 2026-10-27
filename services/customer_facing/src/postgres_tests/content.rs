use super::projection::{
    advance, apply, created, envelope, offer, projection_counts, replay_inbox, submit,
};
use super::*;
use commanding::io::ErasedCommandHandler;

#[tokio::test]
#[ignore = "requires DATABASE_URL_TEST and pg_trgm in customer_facing_extensions"]
async fn full_snapshots_mix_with_legacy_updates_and_replay_atomically() {
    let (_schema, pool) = database(true);
    let (kernel, state) = start_mulac(pool.clone()).unwrap();
    let client = TestClient::new(crate::app(state));
    let id = Uuid::now_v7();
    let list = json!([
        {"language":null,"title":"Unknown charger","description":null},
        {"language":"en","title":"English charger","description":"<p>English &amp;</p>"},
        {"language":"fr","title":"French charger","description":""}
    ]);
    let mut creation = created(id);
    creation["title"] = json!("Unknown charger");
    creation["content"] = list.clone();
    apply(&client, "OfferCreated", &creation).await;
    let message = apply(
        &client,
        "OfferContentsChanged",
        &json!({"id":id,"content":list,"language":"en"}),
    )
    .await;
    let original = offer(&pool);
    assert_eq!(original["content"], list);
    assert_eq!(original["title"], "English charger");
    assert_eq!(original["description"], "<p>English &amp;</p>");
    assert_eq!(original["language"], "en");
    // Selection-only snapshots still update all selected columns together.
    apply(
        &client,
        "OfferContentsChanged",
        &json!({"id":id,"content":list,"language":null}),
    )
    .await;
    assert_eq!(offer(&pool)["description"], Value::Null);
    apply(
        &client,
        "OfferContentsChanged",
        &json!({"id":id,"content":list,"language":"en"}),
    )
    .await;
    apply(
        &client,
        "OfferTitleChanged",
        &json!({"id":id,"title":"Updated charger"}),
    )
    .await;
    apply(
        &client,
        "OfferContentChanged",
        &json!({"id":id,"content":{"description":null}}),
    )
    .await;
    // A legacy language change moves selected content and replaces its collision.
    apply(&client, "OfferLocalizationChanged", &json!({"id":id,"localization":{"language":"fr","target_market":{"kind":"country","code":"FR"}}})).await;
    let expected = json!([
        {"language":null,"title":"Unknown charger","description":null},
        {"language":"fr","title":"Updated charger","description":null}
    ]);
    let before = offer(&pool);
    assert_eq!(before["content"], expected);
    assert_eq!(before["language"], "fr");
    assert_eq!(before["title"], "Updated charger");
    assert!(before["description"].is_null());
    replay_inbox(&pool, message);
    advance(&client, "inbox", 1).await;
    advance(&client, "commands", 1).await;
    assert_eq!(offer(&pool), before);
    assert_eq!(projection_counts(&pool), json!({"offers":1,"receipts":7}));
    for route in ["/api/offers", "/api/offers/search?q=charger"] {
        let response = client.get(route).send().await;
        response.assert_status_is_ok();
        let data: Value = response.json().await.value().deserialize();
        assert_eq!(data["items"][0]["content"], expected);
        assert_eq!(data["items"][0]["title"], "Updated charger");
    }
    client.post("/graphql").body_json(&json!({"query":"{ offers { items { title content { language title description } localization { language } } } }"})).send().await.assert_json(json!({"data":{"offers":{"items":[{"title":"Updated charger","content":expected,"localization":{"language":"fr"}}]}}})).await;
    // A failed scalar/list write cannot consume its original-message receipt.
    pool.get()
        .unwrap()
        .batch_execute(
            "ALTER TABLE offers ADD CONSTRAINT force_content_rollback CHECK (language = 'fr')",
        )
        .unwrap();
    let change = envelope(
        Uuid::now_v7(),
        "OfferContentsChanged",
        &json!({"id":id,"content":list,"language":"en"}),
    );
    let projection = crate::offer_projection::Projection(pool.clone());
    for malformed in [
        json!({"id":id,"content":list}),
        json!({"id":id,"content":[],"language":null}),
        json!({"id":id,"content":list,"language":"de"}),
        json!({"id":id,"content":[list[0],list[0]],"language":null}),
    ] {
        assert!(
            projection
                .execute(&envelope(
                    Uuid::now_v7(),
                    "OfferContentsChanged",
                    &malformed
                ))
                .is_err()
        );
    }
    assert!(projection.execute(&change).is_err());
    assert_eq!(offer(&pool), before);
    assert_eq!(projection_counts(&pool), json!({"offers":1,"receipts":7}));
    pool.get()
        .unwrap()
        .batch_execute("ALTER TABLE offers DROP CONSTRAINT force_content_rollback")
        .unwrap();
    projection.execute(&change).unwrap();
    assert_eq!(offer(&pool)["content"], list);
    assert_eq!(projection_counts(&pool), json!({"offers":1,"receipts":8}));
    kernel.shutdown();
}

#[tokio::test]
#[ignore = "requires DATABASE_URL_TEST and pg_trgm in customer_facing_extensions"]
async fn localized_migration_preserves_exact_scalars_receipts_and_pending_commands() {
    let (_schema, pool) = database(true);
    revert_to(&pool, "20261027000008");
    let mut connection = pool.get().unwrap();
    for (index, language, description) in [
        (1, None, None),
        (2, Some("EN-au"), Some("")),
        (3, Some("fr"), Some(" \t<p>ž &amp;</p>\n")),
    ] {
        diesel::sql_query("INSERT INTO offers(id,retailer_code,title,price_amount,currency,url,created_at,language,description) VALUES ($1,'ankerau',' Original charger ',10,'AUD',$2,now(),$3,$4)")
            .bind::<diesel::sql_types::Uuid,_>(Uuid::from_u128(index))
            .bind::<diesel::sql_types::Text,_>(format!("https://example.com/{index}"))
            .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>,_>(language)
            .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>,_>(description)
            .execute(&mut connection).unwrap();
    }
    connection.batch_execute("INSERT INTO applied_offer_messages(message_id) VALUES ('00000000-0000-0000-0000-000000000009'); INSERT INTO offer_images(offer_id,position,image_url) VALUES ('00000000-0000-0000-0000-000000000001',0,'https://example.com/image.png')").unwrap();
    drop(connection);
    let auxiliary = snapshot(
        &pool,
        "SELECT jsonb_build_object('receipts',(SELECT jsonb_agg(to_jsonb(r)) FROM applied_offer_messages r),'images',(SELECT jsonb_agg(to_jsonb(i)) FROM offer_images i)) AS data",
    );
    let rows = |pool: &DbPool| {
        snapshot(
            pool,
            "SELECT jsonb_agg(to_jsonb(o) ORDER BY id) AS data FROM offers o",
        )
    };
    let before = rows(&pool);
    let (kernel, state) = start_mulac(pool.clone()).unwrap();
    let client = TestClient::new(crate::app(state));
    submit(
        &client,
        "OfferContentChanged",
        &json!({"id":Uuid::from_u128(2),"content":{"description":"pending"}}),
        Uuid::now_v7(),
    )
    .await;
    advance(&client, "inbox", 1).await;
    let queued = snapshot(
        &pool,
        "SELECT jsonb_agg(to_jsonb(c)) AS data FROM command_entries c",
    );
    bootstrap::migrate(&pool).unwrap();
    let migrated = rows(&pool);
    for (old, new) in before
        .as_array()
        .unwrap()
        .iter()
        .zip(migrated.as_array().unwrap())
    {
        assert_eq!(
            new["content"],
            json!([{"language":old["language"],"title":old["title"],"description":old["description"]}])
        );
        let mut scalars = new.clone();
        scalars.as_object_mut().unwrap().remove("content");
        assert_eq!(&scalars, old);
    }
    for sql in [
        "UPDATE offers SET content=NULL",
        "UPDATE offers SET content='null'",
        "UPDATE offers SET content='{}'",
        "UPDATE offers SET content='[]'",
    ] {
        assert!(pool.get().unwrap().batch_execute(sql).is_err());
    }
    revert_to(&pool, "20261027000008");
    assert_eq!(rows(&pool), before);
    bootstrap::migrate(&pool).unwrap();
    assert_eq!(rows(&pool), migrated);
    assert_eq!(
        snapshot(
            &pool,
            "SELECT jsonb_agg(to_jsonb(c)) AS data FROM command_entries c"
        ),
        queued
    );
    assert_eq!(
        snapshot(
            &pool,
            "SELECT jsonb_build_object('receipts',(SELECT jsonb_agg(to_jsonb(r)) FROM applied_offer_messages r),'images',(SELECT jsonb_agg(to_jsonb(i)) FROM offer_images i)) AS data"
        ),
        auxiliary
    );
    advance(&client, "commands", 1).await;
    assert_eq!(rows(&pool)[1]["content"][0]["description"], "pending");
    assert_eq!(rows(&pool)[1]["description"], "pending");
    assert_eq!(projection_counts(&pool), json!({"offers":3,"receipts":2}));
    kernel.shutdown();
}

#[test]
#[ignore = "requires DATABASE_URL_TEST with CREATE SCHEMA permission"]
fn legacy_read_modify_write_waits_for_the_row_lock_and_preserves_the_latest_bucket() {
    let (_schema, pool) = database(true);
    let projection = crate::offer_projection::Projection(pool.clone());
    let id = Uuid::now_v7();
    projection
        .execute(&envelope(Uuid::now_v7(), "OfferCreated", &created(id)))
        .unwrap();
    let mut connection = pool.get().unwrap();
    connection
        .batch_execute("BEGIN; SELECT id FROM offers FOR UPDATE")
        .unwrap();
    let (sender, receiver) = std::sync::mpsc::channel();
    std::thread::scope(|scope| {
        scope.spawn(|| {
            let result = projection.execute(&envelope(
                Uuid::now_v7(),
                "OfferTitleChanged",
                &json!({"id":id,"title":"Latest charger"}),
            ));
            sender.send(result).unwrap();
        });
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
        loop {
            let waiting = snapshot(
                &pool,
                "SELECT to_jsonb(count(*)) AS data FROM pg_stat_activity WHERE datname=current_database() AND wait_event_type='Lock' AND query LIKE 'SELECT content, language FROM offers%'",
            );
            if waiting.as_i64().unwrap() > 0 {
                break;
            }
            if std::time::Instant::now() >= deadline {
                connection.batch_execute("ROLLBACK").unwrap();
                panic!("legacy update did not acquire its read lock");
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        connection.batch_execute("UPDATE offers SET description='concurrent description', content=jsonb_set(content,'{0,description}','\"concurrent description\"'); COMMIT").unwrap();
        receiver
            .recv_timeout(std::time::Duration::from_secs(5))
            .unwrap()
            .unwrap();
    });
    let row = offer(&pool);
    assert_eq!(row["title"], "Latest charger");
    assert_eq!(row["content"][0]["title"], "Latest charger");
    assert_eq!(row["description"], "concurrent description");
    assert_eq!(row["content"][0]["description"], "concurrent description");
}
