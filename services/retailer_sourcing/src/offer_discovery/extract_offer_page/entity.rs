//! Domain input and output for this feature slice.
use chrono::DateTime;
use chrono::Utc;

use serde_json::Value;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq)]
pub(super) struct RawOffer {
    pub object_storage_id: Uuid,
    pub offer_id: Uuid,
    pub body: String,
}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct ExtractedOffer {
    pub scraped_at: Option<DateTime<Utc>>,
    pub offer_id: Uuid,
    pub document: Value,
}
