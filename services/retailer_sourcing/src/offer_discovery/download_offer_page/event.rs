use bodul_shared::retailer::RetailerCode;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::platform::bus::Event;

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
