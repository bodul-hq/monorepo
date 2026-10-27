use super::{tests::run, *};
use crate::platform::{
    offers_publisher::{
        OffersPublisher,
        tests::{envelope, serve},
    },
    test_db::{Database, json_query},
};
use axum::response::IntoResponse;
use axum::{Router, extract::State, routing::post};
use diesel::{RunQueryDsl, connection::SimpleConnection, sql_types};
use inbox_api::io::InboxService;
use mulac_axum::io::{InboxAdapter, SubmitInbox};
use outbox::io::{OutboundMessageEnvelope, OutboxStatus};
use outbox_api::io::OutboxService;
use serde_json::{Value, json};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

fn seed(db: &Database, message: &OutboundMessageEnvelope) {
    diesel::sql_query("INSERT INTO outbox_entries(id,payload,meta) VALUES($1,$2,$3)")
        .bind::<sql_types::Uuid, _>(message.metadata.message_id)
        .bind::<sql_types::Text, _>(String::from_utf8(message.payload.clone()).unwrap())
        .bind::<sql_types::Jsonb, _>(serde_json::to_value(&message.metadata).unwrap())
        .execute(&mut db.pool.get().unwrap())
        .unwrap();
}

fn snapshot(db: &Database) -> Value {
    json_query(
        &mut db.pool.get().unwrap(),
        "SELECT coalesce(jsonb_agg(to_jsonb(t) ORDER BY id),'[]') AS value FROM outbox_entries t",
    )
}

pub(crate) fn state(db: &Database, url: &str) -> AppState {
    let publisher = OffersPublisher::new(url, tokio::runtime::Handle::current()).unwrap();
    let service = OutboxService::from_postgres(db.pool.clone(), Arc::new(publisher));
    AppState {
        outbox: Some(OutboxAdapter::new(service)),
        ..Default::default()
    }
}

