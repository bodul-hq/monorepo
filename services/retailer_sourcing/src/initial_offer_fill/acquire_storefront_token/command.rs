//! AcquireStorefrontToken command.

use serde::{Deserialize, Serialize};
use shared::retailer::RetailerCode;
use uuid::Uuid;

use crate::platform::bus::Command;

/// Acquires a storefront token from the given retailer URL.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcquireStorefrontToken {
    pub run_id: Uuid,
    pub retailer_code: RetailerCode,
    pub url: String,
}

impl Command for AcquireStorefrontToken {
    const NAME: &'static str = "AcquireStorefrontToken";
}
