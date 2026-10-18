use bodul_shared::retailer::RetailerCode;
use chrono::{DateTime, Utc};
use uuid::Uuid;

use super::super::Retailers;
use super::command::ProcessSitemap;
use super::event::SitemapProcessed;
use super::store::StoreError;
use crate::platform::bus::Handle;
use sitemap::{BuildError, ROOT_SITEMAP_SOURCE, RawSitemapDocument, SitemapDocument, build_tree};

/// The storage this slice needs.
pub trait Store: Send + Sync + 'static {
    fn raw_documents(&self, retrieval_id: Uuid) -> Result<Vec<RawSitemapDocument>, StoreError>;

    fn store_processed(&self, sitemap: &ProcessedSitemap) -> Result<(), StoreError>;
}

/// Looks up the sitemaps a retailer's tree starts from.
pub trait RootSitemaps: Send + Sync + 'static {
    fn root_sitemaps(&self, retailer_code: &RetailerCode) -> Option<Vec<String>>;

    /// Whether the crawl downloads the child sitemap at `url`. Every one by
    /// default.
    fn sitemap_greylisted(&self, _retailer_code: RetailerCode, _url: &str) -> bool {
        true
    }
}

impl RootSitemaps for Retailers {
    fn root_sitemaps(&self, retailer_code: &RetailerCode) -> Option<Vec<String>> {
        self.sitemap_config(retailer_code)
            .map(|config| config.sitemap_url)
    }

    fn sitemap_greylisted(&self, retailer_code: RetailerCode, url: &str) -> bool {
        Retailers::sitemap_greylisted(*self, retailer_code, url)
    }
}

/// A retrieval's sitemap tree, ready to store.
pub struct ProcessedSitemap {
    pub id: Uuid,
    pub retrieval_id: Uuid,
    pub retailer_code: RetailerCode,
    pub document: SitemapDocument,
    pub url_count: i32,
    pub processed_at: DateTime<Utc>,
}

#[derive(Debug, thiserror::Error)]
pub enum ProcessSitemapError {
    #[error("no sitemap configuration for retailer {0:?}")]
    NoSitemapConfig(RetailerCode),

