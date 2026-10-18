//! Local metadata for a body stored in object storage.
use crate::platform::db::Json;
use crate::platform::object_storage::ObjectStorageMetadata;
use chrono::{DateTime, Utc};
use diesel::Insertable;
use uuid::Uuid;

use super::super::super::schema::sitemaps_raw;
use super::super::entity::{RetrievedSitemapDocument, StoredSitemapDocument};
use super::StoreError;

#[derive(Insertable)]
#[diesel(table_name = sitemaps_raw)]
pub(super) struct RawSitemapDocumentPgRecord<'a> {
    pub id: Uuid,
    pub retrieval_id: Uuid,
    pub url: &'a str,
    pub last_modified: Option<DateTime<Utc>>,
    pub storage: Json<ObjectStorageMetadata>,
    pub fetched_at: DateTime<Utc>,
}

pub(super) fn validate(document: &RetrievedSitemapDocument) -> Result<(), StoreError> {
    let raw = &document.document;
    i32::try_from(raw.body_size).map_err(|_| StoreError::BodyTooLarge(raw.body_size))?;
    if raw.body_size != raw.body.len() {
        return Err(StoreError::InvalidBodySize);
    }
    Ok(())
}

impl<'a> From<&'a StoredSitemapDocument> for RawSitemapDocumentPgRecord<'a> {
    fn from(stored: &'a StoredSitemapDocument) -> Self {
        let entity = &stored.sitemap;
        Self {
            id: entity.id,
            retrieval_id: entity.retrieval_id,
            url: &entity.document.url,
            last_modified: entity.document.last_modified,
            storage: Json(ObjectStorageMetadata {
                content_size: entity.document.body.len() as u64,
                object_storage_id: stored.object_storage_id,
            }),
            fetched_at: entity.fetched_at,
        }
    }
}
