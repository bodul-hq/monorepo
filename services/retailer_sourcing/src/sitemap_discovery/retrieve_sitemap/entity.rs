//! Retrieved documents with their domain identity, independent of persistence.
use chrono::{DateTime, Utc};
use sitemap::RawSitemapDocument;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub(super) struct RetrievedSitemapDocument {
    pub id: Uuid,
    pub retrieval_id: Uuid,
    pub fetched_at: DateTime<Utc>,
    pub document: RawSitemapDocument,
}

impl RetrievedSitemapDocument {
    /// All documents in a completed crawl share its retrieval time.
    pub fn from_documents(retrieval_id: Uuid, documents: Vec<RawSitemapDocument>) -> Vec<Self> {
        let fetched_at = Utc::now();
        documents
            .into_iter()
            .map(|document| Self {
                id: Uuid::now_v7(),
                retrieval_id,
                fetched_at,
                document,
            })
            .collect()
    }
}

/// A document whose body upload and local metadata insert both succeeded.
#[derive(Debug, Clone)]
pub(super) struct StoredSitemapDocument {
    pub sitemap: RetrievedSitemapDocument,
    pub object_storage_id: Uuid,
}
