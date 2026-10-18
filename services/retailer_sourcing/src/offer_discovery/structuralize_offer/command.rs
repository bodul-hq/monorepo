use bodul_shared::retailer::RetailerCode;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::platform::bus::Command;

/// Structuralizes an offer's normalized document.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StructuralizeOffer {
    pub offer_id: Uuid,
    pub consolidated_sitemap_id: Uuid,
    pub retailer_code: RetailerCode,
    pub url: String,
}

impl Command for StructuralizeOffer {
    const NAME: &'static str = "StructuralizeOffer";
}
