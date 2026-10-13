use bodul_shared::retailer::RetailerCode;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::platform::bus::Event;

/// A retailer's sitemap files were fetched and stored.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SitemapRetrieved {
    pub retrieval_id: Uuid,
    pub retailer_code: RetailerCode,
    pub document_count: usize,
}

impl Event for SitemapRetrieved {
    const NAME: &'static str = "SitemapRetrieved";
}
