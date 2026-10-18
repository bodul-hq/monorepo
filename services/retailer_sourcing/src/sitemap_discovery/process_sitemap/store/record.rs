//! PostgreSQL representations and explicit conversions for this slice.
use crate::platform::db::Json;
use crate::platform::object_storage::ObjectStorageMetadata;
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use uuid::Uuid;

use super::super::super::schema::{sitemaps_processed, sitemaps_raw};
use super::super::entity::{ProcessedSitemap, RawSitemapDocument};
use super::StoreError;

#[derive(Queryable, Selectable)]
#[diesel(table_name = sitemaps_raw)]
pub(super) struct RawSitemapDocumentPgRecord {
    pub id: Uuid,
    pub url: String,
    pub last_modified: Option<DateTime<Utc>>,
    pub storage: Json<ObjectStorageMetadata>,
}

impl RawSitemapDocumentPgRecord {
    pub fn into_entity(self, body: String) -> Result<RawSitemapDocument, StoreError> {
        let body_size = i32::try_from(self.storage.0.content_size)
            .map_err(|_| StoreError::InvalidContentSize)? as usize;
        Ok(RawSitemapDocument {
            id: self.id,
            object_storage_id: self.storage.0.object_storage_id,
            document: sitemap::RawSitemapDocument {
                url: self.url,
                last_modified: self.last_modified,
                body,
                body_size,
            },
        })
    }
}

#[derive(Insertable)]
#[diesel(table_name = sitemaps_processed)]
pub(super) struct ProcessedSitemapPgRecord {
    pub id: Uuid,
    pub retrieval_id: Uuid,
    pub retailer_code: String,
    pub url_count: i32,
    pub storage: Json<ObjectStorageMetadata>,
    pub processed_at: DateTime<Utc>,
}

pub(super) fn validate(entity: &ProcessedSitemap, body: &str) -> Result<(), StoreError> {
    i32::try_from(body.len()).map_err(|_| StoreError::DocumentTooLarge)?;
    stored_count(entity.url_count())?;
    Ok(())
}

impl ProcessedSitemapPgRecord {
    pub fn new(
        entity: &ProcessedSitemap,
        storage: ObjectStorageMetadata,
    ) -> Result<Self, StoreError> {
        Ok(Self {
            id: entity.id,
            retrieval_id: entity.retrieval_id,
            retailer_code: entity.retailer_code.as_str(),
            url_count: stored_count(entity.url_count())?,
            storage: Json(storage),
            processed_at: entity.processed_at,
        })
    }
}

fn stored_count(count: usize) -> Result<i32, StoreError> {
    i32::try_from(count).map_err(|_| StoreError::TooManyUrls(count))
}

#[cfg(test)]
mod tests {
    use super::*;
    use bodul_shared::retailer::RetailerCode;
    use sitemap::SitemapDocument;

    #[test]
    fn maps_domain_tree_and_timestamp_without_changing_ids() {
        let entity = ProcessedSitemap::new(
            Uuid::now_v7(),
            RetailerCode::UgreenCa,
            SitemapDocument::default(),
        );
        let body = serde_json::to_string(entity.document()).unwrap();
        validate(&entity, &body).unwrap();
        let record = ProcessedSitemapPgRecord::new(
            &entity,
            ObjectStorageMetadata {
                content_size: body.len() as u64,
                object_storage_id: Uuid::now_v7(),
            },
        )
        .unwrap();
        assert_eq!(record.id, entity.id);
        assert_eq!(record.retrieval_id, entity.retrieval_id);
        assert_eq!(record.retailer_code, "ugreenca");
        assert_eq!(record.processed_at, entity.processed_at);
        assert_eq!(record.url_count, 0);
        assert_eq!(record.storage.0.content_size, body.len() as u64);
        assert_eq!(stored_count(i32::MAX as usize).unwrap(), i32::MAX);
        assert!(matches!(
            stored_count(i32::MAX as usize + 1),
            Err(StoreError::TooManyUrls(_))
        ));
    }

    #[test]
    fn reads_raw_entity_and_rejects_sizes_above_the_storage_limit() {
        let modified = Utc::now();
        for size in [0, 7, i32::MAX as u64, i32::MAX as u64 + 1] {
            let record = RawSitemapDocumentPgRecord {
                id: Uuid::now_v7(),
                url: "https://shop.test/ž".into(),
                last_modified: Some(modified),
                storage: Json(ObjectStorageMetadata {
                    content_size: size,
                    object_storage_id: Uuid::now_v7(),
                }),
            };
            let id = record.id;
            let object_storage_id = record.storage.0.object_storage_id;
            let result = record.into_entity("ž 🔎".into());
            if size > i32::MAX as u64 {
                assert!(matches!(result, Err(StoreError::InvalidContentSize)));
            } else {
                let entity = result.unwrap();
                assert_eq!(entity.id, id);
                assert_eq!(entity.object_storage_id, object_storage_id);
                assert_eq!(entity.document.body_size, size as usize);
                assert_eq!(entity.document.body, "ž 🔎");
                assert_eq!(entity.document.url, "https://shop.test/ž");
                assert_eq!(entity.document.last_modified, Some(modified));
            }
        }
    }
}
