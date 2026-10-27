//! Domain input and output for this feature slice.
use serde_json::Value;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq)]
pub(super) struct NormalizedOffer {
    pub object_storage_id: Uuid,
    pub offer_id: Uuid,
    pub document: Value,
}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct StructuralizedOffer {
    pub scraped_at: Option<chrono::DateTime<chrono::Utc>>,
    pub offer_id: Uuid,
    pub document: Value,
}
