//! OfferDetailsFetched event.

use serde::{Deserialize, Serialize};
use shared::retailer::RetailerCode;
use uuid::Uuid;

use super::SourceType;
use crate::platform::bus::Event;

/// Records the retailer and source used to fetch offer details.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OfferDetailsFetched {
    pub run_id: Uuid,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub context: Option<retailer_sauce::shopify::markets::MarketContext>,
    pub retailer_code: RetailerCode,
    pub source: SourceType,
    pub fetched_at: chrono::DateTime<chrono::Utc>,
    pub storage: crate::platform::object_storage::OfferStorageMetadata,
}

impl Event for OfferDetailsFetched {
    const NAME: &'static str = "OfferDetailsFetched";
}
