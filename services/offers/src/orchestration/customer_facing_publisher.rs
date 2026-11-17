//! HTTP outbox delivery to customer-facing's durable inbox. Only a `202` that
//! acknowledges the same message UUID completes a row; Mulac's outbox service
//! owns reservation, completion and retries.

use crate::offers::event::OFFER_AVAILABILITY_CHANGED_EVENT;
use crate::offers::event::OFFER_CONTENT_CHANGED_EVENT;
use crate::offers::event::OFFER_CONTENTS_CHANGED_EVENT;
use crate::offers::event::OFFER_IDENTIFIERS_CHANGED_EVENT;
use crate::offers::event::OFFER_ITEM_CONDITION_CHANGED_EVENT;
use crate::offers::event::OFFER_LOCALIZATION_CHANGED_EVENT;
use crate::offers::event::OFFER_PRICING_CHANGED_EVENT;
use crate::offers::event::OFFER_SCRAPED_AT_UPDATED_EVENT;
use crate::offers::event::OfferAvailabilityChangedEvent;
use crate::offers::event::OfferContentChangedEvent;
use crate::offers::event::OfferContentsChangedEvent;
use crate::offers::event::OfferIdentifiersChangedEvent;
use crate::offers::event::OfferItemConditionChangedEvent;
use crate::offers::event::OfferLocalizationChangedEvent;
use crate::offers::event::OfferPricingChangedEvent;
use crate::offers::event::OfferScrapedAtUpdatedEvent;
use env::var;
use io::Error as IoError;
use io::Result as IoResult;
use serde::de::Error as DeError;
use serde_json::Error as SerdeJsonError;
use serde_json::from_slice as serde_json_from_slice;
use serde_json::from_value;
use shared::images::is_valid;
use std::{env, io};
use uuid::Uuid;

use inbox::prelude::SubmitInboxRequest;
use inbox_client::{ConfigError, InboxClient, SubmitError};
use outbox::prelude::{OutboundMessageEnvelope, OutboxError, OutboxPublisherPort};
use serde_json::Value;
use tokio::runtime::Handle;

use crate::offers::event::{
    OFFER_CREATED_EVENT, OFFER_IMAGE_ADDED_EVENT, OFFER_PRICE_CHANGED_EVENT,
    OFFER_TITLE_CHANGED_EVENT, OfferCreatedEvent, OfferImageAddedEvent, OfferPriceChangedEvent,
    OfferTitleChangedEvent,
};

/// The outbox destination `CustomerFacingSubscriber` records.
pub const DESTINATION: &str = "customer_facing";
const DEFAULT_SOURCE: &str = "offers";

#[derive(Clone)]
pub struct CustomerFacingPublisher {
    client: InboxClient,
}

impl CustomerFacingPublisher {
    /// Builds the publisher from `CUSTOMER_FACING_URL`; `None` when it is unset.
    pub fn from_env(runtime: Handle) -> IoResult<Option<Self>> {
        InboxClient::from_env_value(var("CUSTOMER_FACING_URL"), runtime)
            .map(|client| client.map(|client| Self { client }))
            .map_err(err_configuration)
    }

    /// Builds a publisher for `origin`; startup reads it from the environment.
    #[cfg(test)]
    pub fn new(origin: &str, runtime: Handle) -> IoResult<Self> {
        InboxClient::new(origin, runtime)
            .map(|client| Self { client })
            .map_err(err_configuration)
    }

    /// The inbox submission for an outbox row: its event type as routing key
    /// and its raw event payload, message UUID, correlation and source.
    fn submission(envelope: OutboundMessageEnvelope) -> Result<SubmitInboxRequest, OutboxError> {
        let metadata = envelope.metadata;
        Self::guard_destination(&metadata.routing_key)?;
        let payload: Value = serde_json_from_slice(&envelope.payload).map_err(|_| err_payload())?;
        Self::guard_payload(&metadata.event_type, &payload)?;
        let request = SubmitInboxRequest {
            payload: Some(payload),
            routing_key: metadata.event_type,
            message_id: Some(metadata.message_id.to_string()),
            correlation_id: metadata.correlation_id.map(|id| id.to_string()),
            source: Some(metadata.source.unwrap_or_else(|| DEFAULT_SOURCE.into())),
        };
        request.validate().map_err(|_| err_payload())?;
        Ok(request)
    }

    fn guard_destination(routing_key: &str) -> Result<(), OutboxError> {
        if routing_key != DESTINATION {
            return Err(err_destination());
        }
        Ok(())
    }

