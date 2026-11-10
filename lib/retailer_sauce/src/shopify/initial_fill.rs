//! Shared contracts for read-only Shopify initial fill operations.

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StorefrontConfig {
    pub acquire_token: bool,
    pub token_required: bool,
    pub storefront_url: &'static str,
    pub graphql_url: &'static str,
    /// Public browser configuration map key, when acquired from HTML/scripts.
    pub token_market: Option<&'static str>,
    /// Optional environment override; otherwise discover the configured browser map.
    pub token_environment: Option<&'static str>,
    pub country: &'static str,
    pub language: &'static str,
    pub currency: &'static str,
}

impl StorefrontConfig {
    /// Resolve public browser credentials before starting a new fill.
    pub fn attempts_token(&self) -> bool {
        self.acquire_token
    }

    pub fn requires_token(&self) -> bool {
        self.token_required
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphqlRequest {
    #[serde(rename = "operationName")]
    pub operation_name: String,
    pub query: String,
    pub variables: Value,
}

/// A public browser token. Serializable for durable commands/events, redacted in logs.
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct StorefrontToken(String);

impl std::fmt::Debug for StorefrontToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("StorefrontToken([redacted])")
    }
}

impl StorefrontToken {
    pub fn new(value: String) -> Result<Self, String> {
        if value.is_empty()
            || !value
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-')
        {
            return Err("invalid public Storefront token".into());
        }
        Ok(Self(value))
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
}
