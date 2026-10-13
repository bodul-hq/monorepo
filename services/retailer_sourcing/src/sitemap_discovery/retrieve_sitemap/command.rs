use bodul_shared::retailer::RetailerCode;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::platform::bus::Command;

/// Fetches a retailer's sitemaps and stores them under `retrieval_id`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetrieveSitemap {
    retrieval_id: Uuid,
    retailer_code: RetailerCode,
}

impl RetrieveSitemap {
    /// Starts a new retrieval for `retailer_code`, with a fresh retrieval id.
    pub fn new(retailer_code: RetailerCode) -> Self {
        Self {
            retrieval_id: Uuid::now_v7(),
            retailer_code,
        }
    }

    pub fn retrieval_id(&self) -> Uuid {
        self.retrieval_id
    }

    pub fn retailer_code(&self) -> RetailerCode {
        self.retailer_code
    }
}

impl Command for RetrieveSitemap {
    const NAME: &'static str = "RetrieveSitemap";
}
