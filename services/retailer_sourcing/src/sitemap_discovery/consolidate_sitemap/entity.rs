//! Domain values used by consolidation, independent of PostgreSQL storage.

use bodul_shared::retailer::RetailerCode;
use sitemap::{GroupedLinks, SitemapDocument};
use uuid::Uuid;

/// A processed sitemap as this slice reads it.
#[derive(Debug)]
pub(super) struct ProcessedSitemap {
    pub object_storage_id: Uuid,
    pub id: Uuid,
    pub retrieval_id: Uuid,
    pub retailer_code: RetailerCode,
    pub document: SitemapDocument,
}

/// A processed sitemap's URLs grouped by page kind, with consistent counts.
#[derive(Debug, Clone)]
pub(super) struct ConsolidatedSitemap {
    pub id: Uuid,
    pub processed_sitemap_id: Uuid,
    pub retrieval_id: Uuid,
    pub retailer_code: RetailerCode,
    links: GroupedLinks,
    product_count: usize,
    catalog_count: usize,
    content_count: usize,
    not_interested_count: usize,
    unknown_count: usize,
}

impl ConsolidatedSitemap {
    pub fn new(processed: &ProcessedSitemap, links: GroupedLinks) -> Self {
        Self {
            id: Uuid::now_v7(),
            processed_sitemap_id: processed.id,
            retrieval_id: processed.retrieval_id,
            retailer_code: processed.retailer_code,
            product_count: links.product.len(),
            catalog_count: links.catalog.len(),
            content_count: links.content.len(),
            not_interested_count: links.not_interested.len(),
            unknown_count: links.unknown.len(),
            links,
        }
    }

    pub fn links(&self) -> &GroupedLinks {
        &self.links
    }
    pub fn product_count(&self) -> usize {
        self.product_count
    }
    pub fn catalog_count(&self) -> usize {
        self.catalog_count
    }
    pub fn content_count(&self) -> usize {
        self.content_count
    }
    pub fn not_interested_count(&self) -> usize {
        self.not_interested_count
    }
    pub fn unknown_count(&self) -> usize {
        self.unknown_count
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counts_all_groups_including_empty_groups() {
        let processed = ProcessedSitemap {
            object_storage_id: Uuid::now_v7(),
            id: Uuid::now_v7(),
            retrieval_id: Uuid::now_v7(),
            retailer_code: RetailerCode::UgreenCa,
            document: SitemapDocument::default(),
        };
        for counts in [[0, 0, 0, 0, 0], [1, 2, 3, 4, 5]] {
            let links = GroupedLinks {
                product: vec!["product".into(); counts[0]],
                catalog: vec!["catalog".into(); counts[1]],
                content: vec!["content".into(); counts[2]],
                not_interested: vec!["skip".into(); counts[3]],
                unknown: vec!["unknown".into(); counts[4]],
            };
            let entity = ConsolidatedSitemap::new(&processed, links.clone());
            assert_eq!(
                [
                    entity.product_count(),
                    entity.catalog_count(),
                    entity.content_count(),
                    entity.not_interested_count(),
                    entity.unknown_count()
                ],
                counts
            );
            assert_eq!(entity.links(), &links);
            assert_eq!(entity.processed_sitemap_id, processed.id);
            assert_eq!(entity.retrieval_id, processed.retrieval_id);
        }
    }
}
