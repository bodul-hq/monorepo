use serde::{Deserialize, Serialize};
use shared::retailer::RetailerCode;
use uuid::Uuid;

use crate::platform::bus::Command;

/// Records an offer for every product URL of a consolidated sitemap.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DiscoverOffers {
    pub consolidated_sitemap_id: Uuid,
    pub retailer_code: RetailerCode,
}

impl Command for DiscoverOffers {
    const NAME: &'static str = "DiscoverOffers";
}
