//! Domain values used by sitemap processing.
use bodul_shared::retailer::RetailerCode;
use chrono::{DateTime, Utc};
use sitemap::{ROOT_SITEMAP_SOURCE, SitemapDocument};
use uuid::Uuid;

/// A raw document loaded through its local and object-storage identities.
#[derive(Debug, Clone)]
pub(super) struct RawSitemapDocument {
    pub id: Uuid,
    pub object_storage_id: Uuid,
    pub document: sitemap::RawSitemapDocument,
}

/// A retrieval's sitemap tree and its domain count.
#[derive(Debug, Clone)]
pub(super) struct ProcessedSitemap {
    pub id: Uuid,
    pub retrieval_id: Uuid,
    pub retailer_code: RetailerCode,
    document: SitemapDocument,
    url_count: usize,
    pub processed_at: DateTime<Utc>,
}

impl ProcessedSitemap {
    pub fn new(retrieval_id: Uuid, retailer_code: RetailerCode, document: SitemapDocument) -> Self {
        let url_count = document.all_urls(ROOT_SITEMAP_SOURCE).count();
        Self {
            id: Uuid::now_v7(),
            retrieval_id,
            retailer_code,
            document,
            url_count,
            processed_at: Utc::now(),
        }
    }

    pub fn document(&self) -> &SitemapDocument {
        &self.document
    }
    pub fn url_count(&self) -> usize {
        self.url_count
    }
}
