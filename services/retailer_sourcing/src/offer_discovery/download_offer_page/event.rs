use crate::platform::object_storage::OfferStorageMetadata;
use chrono::DateTime;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use shared::retailer::RetailerCode;
use uuid::Uuid;

use crate::platform::bus::Event;

/// An offer's page was downloaded and stored.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OfferPageWasDownloaded {
    #[serde(default)]
    pub scraped_at: Option<DateTime<Utc>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub storage: Option<OfferStorageMetadata>,

    pub offer_id: Uuid,
    pub consolidated_sitemap_id: Uuid,
    pub retailer_code: RetailerCode,
    pub url: String,
}

impl Event for OfferPageWasDownloaded {
    const NAME: &'static str = "OfferPageWasDownloaded";
}
