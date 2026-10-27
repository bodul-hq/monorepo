//! Temporary HTTP outbox transport. Only durable inbox acceptance completes a row.

use std::{env::VarError, io};

use inbox_api::io::SubmitInboxRequest;
use inbox_client::{ConfigError, InboxClient, SubmitError};
use outbox::io::{OutboundMessageEnvelope, OutboxError, OutboxPublisherPort};
use tokio::runtime::Handle;

const DESTINATION: &str = "offers";
const COMMAND: &str = "ImportOfferDetails";

#[derive(Clone)]
pub struct OffersPublisher {
    client: InboxClient,
}

impl OffersPublisher {
    pub fn from_env(runtime: Handle) -> io::Result<Option<Self>> {
        Self::from_value(std::env::var("OFFERS_URL"), runtime)
    }

    fn from_value(value: Result<String, VarError>, runtime: Handle) -> io::Result<Option<Self>> {
        InboxClient::from_env_value(value, runtime)
            .map(|client| client.map(|client| Self { client }))
            .map_err(err_configuration)
    }

    /// Builds a publisher for `origin`; startup reads it from the environment.
    #[cfg(test)]
    pub fn new(origin: &str, runtime: Handle) -> io::Result<Self> {
        InboxClient::new(origin, runtime)
            .map(|client| Self { client })
            .map_err(err_configuration)
    }
}

impl OutboxPublisherPort for OffersPublisher {
    /// Called on Mulac's blocking worker; the application owns the async runtime.
    fn publish(&self, envelope: OutboundMessageEnvelope) -> Result<(), OutboxError> {
        let expected = envelope.metadata.message_id;
        let request = submission(envelope)?;
        self.client.submit(&request, expected).map_err(err_delivery)
    }
}

fn submission(envelope: OutboundMessageEnvelope) -> Result<SubmitInboxRequest, OutboxError> {
    if envelope.metadata.routing_key != DESTINATION {
        return Err(err_destination());
    }
    let payload: serde_json::Value =
        serde_json::from_slice(&envelope.payload).map_err(|_| err_payload())?;
    if !payload.is_object() {
        return Err(err_payload());
    }
    let request = SubmitInboxRequest {
        payload: Some(payload),
        routing_key: COMMAND.into(),
        message_id: Some(envelope.metadata.message_id.to_string()),
        correlation_id: envelope.metadata.correlation_id.map(|id| id.to_string()),
        source: Some(
            envelope
                .metadata
                .source
                .unwrap_or_else(|| "retailer_sourcing".into()),
        ),
    };
    request.validate().map_err(|_| err_payload())?;
    Ok(request)
}

fn err_configuration(error: ConfigError) -> io::Error {
    match error {
        ConfigError::InvalidOrigin => io::Error::other(
            "OFFERS_URL must be an HTTP(S) origin without credentials, path, query or fragment",
        ),
        ConfigError::Client => io::Error::other("offers HTTP client could not be configured"),
    }
}
fn err_delivery(error: SubmitError) -> OutboxError {
    match error {
        SubmitError::Payload => err_payload(),
        SubmitError::Transport => err_transport(),
        SubmitError::Timeout => err_timeout(),
        SubmitError::Rejected => err_rejected(),
        SubmitError::Acknowledgement => err_acknowledgement(),
    }
}
fn err_payload() -> OutboxError {
    OutboxError::Conversion("outbox message must contain a valid offers inbox submission".into())
}
fn err_destination() -> OutboxError {
    OutboxError::Routing("outbox message destination must be offers".into())
}
fn err_transport() -> OutboxError {
    OutboxError::Transport("offers inbox request failed".into())
}
fn err_timeout() -> OutboxError {
    OutboxError::Transport("offers inbox request timed out".into())
}
fn err_rejected() -> OutboxError {
    OutboxError::Transport("offers inbox did not return durable acceptance".into())
}
fn err_acknowledgement() -> OutboxError {
    OutboxError::Transport("offers inbox acknowledgement is invalid".into())
}

#[cfg(test)]
pub(crate) mod tests;
