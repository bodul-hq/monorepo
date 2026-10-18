use crate::platform::object_storage::Stored;
use bodul_shared::link::LinkKind;
use bodul_shared::retailer::RetailerCode;
use sitemap::{GroupedLinks, SitemapUrl};
use uuid::Uuid;

use super::super::Retailers;
use super::command::ConsolidateSitemap;
use super::entity::{ConsolidatedSitemap, ProcessedSitemap};
use super::event::SitemapConsolidated;
use super::store::StoreError;
use crate::platform::bus::Handle;

/// The storage this slice needs.
pub trait Store: Send + Sync + 'static {
    fn processed_sitemap(&self, id: Uuid) -> Result<ProcessedSitemap, StoreError>;

    fn store_consolidated(
        &self,
        consolidated: &ConsolidatedSitemap,
    ) -> Result<Stored<ConsolidatedSitemap>, StoreError>;
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
        tracing::debug!(object_storage_id = %processed.object_storage_id, "input object loaded");
        let links = GroupedLinks::from_document(&processed.document, |url| {
            self.classifier.classify(processed.retailer_code, url)
        });
        let consolidated = ConsolidatedSitemap::new(&processed, links);

        let stored = self.store.store_consolidated(&consolidated)?;

        tracing::info!(
            object_storage_id = %stored.object_storage_id,
            retrieval_id = %consolidated.retrieval_id,
            retailer_code = ?consolidated.retailer_code,
            consolidated_sitemap_id = %consolidated.id,
            products = consolidated.product_count(),
            catalogs = consolidated.catalog_count(),
            content = consolidated.content_count(),
            not_interested = consolidated.not_interested_count(),
            unknown = consolidated.unknown_count(),
            "sitemap consolidated"
        );
        Ok(vec![SitemapConsolidated {
            consolidated_sitemap_id: consolidated.id,
            processed_sitemap_id: consolidated.processed_sitemap_id,
            retrieval_id: consolidated.retrieval_id,
            retailer_code: consolidated.retailer_code,
            product_count: consolidated.product_count(),
            catalog_count: consolidated.catalog_count(),
            content_count: consolidated.content_count(),
            not_interested_count: consolidated.not_interested_count(),
            unknown_count: consolidated.unknown_count(),
        }])
    }
}

#[cfg(test)]
mod tests {
    use sitemap::SitemapDocument;
    use std::sync::Mutex;

    use super::*;

    struct FakeStore {
        fail: bool,
        document: SitemapDocument,
        consolidated: Mutex<Vec<ConsolidatedSitemap>>,
    }

    impl Store for FakeStore {
        fn processed_sitemap(&self, id: Uuid) -> Result<ProcessedSitemap, StoreError> {
            Ok(ProcessedSitemap {
                object_storage_id: uuid::Uuid::now_v7(),
                id,
                retrieval_id: Uuid::nil(),
                retailer_code: RetailerCode::UgreenCa,
                document: self.document.clone(),
            })
        }

        fn store_consolidated(
            &self,
            consolidated: &ConsolidatedSitemap,
        ) -> Result<Stored<ConsolidatedSitemap>, StoreError> {
            if self.fail {
                return Err(StoreError::ObjectStorage(
                    crate::platform::object_storage::Error::InvalidRecord,
                ));
            }

            self.consolidated.lock().unwrap().push(consolidated.clone());
            Ok(Stored {
                entity: consolidated.clone(),
                object_storage_id: uuid::Uuid::now_v7(),
            })
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
            fail: false,
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

        let saved = handler.store.consolidated.lock().unwrap()[0].clone();
        assert_eq!(events[0].product_count, saved.product_count());
        assert_eq!(events[0].catalog_count, saved.catalog_count());
        assert_eq!(events[0].content_count, saved.content_count());
        assert_eq!(events[0].not_interested_count, saved.not_interested_count());
        assert_eq!(events[0].unknown_count, saved.unknown_count());
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
            handler.store.consolidated.lock().unwrap()[0]
                .links()
                .unknown,
            vec!["https://shop.test/pages/about"]
        );
    }

    #[test]
    fn storage_failure_prevents_success_events() {
        let store = FakeStore {
            fail: true,
            document: SitemapDocument::default(),
            consolidated: Mutex::new(Vec::new()),
        };
        let handler = Handler::new(store, ByPath);
        assert!(
            handler
                .handle(ConsolidateSitemap {
                    processed_sitemap_id: Uuid::now_v7()
                })
                .is_err()
        );
    }
}
