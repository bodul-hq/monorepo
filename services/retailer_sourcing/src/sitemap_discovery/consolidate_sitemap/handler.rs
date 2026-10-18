use bodul_shared::link::LinkKind;
use bodul_shared::retailer::RetailerCode;
use uuid::Uuid;

use super::super::Retailers;
use super::command::ConsolidateSitemap;
use super::event::SitemapConsolidated;
use super::store::StoreError;
use crate::platform::bus::Handle;
use sitemap::{GroupedLinks, SitemapDocument, SitemapUrl};

/// The storage this slice needs.
pub trait Store: Send + Sync + 'static {
    fn processed_sitemap(&self, id: Uuid) -> Result<ProcessedSitemap, StoreError>;

    fn store_consolidated(&self, consolidated: &ConsolidatedSitemap) -> Result<(), StoreError>;
}

/// Decides what kind of page a URL points to.
pub trait ClassifyLink: Send + Sync + 'static {
    fn classify(&self, retailer_code: RetailerCode, url: &SitemapUrl) -> LinkKind;
}

impl ClassifyLink for Retailers {
    fn classify(&self, retailer_code: RetailerCode, url: &SitemapUrl) -> LinkKind {
        self.classify_link(retailer_code, &url.location, &url.source, url.images.len())
    }
}

/// A processed sitemap as this slice reads it.
pub struct ProcessedSitemap {
    pub id: Uuid,
    pub retrieval_id: Uuid,
    pub retailer_code: RetailerCode,
    pub document: SitemapDocument,
}

/// A processed sitemap's URLs sorted by kind, ready to store.
pub struct ConsolidatedSitemap {
    pub id: Uuid,
    pub processed_sitemap_id: Uuid,
    pub retrieval_id: Uuid,
    pub retailer_code: RetailerCode,
    pub links: GroupedLinks,
}

pub struct Handler<S, C> {
    store: S,
    classifier: C,
}

impl<S, C> Handler<S, C> {
    pub fn new(store: S, classifier: C) -> Self {
        Self { store, classifier }
    }
}

impl<S: Store, C: ClassifyLink> Handle for Handler<S, C> {
    type Command = ConsolidateSitemap;
    type Event = SitemapConsolidated;
    type Error = StoreError;

    fn handle(&self, command: ConsolidateSitemap) -> Result<Vec<SitemapConsolidated>, StoreError> {
        let processed = self.store.processed_sitemap(command.processed_sitemap_id)?;
        let links = GroupedLinks::from_document(&processed.document, |url| {
            self.classifier.classify(processed.retailer_code, url)
        });
        let consolidated = ConsolidatedSitemap {
            id: Uuid::now_v7(),
            processed_sitemap_id: processed.id,
            retrieval_id: processed.retrieval_id,
            retailer_code: processed.retailer_code,
            links,
        };

        self.store.store_consolidated(&consolidated)?;

        let links = &consolidated.links;
        tracing::info!(
            retrieval_id = %consolidated.retrieval_id,
            retailer_code = ?consolidated.retailer_code,
            consolidated_sitemap_id = %consolidated.id,
            products = links.product.len(),
            catalogs = links.catalog.len(),
            content = links.content.len(),
            not_interested = links.not_interested.len(),
            unknown = links.unknown.len(),
            "sitemap consolidated"
        );
        Ok(vec![SitemapConsolidated {
            consolidated_sitemap_id: consolidated.id,
            processed_sitemap_id: consolidated.processed_sitemap_id,
            retrieval_id: consolidated.retrieval_id,
            retailer_code: consolidated.retailer_code,
            product_count: links.product.len(),
            catalog_count: links.catalog.len(),
            content_count: links.content.len(),
            not_interested_count: links.not_interested.len(),
            unknown_count: links.unknown.len(),
        }])
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use super::*;

    struct FakeStore {
        document: SitemapDocument,
        consolidated: Mutex<Vec<GroupedLinks>>,
    }

    impl Store for FakeStore {
        fn processed_sitemap(&self, id: Uuid) -> Result<ProcessedSitemap, StoreError> {
            Ok(ProcessedSitemap {
                id,
                retrieval_id: Uuid::nil(),
                retailer_code: RetailerCode::UgreenCa,
                document: self.document.clone(),
            })
        }

        fn store_consolidated(&self, consolidated: &ConsolidatedSitemap) -> Result<(), StoreError> {
            self.consolidated
                .lock()
                .unwrap()
                .push(consolidated.links.clone());
            Ok(())
        }
    }

    /// Products live under `/products/`, and nothing else is known.
    struct ByPath;

    impl ClassifyLink for ByPath {
        fn classify(&self, _: RetailerCode, url: &SitemapUrl) -> LinkKind {
            if url.location.contains("/products/") {
                LinkKind::Product
            } else {
                LinkKind::Unknown
            }
        }
    }

    fn url(location: &str) -> SitemapUrl {
        SitemapUrl {
            location: location.to_string(),
            source: String::new(),
            last_modified: None,
            change_frequency: None,
            priority: None,
            images: Vec::new(),
        }
    }

    #[test]
    fn consolidates_every_url_by_kind() {
        let store = FakeStore {
            document: SitemapDocument {
                urls: vec![
                    url("https://shop.test/products/a"),
                    url("https://shop.test/products/b"),
                    url("https://shop.test/pages/about"),
                ],
                ..SitemapDocument::default()
            },
            consolidated: Mutex::new(Vec::new()),
        };
        let handler = Handler::new(store, ByPath);
        let processed_sitemap_id = Uuid::now_v7();

        let events = handler
            .handle(ConsolidateSitemap {
                processed_sitemap_id,
            })
            .unwrap();

        assert_eq!(events.len(), 1);
        let event = &events[0];
        assert_eq!(event.processed_sitemap_id, processed_sitemap_id);
        assert_eq!(event.retailer_code, RetailerCode::UgreenCa);
        assert_eq!(
            (
                event.product_count,
                event.unknown_count,
                event.catalog_count
            ),
            (2, 1, 0)
        );
        assert_eq!(
            handler.store.consolidated.lock().unwrap()[0].unknown,
            vec!["https://shop.test/pages/about"]
        );
    }
}
