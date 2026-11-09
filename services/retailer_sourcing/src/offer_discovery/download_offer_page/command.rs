use serde::{Deserialize, Serialize};
use shared::retailer::RetailerCode;
use uuid::Uuid;

use crate::platform::bus::Command;

/// Downloads the page of an offer.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DownloadOfferPage {
    pub offer_id: Uuid,
    pub consolidated_sitemap_id: Uuid,
    pub retailer_code: RetailerCode,
    pub url: String,
}

impl Command for DownloadOfferPage {
    const NAME: &'static str = "DownloadOfferPage";
}
