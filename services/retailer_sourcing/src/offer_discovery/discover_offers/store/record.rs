//! PostgreSQL projection and conversion for discovery's sitemap input.
use super::super::entity::ConsolidatedSitemap;
use super::StoreError;
use diesel::prelude::*;
use serde::Deserialize;
use uuid::Uuid;

// The columns this slice reads from sitemap discovery's table. A consolidated
// sitemap's content is keyed by the sitemap's id.
diesel::table! {
    sitemaps_consolidated (id) {
        id -> Uuid,
        storage -> Jsonb,
    }
}

#[derive(Queryable, Selectable)]
#[diesel(table_name = sitemaps_consolidated)]
pub(super) struct ConsolidatedSitemapPgRecord {
    pub id: Uuid,
    pub storage: crate::platform::db::Json<crate::platform::object_storage::ObjectStorageMetadata>,
}

#[derive(Deserialize)]
struct ConsolidatedContent {
    product: Vec<String>,
}

impl ConsolidatedSitemapPgRecord {
    pub fn into_entity(self, body: &str) -> Result<ConsolidatedSitemap, StoreError> {
        let content: ConsolidatedContent = serde_json::from_str(body)?;
        Ok(ConsolidatedSitemap {
            id: self.id,
            object_storage_id: self.storage.0.object_storage_id,
            product_urls: content.product,
        })
    }
}
