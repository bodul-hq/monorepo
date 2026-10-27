use super::*;
use commanding::io::{ErasedCommandHandler, NewCommand, NewCommandEnvelope, NewCommandMetadata};

fn created(id: Uuid) -> Value {
    json!({
        "id":id, "retailer_code":"AnkerAu", "title":"  Punjač 🔌\n USB  ",
        "price":{"amount":9_007_199_254_740_993_i64,"currency":"AUD"},
        "url":"https://shop.example/Charger?variant=123#details",
        "created_at":"2026-10-24T14:00:00.123456+02:00"
    })
}

async fn submit(client: &TestClient<poem::Route>, kind: &str, payload: &Value, message: Uuid) {
    client
        .post("/api/inbox")
        .body_json(&json!({
            "routing_key":kind,"message_id":message,"source":"offers",
            "correlation_id":null,"payload":payload
        }))
        .send()
        .await
        .assert_status(StatusCode::ACCEPTED);
}

async fn advance(client: &TestClient<poem::Route>, queue: &str, count: usize) {
    assert_eq!(
        run(client, queue, 100).await,
        json!({"processed":count,"errors":[]})
    );
}

async fn apply(client: &TestClient<poem::Route>, kind: &str, payload: &Value) -> Uuid {
    let message = Uuid::now_v7();
    submit(client, kind, payload, message).await;
    advance(client, "inbox", 1).await;
    advance(client, "commands", 1).await;
    advance(client, "events", 0).await;
    message
}

fn offer(pool: &DbPool) -> Value {
    snapshot(pool, "SELECT to_jsonb(o) AS data FROM offers o")
}

fn projection_counts(pool: &DbPool) -> Value {
    snapshot(
        pool,
        "SELECT jsonb_build_object(
        'offers', (SELECT count(*) FROM offers),
        'receipts', (SELECT count(*) FROM applied_offer_messages)) AS data",
    )
}

// Simulate the inbox-completion crash window. Mulac generates a fresh command
// UUID when the still-pending original inbox message is processed again.
fn replay_inbox(pool: &DbPool, message: Uuid) {
    diesel::sql_query(
        "UPDATE inbox_entries SET status = 0, attempts = 0,
        scheduled_at = now(), processed_at = NULL WHERE id = $1",
    )
    .bind::<diesel::sql_types::Uuid, _>(message)
    .execute(&mut pool.get().unwrap())
    .unwrap();
}

#[tokio::test]
#[ignore = "requires DATABASE_URL_TEST with CREATE SCHEMA permission"]
async fn receipt_progression_fidelity_updates_and_replay_survive_restart() {
    let (_schema, pool) = database(true);
    let (kernel, state) = start_mulac(pool.clone()).unwrap();
    let client = TestClient::new(crate::app(state));
    let id = Uuid::now_v7();
    let message = Uuid::now_v7();
    let payload = created(id);
    submit(&client, "OfferCreated", &payload, message).await;
    let mut duplicate = payload.clone();
    duplicate["title"] = json!("duplicate HTTP content is ignored");
    submit(&client, "OfferCreated", &duplicate, message).await;
    assert_eq!(counts(&pool), json!({"inbox":1,"commands":0,"events":0}));
    assert_eq!(projection_counts(&pool), json!({"offers":0,"receipts":0}));
    advance(&client, "inbox", 1).await;
    assert_eq!(counts(&pool), json!({"inbox":1,"commands":1,"events":0}));
    assert_eq!(projection_counts(&pool), json!({"offers":0,"receipts":0}));
    advance(&client, "events", 0).await;
    advance(&client, "commands", 1).await;
    let mut expected = json!({
        "id":id, "retailer_code":"ankerau", "title":payload["title"],
        "price_amount":payload["price"]["amount"], "currency":"AUD",
        "url":payload["url"], "created_at":"2026-10-24T12:00:00.123456+00:00"
    });
    assert_eq!(offer(&pool), expected);
    let title_message = apply(
        &client,
        "OfferTitleChanged",
        &json!({"id":id,"title":" New čaj ☕ \t"}),
    )
    .await;
    expected["title"] = json!(" New čaj ☕ \t");
    assert_eq!(offer(&pool), expected);
    let price_message = apply(
        &client,
        "OfferPriceChangedEvent",
        &json!({"id":id,"price":{"amount":0,"currency":"EUR"}}),
    )
    .await;
    expected["price_amount"] = json!(0);
    expected["currency"] = json!("EUR");
    assert_eq!(offer(&pool), expected);
    apply(
        &client,
        "OfferTitleChanged",
        &json!({"id":id,"title":"final title"}),
    )
    .await;
    apply(
        &client,
        "OfferPriceChangedEvent",
        &json!({"id":id,"price":{"amount":i64::MIN,"currency":"USD"}}),
    )
    .await;
    expected["title"] = json!("final title");
    expected["price_amount"] = json!(i64::MIN);
    expected["currency"] = json!("USD");
    assert_eq!(offer(&pool), expected);
    drop(client);
    kernel.shutdown();
    drop(kernel);

    bootstrap::migrate(&pool).unwrap();
    let (kernel, state) = start_mulac(pool.clone()).unwrap();
    let client = TestClient::new(crate::app(state));
    assert_eq!(offer(&pool), expected);
    for message in [message, title_message, price_message] {
        replay_inbox(&pool, message);
        advance(&client, "inbox", 1).await;
        advance(&client, "commands", 1).await;
        assert_eq!(offer(&pool), expected);
    }
    assert_eq!(projection_counts(&pool), json!({"offers":1,"receipts":5}));
    assert_eq!(
        snapshot(
            &pool,
            "SELECT to_jsonb(count(DISTINCT id)) AS data FROM command_entries"
        ),
        8
    );
    // Distinct identities carry no revision: a delayed source value applies now.
    apply(
        &client,
        "OfferTitleChanged",
        &json!({"id":id,"title":"older source value"}),
    )
    .await;
    expected["title"] = json!("older source value");
    assert_eq!(offer(&pool), expected);
    kernel.shutdown();
}

