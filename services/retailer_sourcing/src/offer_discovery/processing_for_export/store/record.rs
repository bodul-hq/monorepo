use diesel::sql_types::{Integer, Jsonb, Nullable, Text, Uuid as SqlUuid};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::super::entity::{ExportSource, OfferExport, PreparedExport, SCHEMA_VERSION};
use super::StoreError;
use crate::platform::db::Json;
use crate::platform::object_storage::ObjectStorageMetadata;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct ExportStats {
    pub entries: u64,
    pub exclusions: u64,
}

impl From<&OfferExport> for ExportStats {
    fn from(snapshot: &OfferExport) -> Self {
        Self {
            entries: snapshot.entries.len() as u64,
            exclusions: snapshot.exclusions.len() as u64,
        }
    }
}

#[derive(diesel::QueryableByName)]
pub(super) struct OfferExportPgRecord {
    #[diesel(sql_type = SqlUuid)]
    pub id: Uuid,
    #[diesel(sql_type = SqlUuid)]
    pub offer_id: Uuid,
    #[diesel(sql_type = SqlUuid)]
    pub source_object_id: Uuid,
    #[diesel(sql_type = Integer)]
    pub schema_version: i32,
    #[diesel(sql_type = Jsonb)]
    pub source: Json<ExportSource>,
    #[diesel(sql_type = Jsonb)]
    pub stats: Json<ExportStats>,
    #[diesel(sql_type = Nullable<Text>)]
    pub prepared_body: Option<String>,
    #[diesel(sql_type = Nullable<Jsonb>)]
    pub storage: Option<Json<ObjectStorageMetadata>>,
}

impl OfferExportPgRecord {
    pub fn into_prepared(self, body: String) -> Result<PreparedExport, StoreError> {
        let snapshot: OfferExport = serde_json::from_str(&body)?;
        if self.schema_version != SCHEMA_VERSION as i32
            || snapshot.schema_version != SCHEMA_VERSION
            || snapshot.id != self.id
            || snapshot.source != self.source.0
            || snapshot.source.offer_id != self.offer_id
            || snapshot.source.storage.object_storage_id != self.source_object_id
            || ExportStats::from(&snapshot) != self.stats.0
        {
            return Err(StoreError::InvalidRecord);
        }
        Ok(PreparedExport {
            snapshot,
            body,
            storage: self.storage.map(|storage| storage.0),
        })
    }
}