pub(crate) async fn inbox(db: &Database) -> crate::platform::offers_publisher::tests::TestServer {
    let adapter = InboxAdapter::new(InboxService::from_postgres(db.pool.clone()));
    let app = Router::new()
        .route(
            "/api/inbox",
            post(
                |State(adapter): State<InboxAdapter>, request: SubmitInbox| async move {
                    adapter.submit(request).await
                },
            ),
        )
        .with_state(adapter);
    serve(app).await
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires DATABASE_URL_TEST; run make test-postgres"]
async fn confirms_durable_acceptance_with_limits_and_duplicate_ids() {
    let db = Database::new();
    let receiver = Database::new();
    let server = inbox(&receiver).await;
    let message = envelope();
    seed(&db, &message);
    seed(&db, &envelope());
    let before = snapshot(&db);
    assert_eq!(
        run(AppState::default(), json!({"limit":10})).await.0,
        axum::http::StatusCode::SERVICE_UNAVAILABLE
    );
    assert_eq!(snapshot(&db), before);
    let app_state = state(&db, &server.url);
    assert_eq!(
        run(app_state.clone(), json!({"limit":1})).await.1,
        json!({"processed":1,"errors":[]})
    );
    let rows = snapshot(&db);
    assert_eq!(
        rows.as_array()
            .unwrap()
            .iter()
            .filter(|r| r["status"] == OutboxStatus::Completed as i32)
            .count(),
        1
    );
    let mut conn = db.pool.get().unwrap();
    conn.batch_execute(
        "UPDATE outbox_entries SET status=0, reservation_id=NULL, reserved_at=NULL WHERE status=5",
    )
    .unwrap();
    assert_eq!(
        run(app_state, json!({"limit":10})).await.1,
        json!({"processed":2,"errors":[]})
    );
    let records = json_query(
        &mut receiver.pool.get().unwrap(),
        "SELECT jsonb_agg(to_jsonb(t) ORDER BY id) AS value FROM inbox_entries t",
    );
    assert_eq!(records.as_array().unwrap().len(), 2);
    assert_eq!(records[0]["id"], json!(message.metadata.message_id));
    assert_eq!(
        serde_json::from_str::<Value>(records[0]["payload"].as_str().unwrap()).unwrap(),
        serde_json::from_slice::<Value>(&message.payload).unwrap()
    );
    assert_eq!(records[0]["meta"]["routing_key"], "ImportOfferDetails");
    assert_eq!(
        records[0]["meta"]["correlation_id"],
        json!(message.metadata.correlation_id)
    );
    assert_eq!(
        json_query(
            &mut receiver.pool.get().unwrap(),
            "SELECT to_jsonb(count(*)) AS value FROM command_entries"
        ),
        0
    );
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires DATABASE_URL_TEST; run make test-postgres"]
async fn partial_failure_keeps_success_and_schedules_retry() {
    let db = Database::new();
    let receiver = Database::new();
    let server = inbox(&receiver).await;
    seed(&db, &envelope());
    let mut bad = envelope();
    bad.metadata.routing_key = "wrong".into();
    seed(&db, &bad);
    let result = run(state(&db, &server.url), json!({"limit":10})).await.1;
    assert!(result["processed"].is_null());
    assert_eq!(result["errors"].as_array().unwrap().len(), 1);
    let rows = snapshot(&db);
    assert_eq!(rows[0]["status"], OutboxStatus::Completed as i32);
    assert_eq!(rows[1]["status"], OutboxStatus::Failed as i32);
    assert_eq!(rows[1]["attempts"], 1);
    assert!(rows[1]["reservation_id"].is_null());
    assert_eq!(
        json_query(
            &mut db.pool.get().unwrap(),
            "SELECT to_jsonb(scheduled_at > now()) AS value FROM outbox_entries WHERE status=4"
        ),
        true
    );
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires DATABASE_URL_TEST; run make test-postgres"]
async fn lost_completion_can_redeliver_without_duplicate_inbox_rows() {
    let db = Database::new();
    let receiver = Database::new();
    let server = inbox(&receiver).await;
    seed(&db, &envelope());
    db.pool
        .get()
        .unwrap()
        .batch_execute(
            "ALTER TABLE outbox_entries ADD CONSTRAINT prevent_completion CHECK(status <> 5)",
        )
        .unwrap();
    let app_state = state(&db, &server.url);
    let result = run(app_state.clone(), json!({"limit":1})).await.1;
    assert!(result["processed"].is_null());
    assert_eq!(snapshot(&db)[0]["status"], OutboxStatus::Reserved as i32);
    // Simulate explicit operator recovery of a stale reservation after fixing storage.
    db.pool.get().unwrap().batch_execute("ALTER TABLE outbox_entries DROP CONSTRAINT prevent_completion; UPDATE outbox_entries SET status=0,reservation_id=NULL,reserved_at=NULL,scheduled_at=now()").unwrap();
    assert_eq!(
        run(app_state, json!({"limit":1})).await.1,
        json!({"processed":1,"errors":[]})
    );
    assert_eq!(
        json_query(
            &mut receiver.pool.get().unwrap(),
            "SELECT to_jsonb(count(*)) AS value FROM inbox_entries"
        ),
        1
    );
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires DATABASE_URL_TEST; run make test-postgres"]
async fn a_lost_acceptance_response_retries_the_same_inbox_message() {
    let db = Database::new();
    let receiver = Database::new();
    let adapter = InboxAdapter::new(InboxService::from_postgres(receiver.pool.clone()));
    let first = Arc::new(AtomicBool::new(true));
    let app = Router::new().route(
        "/api/inbox",
        post(move |request: SubmitInbox| {
            let adapter = adapter.clone();
            let first = first.clone();
            async move {
                let stored = adapter.submit(request).await.unwrap();
                if first.swap(false, Ordering::SeqCst) {
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR.into_response()
                } else {
                    stored.into_response()
                }
            }
        }),
    );
    let server = serve(app).await;
    seed(&db, &envelope());
    let app_state = state(&db, &server.url);
    assert!(run(app_state.clone(), json!({"limit":1})).await.1["processed"].is_null());
    assert_eq!(snapshot(&db)[0]["status"], OutboxStatus::Failed as i32);
    let before = json_query(
        &mut receiver.pool.get().unwrap(),
        "SELECT jsonb_agg(to_jsonb(t)) AS value FROM inbox_entries t",
    );
    assert_eq!(before.as_array().unwrap().len(), 1);
    db.pool
        .get()
        .unwrap()
        .batch_execute("UPDATE outbox_entries SET scheduled_at=now()")
        .unwrap();
    assert_eq!(
        run(app_state, json!({"limit":1})).await.1,
        json!({"processed":1,"errors":[]})
    );
    let after = json_query(
        &mut receiver.pool.get().unwrap(),
        "SELECT jsonb_agg(to_jsonb(t)) AS value FROM inbox_entries t",
    );
    assert_eq!(after, before);
}
