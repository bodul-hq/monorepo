//! Delivery from offers' outbox to a separate Mulac inbox, the same inbox
//! customer-facing serves, each in its own isolated schema.

use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

use diesel::{RunQueryDsl, connection::SimpleConnection};
use inbox_api::io::InboxService;
use mulac_poem::io::{InboxAdapter, OutboxAdapter};
use outbox_api::io::OutboxService;
use poem::{Endpoint, EndpointExt, IntoResponse, Route, http::StatusCode, test::TestClient};
use poem_openapi::OpenApiService;
use serde_json::{Value, json};
use testimony_infra::prelude::DbPool;
use tokio::runtime::Handle;
use uuid::Uuid;

use super::CustomerFacingPublisher;
use super::testing::{Receiver, envelope, offer_created, serve};
use crate::offers::event::{
    OFFER_CREATED_EVENT, OFFER_PRICE_CHANGED_EVENT, OFFER_TITLE_CHANGED_EVENT,
};
use crate::orchestration::InboxApi;
use crate::postgres_tests::{TestSchema, inbox_test_database, inbox_test_snapshot, run_queue};

fn rows(pool: &DbPool, table: &str) -> Value {
    inbox_test_snapshot(
        pool,
        &format!(
            "SELECT coalesce(jsonb_agg(to_jsonb(t) ORDER BY id), '[]') AS data FROM {table} t"
        ),
    )
}

/// A Mulac inbox over its own schema, as customer-facing serves it.
fn inbox_endpoint(pool: &DbPool) -> impl Endpoint + 'static {
    let adapter = InboxAdapter::new(InboxService::from_postgres(pool.clone()));
    let api = OpenApiService::new(InboxApi::new(adapter), "Receiver", "1").server("/api");
    Route::new().nest("/api", api)
}

struct Sender {
    _schema: TestSchema,
    pool: DbPool,
    client: TestClient<Route>,
    kernel: Arc<kernel::PersistentKernelHandle>,
}

impl Drop for Sender {
    fn drop(&mut self) {
        self.kernel.shutdown();
    }
}

/// Offers with delivery configured to `url`.
fn sender(url: &str) -> Sender {
    let (schema, pool) = inbox_test_database();
    let (kernel, mut state) = crate::start_mulac(pool.clone()).unwrap();
    let publisher = CustomerFacingPublisher::new(url, Handle::current()).unwrap();
    state.outbox = OutboxAdapter::new(OutboxService::from_postgres(
        pool.clone(),
        Arc::new(publisher),
    ));
    let client = TestClient::new(crate::app_with_reporting_token(state, "test".into()));
    Sender {
        _schema: schema,
        pool,
        client,
        kernel,
    }
}

async fn run_outbox(sender: &Sender, limit: usize) -> Value {
    let response = sender
        .client
        .post("/api/outbox/run")
        .body_json(&json!({"limit": limit}))
        .send()
        .await;
    response.assert_status_is_ok();
    response.json().await.value().deserialize()
}

/// Records the row exactly as `CustomerFacingSubscriber` does for an event.
fn record(pool: &DbPool, event_type: &str, payload: &Value) -> Uuid {
    use kernel::EventSubscriberPort;
    let subscriber =
        crate::subscribers::customer_facing::CustomerFacingSubscriber::new(pool.clone());
    let event_id = Uuid::new_v4();
    let envelope = kernel::NewEventEnvelope {
        event_type: event_type.into(),
        payload: payload.to_string(),
        metadata: Some(kernel::NewEventMetadata {
            event_id,
            correlation_id: Some(Uuid::new_v4()),
            causation_id: Some(Uuid::new_v4()),
            source: Some("offers-test".into()),
        }),
    };
    subscriber.handle(&envelope).unwrap();
    event_id
}

