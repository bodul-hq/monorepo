use super::*;
use axum::http::StatusCode;
use axum::{Json, Router, routing::post};
use outbox::io::OutboxEntryMetadata;
use serde_json::{Value, json};
use std::sync::{Arc, Mutex};
use uuid::Uuid;

pub(crate) struct TestServer {
    pub url: String,
    task: tokio::task::JoinHandle<()>,
}

impl Drop for TestServer {
    fn drop(&mut self) {
        self.task.abort();
    }
}

pub(crate) async fn serve(app: Router) -> TestServer {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    let task = tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    TestServer { url, task }
}

pub(crate) fn envelope() -> OutboundMessageEnvelope {
    OutboundMessageEnvelope {
        payload: serde_json::to_vec(&json!({
            "id":Uuid::now_v7(), "created_at":"2026-10-24T12:00:00Z",
            "retailer_code":"AnkerAu", "title":" USB charger ž ",
            "price":{"amount":9007199254740993_i64,"currency":"AUD"},
            "url":"https://shop.example/Charger?variant=2#offer"
        }))
        .unwrap(),
        metadata: OutboxEntryMetadata {
            message_id: Uuid::now_v7(),
            event_id: Uuid::now_v7(),
            correlation_id: Some(Uuid::now_v7()),
            causation_id: Some(Uuid::now_v7()),
            event_type: "OfferCreated".into(),
            routing_key: DESTINATION.into(),
            source: Some("offers".into()),
            content_type: Some("application/json".into()),
        },
    }
}

async fn publish(
    publisher: CustomerFacingPublisher,
    message: OutboundMessageEnvelope,
) -> Result<(), OutboxError> {
    tokio::task::spawn_blocking(move || publisher.publish(message))
        .await
        .unwrap()
}

#[tokio::test(flavor = "multi_thread")]
async fn preserves_the_command_and_delivery_identity_across_retries() {
    let seen = Arc::new(Mutex::new(Vec::<Value>::new()));
    let captured = seen.clone();
    let app = Router::new().route(
        "/api/inbox",
        post(move |Json(body): Json<Value>| {
            let captured = captured.clone();
            async move {
                captured.lock().unwrap().push(body.clone());
                (
                    StatusCode::ACCEPTED,
                    Json(json!({"message_id":body["message_id"]})),
                )
            }
        }),
    );
    let server = serve(app).await;
    let publisher = CustomerFacingPublisher::new(&server.url, Handle::current()).unwrap();
    let message = envelope();
    publish(publisher.clone(), message.clone()).await.unwrap();
    publish(publisher, message.clone()).await.unwrap();
    let requests = seen.lock().unwrap();
    assert_eq!(requests.len(), 2);
    assert_eq!(requests[0], requests[1]);
    assert_eq!(
        requests[0],
        json!({
            "payload":serde_json::from_slice::<Value>(&message.payload).unwrap(),
            "routing_key":"OfferCreated", "message_id":message.metadata.message_id,
            "correlation_id":message.metadata.correlation_id, "source":"offers"
        })
    );
    assert_ne!(message.metadata.message_id, message.metadata.event_id);
}

#[tokio::test(flavor = "multi_thread")]
async fn all_event_types_preserve_raw_payload_and_metadata() {
    let seen = Arc::new(Mutex::new(Vec::<Value>::new()));
    let captured = seen.clone();
    let app = Router::new().route(
        "/api/inbox",
        post(move |Json(body): Json<Value>| {
            let captured = captured.clone();
            async move {
                captured.lock().unwrap().push(body.clone());
                (
                    StatusCode::ACCEPTED,
                    Json(json!({"message_id":body["message_id"]})),
                )
            }
        }),
    );
    let server = serve(app).await;
    let publisher = CustomerFacingPublisher::new(&server.url, Handle::current()).unwrap();
    for (event_type, payload) in [
        (
            OFFER_CREATED_EVENT,
            serde_json::from_slice::<Value>(&envelope().payload).unwrap(),
        ),
        (
            OFFER_TITLE_CHANGED_EVENT,
            json!({"id":Uuid::now_v7(),"title":" ž new title "}),
        ),
        (
            OFFER_PRICE_CHANGED_EVENT,
            json!({"id":Uuid::now_v7(),"price":{"amount":i64::MIN,"currency":"EUR"}}),
        ),
    ] {
        let mut message = envelope();
        message.metadata.event_type = event_type.into();
        message.payload = serde_json::to_vec(&payload).unwrap();
        message.metadata.source = Some(" original source ".into());
        publish(publisher.clone(), message.clone()).await.unwrap();
        let expected = json!({"routing_key":event_type,"payload":payload,
            "message_id":message.metadata.message_id,"correlation_id":message.metadata.correlation_id,
            "source":" original source "});
        assert_eq!(seen.lock().unwrap().last().unwrap(), &expected);
    }
    let mut message = envelope();
    message.metadata.source = None;
    message.metadata.correlation_id = None;
    publish(publisher, message).await.unwrap();
    let captured = seen.lock().unwrap();
    assert_eq!(captured.last().unwrap()["source"], "offers");
    assert!(captured.last().unwrap()["correlation_id"].is_null());
}

#[test]
fn rejects_unknown_types_and_malformed_event_fields() {
    for event_type in ["OfferPriceChanged", "OfferCreated.v1", "Unknown", ""] {
        let mut message = envelope();
        message.metadata.event_type = event_type.into();
        assert!(matches!(
            CustomerFacingPublisher::submission(message),
            Err(OutboxError::Routing(_))
        ));
    }
    for (field, value) in [
        ("id", json!("not-uuid")),
        ("id", json!(42)),
        ("title", json!(null)),
        ("retailer_code", json!("ankerau")),
        ("price", json!({"amount":1.5,"currency":"AUD"})),
        ("price", json!({"amount":1,"currency":"unknown"})),
        ("created_at", json!("not-a-time")),
        ("url", json!(true)),
    ] {
        let mut message = envelope();
        let mut payload: Value = serde_json::from_slice(&message.payload).unwrap();
        payload[field] = value;
        message.payload = serde_json::to_vec(&payload).unwrap();
        assert!(
            CustomerFacingPublisher::submission(message).is_err(),
            "{field}"
        );
    }
    for event_type in [
        OFFER_CREATED_EVENT,
        OFFER_TITLE_CHANGED_EVENT,
        OFFER_PRICE_CHANGED_EVENT,
    ] {
        let mut message = envelope();
        message.metadata.event_type = event_type.into();
        message.payload = serde_json::to_vec(&json!({"id":Uuid::now_v7()})).unwrap();
        assert!(CustomerFacingPublisher::submission(message).is_err());
    }
}
