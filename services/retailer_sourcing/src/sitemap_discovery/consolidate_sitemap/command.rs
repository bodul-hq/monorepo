use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::platform::bus::Command;

/// Sorts the URLs of a processed sitemap by kind of page.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConsolidateSitemap {
    pub processed_sitemap_id: Uuid,
}

impl Command for ConsolidateSitemap {
    const NAME: &'static str = "ConsolidateSitemap";
}
