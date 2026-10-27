use bodul_shared::retailer::RetailerCode;
use money::Money;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::platform::object_storage::{ObjectStorageMetadata, OfferStorageMetadata};

pub const SCHEMA_VERSION: u32 = 1;

/// The complete immutable input, independent of the latest pipeline metadata row.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExportSource {
    pub offer_id: Uuid,
    pub consolidated_sitemap_id: Uuid,
    pub retailer_code: RetailerCode,
    pub url: String,
    pub storage: OfferStorageMetadata,
}

/// Matches offers' ImportOfferDetails command; this is not its HTTP form DTO.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OfferImport {
    pub retailer_code: RetailerCode,
    pub title: String,
    pub price: Money,
    pub url: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExportEntry {
    /// Delivery identity, not the receiving service's offer identity.
    pub id: Uuid,
    pub variation_index: usize,
    pub payload: OfferImport,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExclusionReason {
    MissingPrice,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExportExclusion {
    pub variation_index: usize,
    pub url: String,
    pub reason: ExclusionReason,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OfferExport {
    pub id: Uuid,
    pub schema_version: u32,
    pub source: ExportSource,
    pub entries: Vec<ExportEntry>,
    pub exclusions: Vec<ExportExclusion>,
}

/// Pending bytes survive crashes; completed snapshots are loaded from object storage.
#[derive(Debug, Clone)]
pub(super) struct PreparedExport {
    pub snapshot: OfferExport,
    pub body: String,
    pub storage: Option<ObjectStorageMetadata>,
}