#[tokio::test]
#[ignore = "requires DATABASE_URL_TEST with CREATE SCHEMA permission"]
async fn missing_offer_updates_retry_after_creation_and_exhaust_visibly() {
    let (_schema, pool) = database(true);
    let (kernel, state) = start_mulac(pool.clone()).unwrap();
    let client = TestClient::new(crate::app(state));
    let id = Uuid::now_v7();
    for (kind, payload) in [
        (
            "OfferTitleChanged",
            json!({"id":id,"title":"retried title"}),
        ),
        (
            "OfferPriceChangedEvent",
            json!({"id":id,"price":{"amount":-1,"currency":"CAD"}}),
        ),
    ] {
        submit(&client, kind, &payload, Uuid::now_v7()).await;
    }
    advance(&client, "inbox", 2).await;
    let failed = run(&client, "commands", 100).await;
    assert!(failed["processed"].is_null());
    assert_eq!(failed["errors"].as_array().unwrap().len(), 2);
    assert!(
        failed["errors"][0]
            .as_str()
            .unwrap()
            .contains("offer has not been created")
    );
    assert_eq!(projection_counts(&pool), json!({"offers":0,"receipts":0}));
    assert_eq!(
        snapshot(
            &pool,
            "SELECT to_jsonb(count(*)) AS data FROM command_entries
        WHERE status = 4 AND attempts = 1 AND scheduled_at > now()
        AND reservation_id IS NULL AND extra_info IS NOT NULL"
        ),
        2
    );
    advance(&client, "commands", 0).await;
    apply(&client, "OfferCreated", &created(id)).await;
    pool.get()
        .unwrap()
        .batch_execute("UPDATE command_entries SET scheduled_at = now() WHERE status = 4")
        .unwrap();
    advance(&client, "commands", 2).await;
    assert_eq!(offer(&pool)["title"], "retried title");
    assert_eq!(offer(&pool)["price_amount"], -1);
    assert_eq!(offer(&pool)["currency"], "CAD");
    assert_eq!(projection_counts(&pool), json!({"offers":1,"receipts":3}));

    submit(
        &client,
        "OfferTitleChanged",
        &json!({"id":Uuid::now_v7(),"title":"never created"}),
        Uuid::now_v7(),
    )
    .await;
    advance(&client, "inbox", 1).await;
    for attempt in 1..=6 {
        pool.get()
            .unwrap()
            .batch_execute("UPDATE command_entries SET scheduled_at = now() WHERE status IN (0,4)")
            .unwrap();
        assert!(run(&client, "commands", 1).await["processed"].is_null());
        assert_eq!(
            snapshot(
                &pool,
                "SELECT to_jsonb(attempts) AS data FROM command_entries WHERE status IN (4,7)"
            ),
            attempt
        );
    }
    assert_eq!(
        snapshot(
            &pool,
            "SELECT to_jsonb(status) AS data FROM command_entries WHERE attempts = 6"
        ),
        7
    );
    advance(&client, "commands", 0).await;
    kernel.shutdown();
}

