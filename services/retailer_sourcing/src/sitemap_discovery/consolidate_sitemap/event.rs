use bodul_shared::retailer::RetailerCode;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::platform::bus::Event;

/// A processed sitemap's URLs were sorted by kind of page and stored.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SitemapConsolidated {
    pub consolidated_sitemap_id: Uuid,
    pub processed_sitemap_id: Uuid,
    pub retrieval_id: Uuid,
    pub retailer_code: RetailerCode,
    pub product_count: usize,
    pub catalog_count: usize,
    pub content_count: usize,
    pub not_interested_count: usize,
    pub unknown_count: usize,
}

impl Event for SitemapConsolidated {
    const NAME: &'static str = "SitemapConsolidated";
}
