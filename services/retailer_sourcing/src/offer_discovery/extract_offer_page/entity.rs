//! Domain input and output for this feature slice.
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
    pub offer_id: Uuid,
    pub document: Value,
}
