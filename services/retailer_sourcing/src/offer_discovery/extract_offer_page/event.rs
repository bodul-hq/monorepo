use bodul_shared::retailer::RetailerCode;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::platform::bus::Event;

/// An offer's page was extracted into JSON and stored.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OfferPageWasExtracted {
    pub offer_id: Uuid,
    pub consolidated_sitemap_id: Uuid,
    pub retailer_code: RetailerCode,
    pub url: String,
}

impl Event for OfferPageWasExtracted {
    const NAME: &'static str = "OfferPageWasExtracted";
}
