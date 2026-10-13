use bodul_shared::retailer::RetailerCode;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::platform::bus::Event;

/// A product URL was recorded as an offer.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OfferDiscovered {
    pub offer_id: Uuid,
    pub consolidated_sitemap_id: Uuid,
    pub retailer_code: RetailerCode,
    pub url: String,
}

impl Event for OfferDiscovered {
    const NAME: &'static str = "OfferDiscovered";
}
