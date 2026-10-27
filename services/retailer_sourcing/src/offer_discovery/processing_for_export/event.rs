use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::entity::ExportSource;
use crate::platform::bus::Event;

/// A completed immutable snapshot, including zero-entry exports.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OfferExportWasPrepared {
    pub export_id: Uuid,
    pub schema_version: u32,
    pub source: ExportSource,
}

impl Event for OfferExportWasPrepared {
    const NAME: &'static str = "OfferExportWasPrepared";
}