#[tokio::test]
#[ignore = "requires DATABASE_URL_TEST with CREATE SCHEMA permission"]
async fn failed_business_write_rolls_back_receipt_and_can_retry() {
    let (_schema, pool) = database(true);
    let (kernel, state) = start_mulac(pool.clone()).unwrap();
    let client = TestClient::new(crate::app(state));
    let id = Uuid::now_v7();
    apply(&client, "OfferCreated", &created(id)).await;
    let before = offer(&pool);
    pool.get()
        .unwrap()
        .batch_execute(
            "ALTER TABLE offers ADD CONSTRAINT simulated_failure CHECK (currency = 'AUD')",
        )
        .unwrap();
    submit(
        &client,
        "OfferPriceChangedEvent",
        &json!({"id":id,"price":{"amount":42,"currency":"EUR"}}),
        Uuid::now_v7(),
    )
    .await;
    advance(&client, "inbox", 1).await;
    assert!(run(&client, "commands", 1).await["processed"].is_null());
    assert_eq!(offer(&pool), before);
    assert_eq!(projection_counts(&pool), json!({"offers":1,"receipts":1}));
    pool.get()
        .unwrap()
        .batch_execute(
            "ALTER TABLE offers DROP CONSTRAINT simulated_failure;
        UPDATE command_entries SET scheduled_at = now() WHERE status = 4",
        )
        .unwrap();
    advance(&client, "commands", 1).await;
    assert_eq!(offer(&pool)["price_amount"], 42);
    assert_eq!(offer(&pool)["currency"], "EUR");
    assert_eq!(projection_counts(&pool), json!({"offers":1,"receipts":2}));
    kernel.shutdown();
}

#[tokio::test]
#[ignore = "requires DATABASE_URL_TEST with CREATE SCHEMA permission"]
async fn conflicting_creates_preserve_identity_and_url_comparison_is_exact() {
    let (_schema, pool) = database(true);
    let (kernel, state) = start_mulac(pool.clone()).unwrap();
    let client = TestClient::new(crate::app(state));
    let original = created(Uuid::now_v7());
    apply(&client, "OfferCreated", &original).await;
    let before = offer(&pool);
    let mut same_id = original.clone();
    same_id["url"] = json!("https://other.example");
    let same_url = created(Uuid::now_v7());
    for payload in [&original, &same_id, &same_url] {
        submit(&client, "OfferCreated", payload, Uuid::now_v7()).await;
        advance(&client, "inbox", 1).await;
        assert!(run(&client, "commands", 1).await["processed"].is_null());
        assert_eq!(offer(&pool), before);
        assert_eq!(projection_counts(&pool), json!({"offers":1,"receipts":1}));
    }
    let mut case_variant = same_url;
    case_variant["url"] = json!(original["url"].as_str().unwrap().to_lowercase());
    apply(&client, "OfferCreated", &case_variant).await;
    assert_eq!(projection_counts(&pool), json!({"offers":2,"receipts":2}));
    kernel.shutdown();
}

