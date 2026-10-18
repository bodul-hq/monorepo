use bodul_shared::retailer::RetailerCode;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::platform::bus::Command;

/// Extracts an offer's downloaded page into JSON.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExtractOfferPage {
    pub offer_id: Uuid,
    pub consolidated_sitemap_id: Uuid,
    pub retailer_code: RetailerCode,
    pub url: String,
}

impl Command for ExtractOfferPage {
    const NAME: &'static str = "ExtractOfferPage";
}
