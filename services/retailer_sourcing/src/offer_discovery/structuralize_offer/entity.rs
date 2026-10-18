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
    pub offer_id: Uuid,
    pub document: Value,
}