#[tokio::test]
#[ignore = "requires DATABASE_URL_TEST with CREATE SCHEMA permission"]
async fn malformed_and_unsupported_messages_fail_without_partial_changes() {
    let (_schema, pool) = database(true);
    let (kernel, state) = start_mulac(pool.clone()).unwrap();
    let client = TestClient::new(crate::app(state));
    let id = Uuid::now_v7();
    apply(&client, "OfferCreated", &created(id)).await;
    let before = offer(&pool);
    let mut bad_creation = created(Uuid::now_v7());
    bad_creation["retailer_code"] = json!("ankerau");
    let cases = vec![
        ("OfferCreated", bad_creation),
        ("OfferCreated", json!({"id":id})),
        ("OfferPriceChangedEvent", json!({"id":id,"price":[1,"EUR"]})),
        ("OfferTitleChanged", json!({"id":"invalid","title":"bad"})),
        ("OfferTitleChanged", json!({"id":id,"title":"nul\0"})),
        ("OfferTitleChanged", json!({"id":id,"title":null})),
        ("OfferTitleChanged", json!([id, "array"])),
        ("OfferTitleChanged", json!("encoded payload")),
        (
            "OfferPriceChangedEvent",
            json!({"id":id,"price":{"amount":1.5,"currency":"EUR"}}),
        ),
        (
            "OfferPriceChangedEvent",
            json!({"id":id,"price":{"amount":1,"currency":"ZZZ"}}),
        ),
        (
            "OfferPriceChangedEvent",
            json!({"id":id,"price":{"amount":u64::MAX,"currency":"EUR"}}),
        ),
        (
            "OfferPriceChanged",
            json!({"id":id,"price":{"amount":1,"currency":"EUR"}}),
        ),
    ];
    for (kind, payload) in cases {
        submit(&client, kind, &payload, Uuid::now_v7()).await;
        advance(&client, "inbox", 1).await;
        let failed = run(&client, "commands", 1).await;
        assert!(failed["processed"].is_null(), "{kind}: {failed}");
        assert!(!failed["errors"].as_array().unwrap().is_empty());
        assert_eq!(offer(&pool), before);
        assert_eq!(projection_counts(&pool), json!({"offers":1,"receipts":1}));
    }
    kernel.shutdown();
}

fn envelope(message: Uuid, kind: &str, payload: &Value) -> NewCommandEnvelope {
    NewCommandEnvelope {
        command: NewCommand {
            command_type: kind.into(),
            payload: payload.to_string(),
        },
        metadata: Some(NewCommandMetadata {
            command_id: Uuid::now_v7(),
            correlation_id: None,
            causation_id: Some(message),
            source: Some("offers".into()),
        }),
    }
}

#[test]
#[ignore = "requires DATABASE_URL_TEST with CREATE SCHEMA permission"]
fn concurrent_original_message_has_one_effect_even_with_distinct_commands() {
    let (_schema, pool) = database(true);
    let projection = crate::offer_projection::Projection(pool.clone());
    let message = Uuid::now_v7();
    let payload = created(Uuid::now_v7());
    let barrier = std::sync::Barrier::new(3);
    std::thread::scope(|scope| {
        for _ in 0..3 {
            let command = envelope(message, "OfferCreated", &payload);
            let projection = &projection;
            let barrier = &barrier;
            scope.spawn(move || {
                barrier.wait();
                assert!(projection.execute(&command).unwrap().is_empty());
            });
        }
    });
    assert_eq!(projection_counts(&pool), json!({"offers":1,"receipts":1}));
}

#[tokio::test]
#[ignore = "requires DATABASE_URL_TEST with CREATE SCHEMA permission"]
async fn additive_migration_keeps_existing_queued_work() {
    let (_schema, pool) = database(false);
    pool.get()
        .unwrap()
        .batch_execute(include_str!(
            "../../migrations/2026-10-24-000001_mulac/up.sql"
        ))
        .unwrap();
    // Mark the existing bootstrap migration exactly as Diesel does.
    pool.get()
        .unwrap()
        .batch_execute(
            "CREATE TABLE __diesel_schema_migrations (
        version VARCHAR(50) PRIMARY KEY, run_on TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP);
        INSERT INTO __diesel_schema_migrations (version) VALUES ('20261024000001')",
        )
        .unwrap();
    let (kernel, state) = start_mulac(pool.clone()).unwrap();
    let client = TestClient::new(crate::app(state));
    submit(
        &client,
        "OfferCreated",
        &created(Uuid::now_v7()),
        Uuid::now_v7(),
    )
    .await;
    let before = snapshot(&pool, "SELECT to_jsonb(i) AS data FROM inbox_entries i");
    bootstrap::migrate(&pool).unwrap();
    assert_eq!(
        snapshot(&pool, "SELECT to_jsonb(i) AS data FROM inbox_entries i"),
        before
    );
    advance(&client, "inbox", 1).await;
    advance(&client, "commands", 1).await;
    assert_eq!(projection_counts(&pool), json!({"offers":1,"receipts":1}));
    kernel.shutdown();
}
