use bodul_shared::retailer::RetailerCode;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::platform::bus::{Event, Publish};

/// An offer's page was downloaded and stored.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OfferPageWasDownloaded {
    pub offer_id: Uuid,
    pub consolidated_sitemap_id: Uuid,
    pub retailer_code: RetailerCode,
    pub url: String,
}

impl Event for OfferPageWasDownloaded {
    const NAME: &'static str = "OfferPageWasDownloaded";
}

/// An offer's page wasn't downloaded because its URL was fetched recently.
/// `raw_offer_id` is the page stored then.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OfferPageDownloadSkipped {
    pub offer_id: Uuid,
    pub consolidated_sitemap_id: Uuid,
    pub retailer_code: RetailerCode,
    pub url: String,
    pub raw_offer_id: Uuid,
}

impl Event for OfferPageDownloadSkipped {
    const NAME: &'static str = "OfferPageDownloadSkipped";
}

/// The event one download produces.
#[derive(Debug, PartialEq, Serialize)]
#[serde(untagged)]
pub enum DownloadOutcome {
    Downloaded(OfferPageWasDownloaded),
    Skipped(OfferPageDownloadSkipped),
}

impl Publish for DownloadOutcome {
    fn event_type(&self) -> &'static str {
        match self {
            Self::Downloaded(_) => OfferPageWasDownloaded::NAME,
            Self::Skipped(_) => OfferPageDownloadSkipped::NAME,
        }
    }
}