    /// Only the supported events are sent, and each payload must be that
    /// event; the original JSON is sent unchanged.
    fn guard_payload(event_type: &str, payload: &Value) -> Result<(), OutboxError> {
        if !payload.is_object() {
            return Err(err_payload());
        }
        let parsed = match event_type {
            OFFER_CONTENTS_CHANGED_EVENT => {
                from_value::<OfferContentsChangedEvent>(payload.clone())
                    .and_then(|event| event.guard().map_err(<SerdeJsonError as DeError>::custom))
            }
            OFFER_CONTENT_CHANGED_EVENT => from_value::<OfferContentChangedEvent>(payload.clone())
                .and_then(|event| {
                    event
                        .content
                        .guard()
                        .map_err(<SerdeJsonError as DeError>::custom)
                }),
            OFFER_ITEM_CONDITION_CHANGED_EVENT => {
                from_value::<OfferItemConditionChangedEvent>(payload.clone()).and_then(|event| {
                    event
                        .item_condition
                        .as_ref()
                        .map_or(Ok(()), |value| value.guard())
                        .map_err(<SerdeJsonError as DeError>::custom)
                })
            }
            OFFER_AVAILABILITY_CHANGED_EVENT => {
                from_value::<OfferAvailabilityChangedEvent>(payload.clone()).and_then(|event| {
                    event
                        .availability
                        .as_ref()
                        .map_or(Ok(()), |value| value.guard())
                        .map_err(<SerdeJsonError as DeError>::custom)
                })
            }
            OFFER_PRICING_CHANGED_EVENT => from_value::<OfferPricingChangedEvent>(payload.clone())
                .and_then(|event| {
                    event
                        .pricing
                        .guard()
                        .map_err(<SerdeJsonError as DeError>::custom)
                }),
            OFFER_IDENTIFIERS_CHANGED_EVENT => {
                from_value::<OfferIdentifiersChangedEvent>(payload.clone()).map(|_| ())
            }
            OFFER_LOCALIZATION_CHANGED_EVENT => {
                from_value::<OfferLocalizationChangedEvent>(payload.clone()).map(|_| ())
            }
            OFFER_SCRAPED_AT_UPDATED_EVENT => {
                from_value::<OfferScrapedAtUpdatedEvent>(payload.clone()).map(|_| ())
            }
            OFFER_IMAGE_ADDED_EVENT => from_value::<OfferImageAddedEvent>(payload.clone())
                .and_then(|event| {
                    if is_valid(&event.image_url) {
                        Ok(())
                    } else {
                        Err(<SerdeJsonError as DeError>::custom("invalid image URL"))
                    }
                }),
            OFFER_CREATED_EVENT => from_value::<OfferCreatedEvent>(payload.clone())
                .and_then(|event| event.guard().map_err(<SerdeJsonError as DeError>::custom)),
            OFFER_TITLE_CHANGED_EVENT => {
                from_value::<OfferTitleChangedEvent>(payload.clone()).map(drop)
            }
            OFFER_PRICE_CHANGED_EVENT => {
                from_value::<OfferPriceChangedEvent>(payload.clone()).map(drop)
            }
            _ => return Err(err_event_type()),
        };
        parsed.map_err(|_| err_payload())?;
        let id = payload["id"].as_str().ok_or_else(err_payload)?;
        Uuid::parse_str(id).map_err(|_| err_payload())?;
        Ok(())
    }
}

impl OutboxPublisherPort for CustomerFacingPublisher {
    /// Called on Mulac's blocking worker; the application owns the async runtime.
    fn publish(&self, envelope: OutboundMessageEnvelope) -> Result<(), OutboxError> {
        let expected = envelope.metadata.message_id;
        let request = Self::submission(envelope)?;
        self.client.submit(&request, expected).map_err(err_delivery)
    }
}

fn err_configuration(error: ConfigError) -> IoError {
    match error {
        ConfigError::InvalidOrigin => IoError::other(
            "CUSTOMER_FACING_URL must be an HTTP(S) origin without credentials, path, query or fragment; it must be valid Unicode",
        ),
        ConfigError::Client => {
            IoError::other("customer-facing HTTP client could not be configured")
        }
    }
}

fn err_destination() -> OutboxError {
    OutboxError::Routing("outbox message destination must be customer_facing".into())
}

fn err_event_type() -> OutboxError {
    OutboxError::Routing("outbox message event type must be a supported offer event".into())
}

/// Never names the payload's content.
fn err_payload() -> OutboxError {
    OutboxError::Conversion(
        "outbox message must contain a valid customer-facing inbox submission".into(),
    )
}

