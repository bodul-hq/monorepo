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
            sequence: None,
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

#[test]
fn fact_event_submissions_preserve_nulls_and_raw_text_and_reject_partial_updates() {
    use crate::offers::event::{
        OFFER_AVAILABILITY_CHANGED_EVENT, OFFER_CONTENT_CHANGED_EVENT,
        OFFER_ITEM_CONDITION_CHANGED_EVENT, OFFER_PRICING_CHANGED_EVENT,
    };
    let id = Uuid::now_v7();
    for (event_type, payload) in [
        (
            OFFER_PRICING_CHANGED_EVENT,
            json!({"id":id,"pricing":{"price":{"amount":1999,"currency":"AUD"},"was_price":{"amount":2999,"currency":"AUD"}}}),
        ),
        (
            OFFER_PRICING_CHANGED_EVENT,
            json!({"id":id,"pricing":{"price":{"amount":1999,"currency":"AUD"},"was_price":null}}),
        ),
        (
            OFFER_AVAILABILITY_CHANGED_EVENT,
            json!({"id":id,"availability":{"other":" https://schema.org/PreOrder "}}),
        ),
        (
            OFFER_AVAILABILITY_CHANGED_EVENT,
            json!({"id":id,"availability":null}),
        ),
        (
            OFFER_ITEM_CONDITION_CHANGED_EVENT,
            json!({"id":id,"item_condition":"new"}),
        ),
        (
            OFFER_ITEM_CONDITION_CHANGED_EVENT,
            json!({"id":id,"item_condition":null}),
        ),
        (
            OFFER_CONTENT_CHANGED_EVENT,
            json!({"id":id,"content":{"description":" <p>Exact &amp; source ž</p>\n"}}),
        ),
        (
            OFFER_CONTENT_CHANGED_EVENT,
            json!({"id":id,"content":{"description":null}}),
        ),
    ] {
        let message = super::testing::envelope(event_type, &payload);
        let submission = CustomerFacingPublisher::submission(message).unwrap();
        assert_eq!(submission.routing_key, event_type);
        assert_eq!(submission.payload, Some(payload));
        let missing = super::testing::envelope(event_type, &json!({"id":id}));
        assert!(CustomerFacingPublisher::submission(missing).is_err());
    }
    for invalid in [
        json!({"id":id,"pricing":{"price":{"amount":1999,"currency":"AUD"}}}),
        json!({"id":id,"pricing":{"price":{"amount":1999,"currency":"AUD"},"was_price":{"amount":2999,"currency":"USD"}}}),
    ] {
        let message = super::testing::envelope(OFFER_PRICING_CHANGED_EVENT, &invalid);
        assert!(CustomerFacingPublisher::submission(message).is_err());
    }
}

#[test]
fn old_and_current_creation_and_contents_keep_raw_json_and_delivery_metadata() {
    use crate::offers::event::OFFER_CONTENTS_CHANGED_EVENT;
    let old_message = envelope();
    let old: Value = serde_json::from_slice(&old_message.payload).unwrap();
    let mut new = old.clone();
    new["content"] =
        json!([{"language":null,"title":old["title"],"description":" <p>Exact ž</p>\n"}]);
    let contents = json!({"id":old["id"],"content":[{"language":null,"title":" Unknown ","description":""},{"language":"en-US","title":" English ","description":null}],"language":"en-US"});
    for (kind, payload) in [
        (OFFER_CREATED_EVENT, old),
        (OFFER_CREATED_EVENT, new),
        (OFFER_CONTENTS_CHANGED_EVENT, contents.clone()),
    ] {
        let mut message = old_message.clone();
        message.metadata.event_type = kind.into();
        message.metadata.sequence = Some(17);
        message.payload = serde_json::to_vec(&payload).unwrap();
        let request = CustomerFacingPublisher::submission(message.clone()).unwrap();
        assert_eq!(request.payload, Some(payload));
        assert_eq!(request.routing_key, kind);
        let request = serde_json::to_value(request).unwrap();
        assert_eq!(request["message_id"], json!(message.metadata.message_id));
        assert_eq!(
            request["correlation_id"],
            json!(message.metadata.correlation_id)
        );
        assert_eq!(request["source"], json!(message.metadata.source));
    }
    for (content, language) in [
        (json!([]), json!(null)),
        (contents["content"].clone(), json!("fr")),
    ] {
        let invalid = json!({"id":contents["id"],"content":content,"language":language});
        let message = super::testing::envelope(OFFER_CONTENTS_CHANGED_EVENT, &invalid);
        assert!(CustomerFacingPublisher::submission(message).is_err());
    }
}