    #[error("sitemap: {0}")]
    Build(#[from] BuildError),

    #[error("sitemap contains {count} URLs, which exceeds the supported maximum")]
    TooManyUrls { count: usize },

    #[error("processing: {0}")]
    Store(#[from] StoreError),
}

pub struct Handler<S, R> {
    store: S,
    retailers: R,
}

impl<S, R> Handler<S, R> {
    pub fn new(store: S, retailers: R) -> Self {
        Self { store, retailers }
    }
}

impl<S: Store, R: RootSitemaps> Handle for Handler<S, R> {
    type Command = ProcessSitemap;
    type Event = SitemapProcessed;
    type Error = ProcessSitemapError;

    fn handle(
        &self,
        command: ProcessSitemap,
    ) -> Result<Vec<SitemapProcessed>, ProcessSitemapError> {
        let ProcessSitemap {
            retrieval_id,
            retailer_code,
        } = command;
        let root_urls = self
            .retailers
            .root_sitemaps(&retailer_code)
            .ok_or(ProcessSitemapError::NoSitemapConfig(retailer_code))?;
        let raw_documents = self.store.raw_documents(retrieval_id)?;

        // A root sitemap that can't be built is left out of the tree.
        let greylisted = |url: &str| self.retailers.sitemap_greylisted(retailer_code, url);
        let (document, errors) = build_tree(&root_urls, &raw_documents, greylisted)?;
        for error in &errors {
            tracing::warn!(%retrieval_id, ?retailer_code, "sitemap left out of the tree: {error}");
        }

        let url_count = document.all_urls(ROOT_SITEMAP_SOURCE).count();
        let stored_url_count = i32::try_from(url_count)
            .map_err(|_| ProcessSitemapError::TooManyUrls { count: url_count })?;

        let processed = ProcessedSitemap {
            id: Uuid::now_v7(),
            retrieval_id,
            retailer_code,
            document,
            url_count: stored_url_count,
            processed_at: Utc::now(),
        };
        self.store.store_processed(&processed)?;
        tracing::info!(
            %retrieval_id,
            ?retailer_code,
            processed_sitemap_id = %processed.id,
            urls = url_count,
            "sitemap processed"
        );

        Ok(vec![SitemapProcessed {
            retrieval_id,
            processed_sitemap_id: processed.id,
            retailer_code,
            url_count,
        }])
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use super::*;

    const ROOT: &str = "https://shop.test/sitemap.xml";
    const PRODUCTS: &str = "https://shop.test/sitemap_products.xml";
    const PAGES: &str = "https://shop.test/sitemap_pages.xml";

    struct FakeStore {
        raw_documents: Vec<RawSitemapDocument>,
        processed: Mutex<Vec<(Uuid, usize)>>,
    }

    impl FakeStore {
        fn with(raw_documents: Vec<RawSitemapDocument>) -> Self {
            Self {
                raw_documents,
                processed: Mutex::new(Vec::new()),
            }
        }
    }

    impl Store for FakeStore {
        fn raw_documents(&self, _: Uuid) -> Result<Vec<RawSitemapDocument>, StoreError> {
            Ok(self.raw_documents.clone())
        }

        fn store_processed(&self, sitemap: &ProcessedSitemap) -> Result<(), StoreError> {
            let url_count = sitemap.document.all_urls(ROOT_SITEMAP_SOURCE).count();
            self.processed
                .lock()
                .unwrap()
                .push((sitemap.retrieval_id, url_count));
            Ok(())
        }
    }

    struct Roots(Option<Vec<String>>);

    impl RootSitemaps for Roots {
        fn root_sitemaps(&self, _: &RetailerCode) -> Option<Vec<String>> {
            self.0.clone()
        }
    }

    fn raw(url: &str, body: String) -> RawSitemapDocument {
        RawSitemapDocument {
            url: url.to_string(),
            last_modified: None,
            body_size: body.len(),
            body,
        }
    }

    fn urlset(urls: &[&str]) -> String {
        let entries: String = urls
            .iter()
            .map(|url| format!("<url><loc>{url}</loc></url>"))
            .collect();
        format!("<urlset>{entries}</urlset>")
    }

    fn index(children: &[&str]) -> String {
        let entries: String = children
            .iter()
            .map(|child| format!("<sitemap><loc>{child}</loc></sitemap>"))
            .collect();
        format!("<sitemapindex>{entries}</sitemapindex>")
    }

    #[test]
    fn builds_one_tree_from_the_stored_files() {
        let store = FakeStore::with(vec![
            raw(ROOT, index(&[PRODUCTS, PAGES])),
            raw(
                PRODUCTS,
                urlset(&[
                    "https://shop.test/products/a",
                    "https://shop.test/products/b",
                ]),
            ),
            raw(PAGES, urlset(&["https://shop.test/pages/about"])),
        ]);
        let handler = Handler::new(store, Roots(Some(vec![ROOT.to_string()])));
        let retrieval_id = Uuid::now_v7();

        let events = handler
            .handle(ProcessSitemap {
                retrieval_id,
                retailer_code: RetailerCode::UgreenCa,
            })
            .unwrap();

        assert_eq!(events.len(), 1);
        assert_eq!(events[0].retrieval_id, retrieval_id);
        assert_eq!(events[0].retailer_code, RetailerCode::UgreenCa);
        assert_eq!(events[0].url_count, 3);
        assert_eq!(
            *handler.store.processed.lock().unwrap(),
            vec![(retrieval_id, 3)]
        );
    }

    #[test]
    fn fails_when_the_retailer_has_no_sitemaps() {
        let handler = Handler::new(FakeStore::with(Vec::new()), Roots(None));

        let error = handler
            .handle(ProcessSitemap {
                retrieval_id: Uuid::nil(),
                retailer_code: RetailerCode::UgreenCa,
            })
            .unwrap_err();

        assert!(matches!(
            error,
            ProcessSitemapError::NoSitemapConfig(RetailerCode::UgreenCa)
        ));
        assert!(handler.store.processed.lock().unwrap().is_empty());
    }
}
