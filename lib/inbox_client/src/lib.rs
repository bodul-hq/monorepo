//! Blocking HTTP client for a service's durable `/api/inbox`.
//!
//! Only `202 Accepted` with a JSON acknowledgement naming the submitted message
//! UUID confirms acceptance. One deadline covers the request and the
//! acknowledgement body, acknowledgements are bounded, and redirects and
//! automatic retries are disabled. Errors never include URLs, credentials or
//! response bodies.

use std::{env::VarError, time::Duration};

use reqwest::{Client, StatusCode, Url};
use serde::{Deserialize, Serialize};
use tokio::runtime::Handle;
use uuid::Uuid;

pub const TIMEOUT: Duration = Duration::from_secs(10);
pub const MAX_ACK_BYTES: usize = 64 * 1024;

/// The destination could not be configured.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum ConfigError {
    #[error("must be an HTTP(S) origin without credentials, path, query or fragment")]
    InvalidOrigin,
    #[error("HTTP client could not be configured")]
    Client,
}

/// A submission that was not confirmed as durably accepted.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum SubmitError {
    #[error("inbox submission could not be serialized")]
    Payload,
    #[error("inbox request failed")]
    Transport,
    #[error("inbox request timed out")]
    Timeout,
    #[error("inbox did not return durable acceptance")]
    Rejected,
    #[error("inbox acknowledgement is invalid")]
    Acknowledgement,
}

#[derive(Deserialize)]
struct Acknowledgement {
    message_id: String,
}

#[derive(Debug, Clone)]
pub struct InboxClient {
    client: Client,
    inbox_url: Url,
    runtime: Handle,
    timeout: Duration,
}

impl InboxClient {
    /// The client for an origin read from the environment: `None` when the
    /// variable is unset, an error when it is not valid Unicode or not a
    /// valid origin (including when it is empty).
    pub fn from_env_value(
        value: Result<String, VarError>,
        runtime: Handle,
    ) -> Result<Option<Self>, ConfigError> {
        match value {
            Err(VarError::NotPresent) => Ok(None),
            Ok(origin) => Self::new(&origin, runtime).map(Some),
            Err(VarError::NotUnicode(_)) => Err(ConfigError::InvalidOrigin),
        }
    }

    pub fn new(origin: &str, runtime: Handle) -> Result<Self, ConfigError> {
        Self::with_timeout(origin, runtime, TIMEOUT)
    }

    pub fn with_timeout(
        origin: &str,
        runtime: Handle,
        timeout: Duration,
    ) -> Result<Self, ConfigError> {
        let inbox_url = inbox_url(origin)?;
        let client = Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .retry(reqwest::retry::never())
            .timeout(timeout)
            .build()
            .map_err(|_| ConfigError::Client)?;
        Ok(Self {
            client,
            inbox_url,
            runtime,
            timeout,
        })
    }

    /// Posts `submission` and waits for acceptance of `expected`. Blocks on
    /// the configured runtime, so call it from a blocking thread.
    pub fn submit(&self, submission: &impl Serialize, expected: Uuid) -> Result<(), SubmitError> {
        let body = serde_json::to_vec(submission).map_err(|_| SubmitError::Payload)?;
        self.runtime.block_on(async {
            tokio::time::timeout(self.timeout, self.deliver(body, expected))
                .await
                .map_err(|_| SubmitError::Timeout)?
        })
    }

    async fn deliver(&self, body: Vec<u8>, expected: Uuid) -> Result<(), SubmitError> {
        let mut response = self
            .client
            .post(self.inbox_url.clone())
            .header(reqwest::header::CONTENT_TYPE, "application/json")
            .body(body)
            .send()
            .await
            .map_err(|_| SubmitError::Transport)?;
        if response.status() != StatusCode::ACCEPTED {
            return Err(SubmitError::Rejected);
        }
        if response
            .content_length()
            .is_some_and(|length| length > MAX_ACK_BYTES as u64)
        {
            return Err(SubmitError::Acknowledgement);
        }
        let mut bytes = Vec::new();
        while let Some(chunk) = response.chunk().await.map_err(|_| SubmitError::Transport)? {
            if chunk.len() > MAX_ACK_BYTES - bytes.len() {
                return Err(SubmitError::Acknowledgement);
            }
            bytes.extend_from_slice(&chunk);
        }
        let acknowledgement: Acknowledgement =
            serde_json::from_slice(&bytes).map_err(|_| SubmitError::Acknowledgement)?;
        if Uuid::parse_str(&acknowledgement.message_id).ok() != Some(expected) {
            return Err(SubmitError::Acknowledgement);
        }
        Ok(())
    }
}

fn inbox_url(origin: &str) -> Result<Url, ConfigError> {
    let mut url = Url::parse(origin).map_err(|_| ConfigError::InvalidOrigin)?;
    let authority = origin
        .split_once("://")
        .map(|(_, rest)| rest.strip_suffix('/').unwrap_or(rest));
    if authority.is_none_or(|value| value.contains(['/', '\\', '@']))
        || origin.trim() != origin
        || origin.bytes().any(|byte| byte.is_ascii_control())
        || !matches!(url.scheme(), "http" | "https")
        || url.host_str().is_none()
        || !url.username().is_empty()
        || url.password().is_some()
        || url.path() != "/"
        || url.query().is_some()
        || url.fragment().is_some()
    {
        return Err(ConfigError::InvalidOrigin);
    }
    url.set_path("/api/inbox");
    Ok(url)
}

#[cfg(test)]
mod tests;
