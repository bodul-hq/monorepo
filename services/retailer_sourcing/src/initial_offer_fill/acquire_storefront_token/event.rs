//! StorefrontTokenAcquired event.

use chrono::{DateTime, Utc};
use retailer_sauce::shopify::initial_fill::StorefrontToken;
use serde::{Deserialize, Serialize};
use shared::retailer::RetailerCode;
use uuid::Uuid;

use crate::platform::bus::Event;

/// Resolves storefront access. None means an optional public token was not found.
/// Historical events containing a token string still deserialize as Some.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StorefrontTokenAcquired {
    pub run_id: Uuid,
    pub storefront_token: Option<StorefrontToken>,
    pub retailer_code: RetailerCode,
    pub url: String,
    pub acquired_at: DateTime<Utc>,
}

impl Event for StorefrontTokenAcquired {
    const NAME: &'static str = "StorefrontTokenAcquired";
}
