use super::*;
use axum::{Json, Router, http::StatusCode, routing::post};
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
            "retailer_code":"AnkerAu", "title":"USB charger",
            "price":{"amount":9007199254740993_i64,"currency":"AUD"},
            "url":"https://shop.example/Charger?variant=2#offer"
        }))
        .unwrap(),
        metadata: OutboxEntryMetadata {
            message_id: Uuid::now_v7(),
            event_id: Uuid::now_v7(),
            correlation_id: Some(Uuid::now_v7()),
            causation_id: Some(Uuid::now_v7()),
            event_type: "OfferExported".into(),
            routing_key: DESTINATION.into(),
            source: Some("retailer_sourcing".into()),
            content_type: Some("application/json".into()),
        },
    }
}

async fn publish(
    publisher: OffersPublisher,
    message: OutboundMessageEnvelope,
) -> Result<(), OutboxError> {
    tokio::task::spawn_blocking(move || publisher.publish(message))
        .await
        .unwrap()
}

#[test]
fn configuration_reads_offers_url_without_leaking_input() {
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let handle = || runtime.handle().clone();
    assert!(
        OffersPublisher::from_value(Err(VarError::NotPresent), handle())
            .unwrap()
            .is_none()
    );
    assert!(OffersPublisher::from_value(Ok("http://localhost:3001".into()), handle()).is_ok());
    for origin in [
        "",
        "http://user:secret@example.com",
        "http://example.com/?token=secret",
    ] {
        let error = OffersPublisher::from_value(Ok(origin.into()), handle())
            .err()
            .unwrap();
        assert_eq!(
            error.to_string(),
            "OFFERS_URL must be an HTTP(S) origin without credentials, path, query or fragment"
        );
    }
    #[cfg(unix)]
    {
        use std::{ffi::OsString, os::unix::ffi::OsStringExt};
        let invalid = VarError::NotUnicode(OsString::from_vec(vec![0xff]));
        assert!(OffersPublisher::from_value(Err(invalid), handle()).is_err());
    }
}

#[test]
fn rejects_bad_destinations_and_payloads_before_http() {
    let mut message = envelope();
    message.metadata.routing_key = "customer_facing".into();
    assert!(matches!(submission(message), Err(OutboxError::Routing(_))));
    for payload in [b"null".as_slice(), b"[]", b"{}", b"\"text\"", b"{"] {
        let mut message = envelope();
        message.payload = payload.to_vec();
        assert!(submission(message).is_err());
    }
    let mut message = envelope();
    message.metadata.source = Some(" ".into());
    assert!(submission(message).is_err());
    let mut message = envelope();
    message.metadata.source = None;
    assert_eq!(
        submission(message).unwrap().source.as_deref(),
        Some("retailer_sourcing")
    );
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
    let publisher = OffersPublisher::new(&server.url, Handle::current()).unwrap();
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
            "routing_key":"ImportOfferDetails", "message_id":message.metadata.message_id,
            "correlation_id":message.metadata.correlation_id, "source":"retailer_sourcing"
        })
    );
    assert_ne!(message.metadata.message_id, message.metadata.event_id);
}

#[tokio::test(flavor = "multi_thread")]
async fn reports_transport_failures_with_offers_messages() {
    let app = Router::new().route(
        "/api/inbox",
        post(|| async { (StatusCode::INTERNAL_SERVER_ERROR, "private response body") }),
    );
    let server = serve(app).await;
    let publisher = OffersPublisher::new(&server.url, Handle::current()).unwrap();
    assert_eq!(
        publish(publisher, envelope())
            .await
            .unwrap_err()
            .to_string(),
        err_rejected().to_string()
    );
    drop(server);
    let unavailable = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let address = format!("http://{}", unavailable.local_addr().unwrap());
    drop(unavailable);
    let publisher = OffersPublisher::new(&address, Handle::current()).unwrap();
    assert_eq!(
        publish(publisher, envelope())
            .await
            .unwrap_err()
            .to_string(),
        err_transport().to_string()
    );
}
