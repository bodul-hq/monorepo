//! FetchOfferDetails command.

use retailer_sauce::shopify::initial_fill::StorefrontToken;
use serde::{Deserialize, Serialize};
use shared::retailer::RetailerCode;
use uuid::Uuid;

use super::SourceType;
use crate::platform::bus::Command;

/// Fetches all selected offer details from the given source.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FetchOfferDetails {
    pub run_id: Uuid,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub context: Option<retailer_sauce::shopify::markets::MarketContext>,
    pub storefront_token: Option<StorefrontToken>,
    pub retailer_code: RetailerCode,
    pub source: SourceType,
}

impl Command for FetchOfferDetails {
    const NAME: &'static str = "FetchOfferDetails";
}
