use bodul_shared::retailer::RetailerCode;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::platform::bus::Event;

/// An offer's extracted JSON was normalized and stored.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OfferWasNormalized {
    pub offer_id: Uuid,
    pub consolidated_sitemap_id: Uuid,
    pub retailer_code: RetailerCode,
    pub url: String,
}

impl Event for OfferWasNormalized {
    const NAME: &'static str = "OfferWasNormalized";
}