fn err_delivery(error: SubmitError) -> OutboxError {
    match error {
        SubmitError::Payload => err_payload(),
        SubmitError::Transport => err_transport("customer-facing inbox request failed"),
        SubmitError::Timeout => err_transport("customer-facing inbox request timed out"),
        SubmitError::Rejected => {
            err_transport("customer-facing inbox did not return durable acceptance")
        }
        SubmitError::Acknowledgement => {
            err_transport("customer-facing inbox acknowledgement is invalid")
        }
    }
}

fn err_transport(message: &str) -> OutboxError {
    OutboxError::Transport(message.into())
}

#[cfg(test)]
pub(crate) mod testing {
    use super::DESTINATION as SUPER_DESTINATION;
    use serde_json::to_vec as serde_json_to_vec;
    use std::net::TcpListener;
    use tokio::spawn;
    use tokio::task::JoinHandle;

    use outbox::prelude::{OutboundMessageEnvelope, OutboxEntryMetadata};
    use poem::{Endpoint, Server, listener::TcpAcceptor};
    use serde_json::{Value, json};
    use uuid::Uuid;

    /// An HTTP server on an ephemeral port, stopped when dropped.
    pub struct Receiver {
        pub url: String,
        task: JoinHandle<()>,
    }

    impl Drop for Receiver {
        fn drop(&mut self) {
            self.task.abort();
        }
    }

    pub fn serve(endpoint: impl Endpoint + 'static) -> Receiver {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        listener.set_nonblocking(true).unwrap();
        let url = format!("http://{}", listener.local_addr().unwrap());
        let acceptor = TcpAcceptor::from_std(listener).unwrap();
        let task = spawn(async move {
            let _ = Server::new_with_acceptor(acceptor).run(endpoint).await;
        });
        Receiver { url, task }
    }

    pub fn offer_created() -> Value {
        json!({
            "id": Uuid::new_v4(),
            "retailer_code": "AnkerAu",
            "title": "USB charger",
            "price": {"amount": 1999, "currency": "AUD"},
            "url": "https://shop.example/Charger?variant=123#details",
            "created_at": "2026-10-24T12:00:00Z"
        })
    }

