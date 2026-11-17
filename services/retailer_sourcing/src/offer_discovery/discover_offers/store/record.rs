//! PostgreSQL projection and conversion for discovery's sitemap input.

use super::super::entity::ConsolidatedSitemap;
use super::StoreError;
use crate::platform::db::Json;
use crate::platform::object_storage::ObjectStorageMetadata;
use diesel::prelude::*;
use diesel::table as diesel_table;
use serde::Deserialize;
use serde_json::from_str;
use uuid::Uuid;

// The columns this slice reads from sitemap discovery's table. A consolidated
// sitemap's content is keyed by the sitemap's id.
diesel_table! {
    sitemaps_consolidated (id) {
        id -> Uuid,
        storage -> Jsonb,
    }
}

#[derive(Queryable, Selectable)]
#[diesel(table_name = sitemaps_consolidated)]
pub(super) struct ConsolidatedSitemapPgRecord {
    pub id: Uuid,
    pub storage: Json<ObjectStorageMetadata>,
}

#[derive(Deserialize)]
struct ConsolidatedContent {
    product: Vec<String>,
}

impl ConsolidatedSitemapPgRecord {
    pub fn into_entity(self, body: &str) -> Result<ConsolidatedSitemap, StoreError> {
        let content: ConsolidatedContent = from_str(body)?;
        Ok(ConsolidatedSitemap {
            id: self.id,
            object_storage_id: self.storage.0.object_storage_id,
            product_urls: content.product,
        })
    }
}
