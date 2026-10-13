use bodul_shared::retailer::RetailerCode;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::platform::bus::Command;

/// Builds the sitemap tree from the files stored for `retrieval_id`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProcessSitemap {
    pub retrieval_id: Uuid,
    pub retailer_code: RetailerCode,
}

impl Command for ProcessSitemap {
    const NAME: &'static str = "ProcessSitemap";
}
