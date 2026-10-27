use bodul_shared::retailer::RetailerCode;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::platform::bus::Command;
use crate::platform::object_storage::OfferStorageMetadata;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProcessOfferForExport {
    pub offer_id: Uuid,
    pub consolidated_sitemap_id: Uuid,
    pub retailer_code: RetailerCode,
    pub url: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub storage: Option<OfferStorageMetadata>,
}

impl Command for ProcessOfferForExport {
    const NAME: &'static str = "ProcessOfferForExport";
}
