use bodul_shared::retailer::RetailerCode;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::platform::bus::Command;

/// Normalizes an offer's extracted JSON.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NormalizeOffer {
    #[serde(default)]
    pub scraped_at: Option<chrono::DateTime<chrono::Utc>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub storage: Option<crate::platform::object_storage::OfferStorageMetadata>,

    pub offer_id: Uuid,
    pub consolidated_sitemap_id: Uuid,
    pub retailer_code: RetailerCode,
    pub url: String,
}

impl Command for NormalizeOffer {
    const NAME: &'static str = "NormalizeOffer";
}
