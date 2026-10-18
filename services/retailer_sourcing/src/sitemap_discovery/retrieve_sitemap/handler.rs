use std::time::Duration;

use bodul_shared::retailer::RetailerCode;
use uuid::Uuid;

use super::super::Retailers;
use super::command::RetrieveSitemap;
use super::crawl::crawl;
use super::event::SitemapRetrieved;
use super::store::StoreError;
use crate::platform::bus::Handle;
use crate::platform::http::{self, FetchError};
use sitemap::RawSitemapDocument;

#[rustfmt::skip]
/// The storage this slice needs.
pub trait Store: Send + Sync + 'static {
    fn store_documents(&self, retrieval_id: Uuid, documents: &[RawSitemapDocument]) -> Result<(), StoreError>;
}

/// Fetches the body of one sitemap file.
pub trait FetchSitemapBody: Send + Sync + 'static {
    fn fetch(&self, url: &str) -> Result<String, FetchError>;
}

impl FetchSitemapBody for http::Client {
    fn fetch(&self, url: &str) -> Result<String, FetchError> {
        self.get(url)
    }
}

/// Looks up the sitemaps a retailer's crawl starts from.
pub trait RootSitemaps: Send + Sync + 'static {
    fn root_sitemaps(&self, retailer_code: &RetailerCode) -> Option<Vec<String>>;

    /// Whether the crawl downloads the child sitemap at `url`. Every one by
    /// default.
    fn sitemap_greylisted(&self, _retailer_code: RetailerCode, _url: &str) -> bool {
        true
    }

    /// How long the crawl pauses between groups of downloads. None by
    /// default.
    fn sitemap_pause(&self, _retailer_code: RetailerCode) -> Duration {
        Duration::ZERO
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

    fn sitemap_pause(&self, retailer_code: RetailerCode) -> Duration {
        Retailers::sitemap_pause(*self, retailer_code)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum RetrieveSitemapError {
    #[error("no sitemap configuration for retailer {0:?}")]
    NoSitemapConfig(RetailerCode),

    #[error("retrieval: {0}")]
    Store(#[from] StoreError),
}

pub struct Handler<S, F, R> {
    store: S,
    fetcher: F,
    retailers: R,
}

impl<S, F, R> Handler<S, F, R> {
    pub fn new(store: S, fetcher: F, retailers: R) -> Self {
        Self {
            store,
            fetcher,
            retailers,
        }
    }
}

impl<S: Store, F: FetchSitemapBody, R: RootSitemaps> Handle for Handler<S, F, R> {
    type Command = RetrieveSitemap;
    type Event = SitemapRetrieved;
    type Error = RetrieveSitemapError;

    fn handle(
        &self,
        command: RetrieveSitemap,
    ) -> Result<Vec<SitemapRetrieved>, RetrieveSitemapError> {
        let retrieval_id = command.retrieval_id();
        let retailer_code = command.retailer_code();

        let Some(root_urls) = self.retailers.root_sitemaps(&retailer_code) else {
            return Err(RetrieveSitemapError::NoSitemapConfig(retailer_code));
        };

        // A problem with one sitemap file skips that file; the crawl goes on.
        let greylisted = |url: &str| self.retailers.sitemap_greylisted(retailer_code, url);
        let pause = self.retailers.sitemap_pause(retailer_code);
        let (documents, not_greylisted, errors) =
            crawl(&self.fetcher, &root_urls, greylisted, pause);
        for url in &not_greylisted {
            tracing::info!(%retrieval_id, ?retailer_code, url, "sitemap file not greylisted, not downloaded");
        }
        for error in &errors {
            tracing::warn!(%retrieval_id, ?retailer_code, "sitemap file skipped: {error}");
        }

        self.store.store_documents(retrieval_id, &documents)?;
        tracing::info!(
            %retrieval_id,
            ?retailer_code,
            documents = documents.len(),
            not_greylisted = not_greylisted.len(),
            skipped = errors.len(),
            "sitemaps retrieved"
        );

        Ok(vec![SitemapRetrieved {
            retrieval_id,
            retailer_code,
            document_count: documents.len(),
        }])
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::sync::Mutex;

    use super::*;

    const ROOT: &str = "https://shop.test/sitemap.xml";
    const PRODUCTS: &str = "https://shop.test/sitemap_products.xml";
    const PAGES: &str = "https://shop.test/sitemap_pages.xml";

    #[derive(Default)]
    struct FakeStore {
        documents: Mutex<Vec<String>>,
    }

    impl Store for FakeStore {
        fn store_documents(
            &self,
            _: Uuid,
            documents: &[RawSitemapDocument],
        ) -> Result<(), StoreError> {
            let urls = documents.iter().map(|document| document.url.clone());
            self.documents.lock().unwrap().extend(urls);
            Ok(())
        }
    }

    /// Serves canned bodies and fails every other URL.
    struct Pages(HashMap<&'static str, String>);

    impl FetchSitemapBody for Pages {
        fn fetch(&self, url: &str) -> Result<String, FetchError> {
            self.0.get(url).cloned().ok_or_else(|| FetchError {
                message: "404 Not Found".to_string(),
            })
        }
    }

    struct Roots(Option<Vec<String>>);

    impl RootSitemaps for Roots {
        fn root_sitemaps(&self, _: &RetailerCode) -> Option<Vec<String>> {
            self.0.clone()
        }
    }

    fn index(children: &[&str]) -> String {
        let entries: String = children
            .iter()
            .map(|child| format!("<sitemap><loc>{child}</loc></sitemap>"))
            .collect();
        format!("<sitemapindex>{entries}</sitemapindex>")
    }

    fn urlset(url: &str) -> String {
        format!("<urlset><url><loc>{url}</loc></url></urlset>")
    }

    fn handler(pages: Pages, roots: Roots) -> Handler<FakeStore, Pages, Roots> {
        Handler::new(FakeStore::default(), pages, roots)
    }

    fn command() -> RetrieveSitemap {
        RetrieveSitemap::new(RetailerCode::UgreenCa)
    }

    #[test]
    fn retrieves_the_root_and_every_child_sitemap() {
        let pages = Pages(HashMap::from([
            (ROOT, index(&[PRODUCTS, PAGES])),
            (PRODUCTS, urlset("https://shop.test/products/a")),
            (PAGES, urlset("https://shop.test/pages/about")),
        ]));
        let handler = handler(pages, Roots(Some(vec![ROOT.to_string()])));

        let command = command();
        let retrieval_id = command.retrieval_id();

        let events = handler.handle(command).unwrap();

        assert_eq!(
            events,
            vec![SitemapRetrieved {
                retrieval_id,
                retailer_code: RetailerCode::UgreenCa,
                document_count: 3,
            }]
        );
        assert_eq!(
            *handler.store.documents.lock().unwrap(),
            vec![ROOT, PRODUCTS, PAGES]
        );
    }

    #[test]
    fn fetches_a_child_listed_twice_only_once() {
        let pages = Pages(HashMap::from([
            (ROOT, index(&[PRODUCTS, PRODUCTS])),
            (PRODUCTS, urlset("https://shop.test/products/a")),
        ]));
        let handler = handler(pages, Roots(Some(vec![ROOT.to_string()])));

        let events = handler.handle(command()).unwrap();

        assert_eq!(events[0].document_count, 2);
    }

    #[test]
    fn fails_when_the_retailer_has_no_sitemaps() {
        let handler = handler(Pages(HashMap::new()), Roots(None));

        let error = handler.handle(command()).unwrap_err();

        assert!(matches!(
            error,
            RetrieveSitemapError::NoSitemapConfig(RetailerCode::UgreenCa)
        ));
        assert!(handler.store.documents.lock().unwrap().is_empty());
    }
}