    pub fn envelope(event_type: &str, payload: &Value) -> OutboundMessageEnvelope {
        let event_id = Uuid::new_v4();
        let metadata = OutboxEntryMetadata {
            sequence: None,
            event_id,
            message_id: event_id,
            correlation_id: Some(Uuid::new_v4()),
            causation_id: Some(Uuid::new_v4()),
            event_type: event_type.into(),
            routing_key: SUPER_DESTINATION.into(),
            source: Some("offers-api".into()),
            content_type: Some("application/json".into()),
        };
        OutboundMessageEnvelope {
            payload: serde_json_to_vec(payload).unwrap(),
            metadata,
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::from_slice as decode_json_slice;

    use env::current_exe;
    use env::var_os;
    use serde_json::Value as SerdeJsonValue;
    use serde_json::to_value;
    use std::net::TcpListener;
    use std::process::Command;
    use tokio::runtime::Runtime;
    use tokio::task::spawn_blocking;
    use tokio::test as tokio_test;

    use std::ffi::OsStr;
    use std::sync::{Arc, Mutex};

    use poem::{
        EndpointExt, Route, handler, http::StatusCode, post, web::Data, web::Json as PoemJson,
    };
    use serde_json::json;
    use uuid::Uuid;

    use super::testing::{envelope, offer_created, serve};
    use super::*;

    const FROM_ENV_PROBE: &str = "OFFERS_CUSTOMER_FACING_FROM_ENV_PROBE";

    /// Runs `from_env` in a child test process so that tests never mutate this
    /// process's environment.
    fn from_env_in_child(vars: &[(&str, &OsStr)]) -> String {
        let mut command = Command::new(current_exe().unwrap());
        command
            .args([
                "--exact",
                "orchestration::customer_facing_publisher::tests::from_env_probe",
                "--nocapture",
                "--test-threads=1",
            ])
            .env(FROM_ENV_PROBE, "1")
            .env_remove("CUSTOMER_FACING_URL")
            .env_remove("AMQP_URL")
            .env_remove("OUTBOX_EXCHANGE");
        for (name, value) in vars {
            command.env(name, value);
        }
        let output = command.output().unwrap();
        assert!(output.status.success(), "{output:?}");
        String::from_utf8_lossy(&output.stdout)
            .lines()
            .find_map(|line| line.split_once("FROM_ENV:").map(|(_, result)| result))
            .unwrap_or_else(|| panic!("probe printed no result: {output:?}"))
            .to_owned()
    }

    #[test]
    fn from_env_probe() {
        if var_os(FROM_ENV_PROBE).is_none() {
            return;
        }
        let runtime = Runtime::new().unwrap();
        match CustomerFacingPublisher::from_env(runtime.handle().clone()) {
            Ok(None) => println!("FROM_ENV:none"),
            Ok(Some(_)) => println!("FROM_ENV:configured"),
            Err(err) => println!("FROM_ENV:error={err}"),
        }
    }

    #[test]
    fn from_env_reads_customer_facing_url_and_ignores_legacy_amqp_settings() {
        let config_error = format!("error={}", err_configuration(ConfigError::InvalidOrigin));
        assert_eq!(from_env_in_child(&[]), "none");
        let legacy = [
            (
                "AMQP_URL",
                OsStr::new("amqp://offers:secret@127.0.0.1:5672/%2f"),
            ),
            ("OUTBOX_EXCHANGE", OsStr::new("offers")),
        ];
        assert_eq!(from_env_in_child(&legacy), "none");
        assert_eq!(
            from_env_in_child(&[("CUSTOMER_FACING_URL", OsStr::new("http://127.0.0.1:3003"))]),
            "configured"
        );
        for url in [
            "",
            " http://localhost",
            "amqp://localhost",
            "http://user:secret@localhost",
            "http://localhost/api",
            "http://localhost/?token=secret",
        ] {
            let result = from_env_in_child(&[("CUSTOMER_FACING_URL", OsStr::new(url))]);
            assert_eq!(result, config_error, "{url:?}");
            assert!(!result.contains("secret"));
        }
        #[cfg(unix)]
        {
            use std::os::unix::ffi::OsStrExt;
            let invalid = OsStr::from_bytes(&[0xff]);
            assert_eq!(
                from_env_in_child(&[("CUSTOMER_FACING_URL", invalid)]),
                config_error
            );
        }
    }

    #[test]
    fn submits_each_supported_event_with_its_type_and_raw_payload() {
        let id = Uuid::new_v4();
        let price = json!({"amount": 9007199254740993_i64, "currency": "AUD"});
        for (event_type, payload) in [
            (OFFER_CREATED_EVENT, offer_created()),
            (
                OFFER_IMAGE_ADDED_EVENT,
                json!({"id":id,"image_url":"https://shop.example/A?x=1&y=2","position":u32::MAX}),
            ),
            (
                OFFER_TITLE_CHANGED_EVENT,
                json!({"id": id, "title": " New title ž "}),
            ),
            (OFFER_PRICE_CHANGED_EVENT, json!({"id": id, "price": price})),
        ] {
            let message = envelope(event_type, &payload);
            let metadata = message.metadata.clone();

            let request = CustomerFacingPublisher::submission(message).unwrap();

            assert_eq!(
                to_value(&request).unwrap(),
                json!({
                    "payload": payload,
                    "routing_key": event_type,
                    "message_id": metadata.message_id,
                    "correlation_id": metadata.correlation_id,
                    "source": "offers-api",
                })
            );
        }
    }

    #[test]
    fn defaults_an_absent_source_and_rejects_a_blank_one() {
        let mut message = envelope(OFFER_CREATED_EVENT, &offer_created());
        message.metadata.source = None;
        message.metadata.correlation_id = None;
        let request = CustomerFacingPublisher::submission(message).unwrap();
        assert_eq!(request.source.as_deref(), Some("offers"));
        assert_eq!(request.correlation_id, None);

        let mut message = envelope(OFFER_CREATED_EVENT, &offer_created());
        message.metadata.source = Some(" ".into());
        assert!(matches!(
            CustomerFacingPublisher::submission(message),
            Err(OutboxError::Conversion(_))
        ));
    }

    #[test]
    fn rejects_other_destinations_types_and_malformed_payloads_before_http() {
        let mut other_destination = envelope(OFFER_CREATED_EVENT, &offer_created());
        other_destination.metadata.routing_key = "offers".into();
        assert!(matches!(
            CustomerFacingPublisher::submission(other_destination),
            Err(OutboxError::Routing(_))
        ));
        for event_type in ["OfferPriceChanged", "OfferDeleted", ""] {
            assert!(matches!(
                CustomerFacingPublisher::submission(envelope(event_type, &offer_created())),
                Err(OutboxError::Routing(_))
            ));
        }
        let missing_title = json!({"id": Uuid::new_v4()});
        for (event_type, payload) in [
            (OFFER_CREATED_EVENT, json!(null)),
            (OFFER_CREATED_EVENT, json!([offer_created()])),
            (OFFER_CREATED_EVENT, json!(offer_created().to_string())),
            (OFFER_CREATED_EVENT, json!({"id": Uuid::new_v4()})),
            (OFFER_TITLE_CHANGED_EVENT, missing_title),
            (
                OFFER_PRICE_CHANGED_EVENT,
                json!({"id": "not-a-uuid", "price": {}}),
            ),
        ] {
            let result = CustomerFacingPublisher::submission(envelope(event_type, &payload));
            assert!(
                matches!(result, Err(OutboxError::Conversion(_))),
                "{event_type} {payload}"
            );
        }
        let mut not_json = envelope(OFFER_CREATED_EVENT, &offer_created());
        not_json.payload = b"{".to_vec();
        assert!(matches!(
            CustomerFacingPublisher::submission(not_json),
            Err(OutboxError::Conversion(_))
        ));
    }

    #[handler]
    fn accept(
        Data(seen): Data<&Arc<Mutex<Vec<SerdeJsonValue>>>>,
        PoemJson(body): PoemJson<SerdeJsonValue>,
    ) -> (StatusCode, PoemJson<SerdeJsonValue>) {
        seen.lock().unwrap().push(body.clone());
        (
            StatusCode::ACCEPTED,
            PoemJson(json!({"message_id": body["message_id"]})),
        )
    }

    #[handler]
    fn reject() -> (StatusCode, &'static str) {
        (StatusCode::TOO_MANY_REQUESTS, "private response body")
    }

    async fn publish(
        publisher: CustomerFacingPublisher,
        message: OutboundMessageEnvelope,
    ) -> Result<(), OutboxError> {
        spawn_blocking(move || publisher.publish(message))
            .await
            .unwrap()
    }

    #[tokio_test(flavor = "multi_thread")]
    async fn repeated_acceptance_keeps_identity_and_content() {
        let seen = Arc::new(Mutex::new(Vec::<SerdeJsonValue>::new()));
        let receiver = serve(
            Route::new()
                .at("/api/inbox", post(accept))
                .data(seen.clone()),
        );
        let publisher = CustomerFacingPublisher::new(&receiver.url, Handle::current()).unwrap();
        let message = envelope(OFFER_CREATED_EVENT, &offer_created());

        publish(publisher.clone(), message.clone()).await.unwrap();
        publish(publisher, message.clone()).await.unwrap();

        let requests = seen.lock().unwrap();
        assert_eq!(requests.len(), 2);
        assert_eq!(requests[0], requests[1]);
        assert_eq!(
            requests[0]["message_id"],
            json!(message.metadata.message_id)
        );
        assert_eq!(
            requests[0]["payload"],
            decode_json_slice::<SerdeJsonValue>(&message.payload).unwrap()
        );
    }

    #[tokio_test(flavor = "multi_thread")]
    async fn failures_use_sanitized_customer_facing_messages() {
        let receiver = serve(Route::new().at("/api/inbox", post(reject)));
        let publisher = CustomerFacingPublisher::new(&receiver.url, Handle::current()).unwrap();
        let error = publish(publisher, envelope(OFFER_CREATED_EVENT, &offer_created()))
            .await
            .unwrap_err()
            .to_string();
        assert_eq!(
            error,
            "transport error: customer-facing inbox did not return durable acceptance"
        );
        assert!(!error.contains("private"));
        drop(receiver);

        let unavailable = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = format!("http://{}", unavailable.local_addr().unwrap());
        drop(unavailable);
        let publisher = CustomerFacingPublisher::new(&address, Handle::current()).unwrap();
        assert_eq!(
            publish(publisher, envelope(OFFER_CREATED_EVENT, &offer_created()))
                .await
                .unwrap_err()
                .to_string(),
            "transport error: customer-facing inbox request failed"
        );
    }

    #[test]
    fn creation_images_keep_the_raw_payload_and_invalid_lists_fail() {
        let mut payload = offer_created();
        payload["images"] = json!(["https://shop.example/A?x=1&y=2", "https://shop.example/B"]);
        let message = envelope(OFFER_CREATED_EVENT, &payload);
        let id = message.metadata.message_id;
        let result = CustomerFacingPublisher::submission(message).unwrap();
        let result = to_value(result).unwrap();
        assert_eq!(result["payload"], payload);
        assert_eq!(result["message_id"], json!(id));
        for images in [
            json!(null),
            json!("url"),
            json!([1]),
            json!(["/bad"]),
            json!(["https://shop.example/A", "https://shop.example/A"]),
        ] {
            payload["images"] = images;
            assert!(
                CustomerFacingPublisher::submission(envelope(OFFER_CREATED_EVENT, &payload))
                    .is_err()
            );
        }
    }
}

#[cfg(test)]
mod postgres_tests;

#[cfg(test)]
mod regression_tests;
