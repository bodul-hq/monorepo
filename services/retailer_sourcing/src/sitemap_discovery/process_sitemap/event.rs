use bodul_shared::retailer::RetailerCode;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::platform::bus::Event;

/// A retrieval's sitemap files were parsed into one tree and stored.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SitemapProcessed {
    pub retrieval_id: Uuid,
    pub processed_sitemap_id: Uuid,
    pub retailer_code: RetailerCode,
    pub url_count: usize,
}

impl Event for SitemapProcessed {
    const NAME: &'static str = "SitemapProcessed";
}