fn make_due(pool: &DbPool) {
    pool.get()
        .unwrap()
        .batch_execute("UPDATE outbox_entries SET scheduled_at = now() WHERE status = 4")
        .unwrap();
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires DATABASE_URL_TEST with CREATE SCHEMA permission"]
async fn postgres_offer_events_reach_the_inbox_with_their_identity_and_payloads() {
    let (_receiver_schema, receiver_pool) = inbox_test_database();
    let receiver = serve(inbox_endpoint(&receiver_pool));
    let sender = sender(&receiver.url);

    // A real import records OfferCreated; updates are recorded the same way.
    let response = sender
        .client
        .post("/api/tmp-import")
        .body_json(&json!({
            "retailer_code": "ankerau", "title": "Charger", "price_amount": 1999,
            "currency": "AUD", "url": "https://example.com/Charger?q=red#details"
        }))
        .send()
        .await;
    response.assert_status(StatusCode::ACCEPTED);
    run_queue(&sender.client, "inbox", 10, 1).await;
    run_queue(&sender.client, "commands", 10, 1).await;
    run_queue(&sender.client, "events", 10, 1).await;
    let created = rows(&sender.pool, "outbox_entries");
    let offer_id = serde_json::from_str::<Value>(created[0]["payload"].as_str().unwrap()).unwrap()
        ["id"]
        .clone();
    record(
        &sender.pool,
        OFFER_TITLE_CHANGED_EVENT,
        &json!({"id": offer_id, "title": "New title"}),
    );
    record(
        &sender.pool,
        OFFER_PRICE_CHANGED_EVENT,
        &json!({"id": offer_id, "price": {"amount": 1299, "currency": "EUR"}}),
    );
    let before = rows(&sender.pool, "outbox_entries");
    assert_eq!(before.as_array().unwrap().len(), 3);

    assert_eq!(
        run_outbox(&sender, 10).await,
        json!({"processed": 3, "errors": []})
    );

    let after = rows(&sender.pool, "outbox_entries");
    let accepted = rows(&receiver_pool, "inbox_entries");
    assert_eq!(accepted.as_array().unwrap().len(), 3);
    let mut event_types = Vec::new();
    for (sent, delivered) in before
        .as_array()
        .unwrap()
        .iter()
        .zip(after.as_array().unwrap())
    {
        assert_eq!(delivered["status"], 5);
        assert_eq!(delivered["id"], sent["id"]);
        assert_eq!(delivered["payload"], sent["payload"]);
        assert_eq!(delivered["meta"], sent["meta"]);
        let inbox = accepted
            .as_array()
            .unwrap()
            .iter()
            .find(|row| row["id"] == sent["meta"]["message_id"])
            .unwrap();
        assert_eq!(inbox["meta"]["routing_key"], sent["meta"]["event_type"]);
        assert_eq!(
            inbox["meta"]["correlation_id"],
            sent["meta"]["correlation_id"]
        );
        // An absent source, as on events from imports, defaults to offers.
        let source = match &sent["meta"]["source"] {
            Value::Null => json!("offers"),
            source => source.clone(),
        };
        assert_eq!(inbox["meta"]["source"], source);
        assert_eq!(
            serde_json::from_str::<Value>(inbox["payload"].as_str().unwrap()).unwrap(),
            serde_json::from_str::<Value>(sent["payload"].as_str().unwrap()).unwrap()
        );
        event_types.push(sent["meta"]["event_type"].as_str().unwrap().to_owned());
    }
    event_types.sort();
    assert_eq!(
        event_types,
        [
            OFFER_CREATED_EVENT,
            OFFER_PRICE_CHANGED_EVENT,
            OFFER_TITLE_CHANGED_EVENT
        ]
    );
    // Acceptance is storage only; customer-facing has queued nothing.
    assert_eq!(rows(&receiver_pool, "command_entries"), json!([]));
    assert_eq!(
        run_outbox(&sender, 10).await,
        json!({"processed": 0, "errors": []})
    );
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires DATABASE_URL_TEST with CREATE SCHEMA permission"]
async fn postgres_partial_batch_keeps_deliveries_and_fails_invalid_rows_visibly() {
    let (_receiver_schema, receiver_pool) = inbox_test_database();
    let receiver = serve(inbox_endpoint(&receiver_pool));
    let sender = sender(&receiver.url);
    let good = record(&sender.pool, OFFER_CREATED_EVENT, &offer_created());
    let unknown = record(&sender.pool, OFFER_CREATED_EVENT, &offer_created());
    // A manually seeded row with an unsupported type and a malformed one.
    sender
        .pool
        .get()
        .unwrap()
        .batch_execute(&format!(
            "UPDATE outbox_entries SET meta = jsonb_set(meta, '{{event_type}}', '\"OfferDeleted\"') WHERE id = '{unknown}';"
        ))
        .unwrap();
    let malformed = envelope(OFFER_CREATED_EVENT, &json!({"id": "x"}));
    diesel::sql_query("INSERT INTO outbox_entries (id, payload, meta) VALUES ($1, $2, $3)")
        .bind::<diesel::sql_types::Uuid, _>(malformed.metadata.message_id)
        .bind::<diesel::sql_types::Text, _>(String::from_utf8(malformed.payload.clone()).unwrap())
        .bind::<diesel::sql_types::Jsonb, _>(serde_json::to_value(&malformed.metadata).unwrap())
        .execute(&mut sender.pool.get().unwrap())
        .unwrap();

    let report = run_outbox(&sender, 10).await;

    assert!(report["processed"].is_null(), "{report}");
    let errors = report["errors"].to_string();
    assert_eq!(report["errors"].as_array().unwrap().len(), 2, "{errors}");
    assert!(errors.contains("event type must be"), "{errors}");
    assert!(
        errors.contains("valid customer-facing inbox submission"),
        "{errors}"
    );
    let after = rows(&sender.pool, "outbox_entries");
    for row in after.as_array().unwrap() {
        if row["id"] == json!(good) {
            assert_eq!(row["status"], 5);
        } else {
            assert_eq!(row["status"], 4, "{row}");
            assert_eq!(row["attempts"], 1);
            assert!(row["reservation_id"].is_null());
        }
    }
    let accepted = rows(&receiver_pool, "inbox_entries");
    assert_eq!(accepted.as_array().unwrap().len(), 1);
    assert_eq!(accepted[0]["id"], json!(good));
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires DATABASE_URL_TEST with CREATE SCHEMA permission"]
async fn postgres_failed_completion_needs_recovery_and_redelivers_one_inbox_row() {
    let (_receiver_schema, receiver_pool) = inbox_test_database();
    let receiver = serve(inbox_endpoint(&receiver_pool));
    let sender = sender(&receiver.url);
    record(&sender.pool, OFFER_CREATED_EVENT, &offer_created());
    sender
        .pool
        .get()
        .unwrap()
        .batch_execute(
            "ALTER TABLE outbox_entries ADD CONSTRAINT prevent_completion CHECK (status <> 5)",
        )
        .unwrap();

    assert!(run_outbox(&sender, 1).await["processed"].is_null());
    // Accepted, but the row is left reserved: a stale reservation.
    assert_eq!(rows(&sender.pool, "outbox_entries")[0]["status"], 2);
    assert_eq!(
        rows(&receiver_pool, "inbox_entries")
            .as_array()
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        run_outbox(&sender, 1).await,
        json!({"processed": 0, "errors": []})
    );

    // Operator recovery after fixing the underlying storage problem.
    sender
        .pool
        .get()
        .unwrap()
        .batch_execute(
            "ALTER TABLE outbox_entries DROP CONSTRAINT prevent_completion;
             UPDATE outbox_entries SET status = 0, reservation_id = NULL, reserved_at = NULL, scheduled_at = now() WHERE status = 2",
        )
        .unwrap();
    let accepted = rows(&receiver_pool, "inbox_entries");
    assert_eq!(
        run_outbox(&sender, 1).await,
        json!({"processed": 1, "errors": []})
    );
    assert_eq!(rows(&sender.pool, "outbox_entries")[0]["status"], 5);
    assert_eq!(rows(&receiver_pool, "inbox_entries"), accepted);
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires DATABASE_URL_TEST with CREATE SCHEMA permission"]
async fn postgres_lost_acceptance_response_retries_the_same_message() {
    let (_receiver_schema, receiver_pool) = inbox_test_database();
    let first = Arc::new(AtomicBool::new(true));
    // Stores the message, then loses the first response.
    let endpoint = inbox_endpoint(&receiver_pool).around(move |endpoint, request| {
        let first = first.clone();
        async move {
            let response = endpoint.call(request).await?.into_response();
            if first.swap(false, Ordering::SeqCst) {
                return Ok(StatusCode::INTERNAL_SERVER_ERROR.into_response());
            }
            Ok(response)
        }
    });
    let receiver: Receiver = serve(endpoint);
    let sender = sender(&receiver.url);
    let message_id = record(&sender.pool, OFFER_CREATED_EVENT, &offer_created());

    let report = run_outbox(&sender, 1).await;
    assert_eq!(
        report["errors"],
        json!(["transport error: customer-facing inbox did not return durable acceptance"])
    );
    let failed = rows(&sender.pool, "outbox_entries");
    assert_eq!(failed[0]["status"], 4);
    let accepted = rows(&receiver_pool, "inbox_entries");
    assert_eq!(accepted.as_array().unwrap().len(), 1);

    make_due(&sender.pool);
    assert_eq!(
        run_outbox(&sender, 1).await,
        json!({"processed": 1, "errors": []})
    );

    let delivered = rows(&sender.pool, "outbox_entries");
    assert_eq!(delivered[0]["id"], json!(message_id));
    assert_eq!(delivered[0]["payload"], failed[0]["payload"]);
    assert_eq!(delivered[0]["meta"], failed[0]["meta"]);
    // Both attempts count, the failed one and the delivery.
    assert_eq!(delivered[0]["attempts"], 2);
    assert_eq!(rows(&receiver_pool, "inbox_entries"), accepted);
}
