use std::time::Duration;

use bodul_shared::retailer::RetailerCode;

use super::super::Retailers;
use super::command::RetrieveSitemap;
use super::crawl::crawl;
use super::entity::{RetrievedSitemapDocument, StoredSitemapDocument};
use super::event::SitemapRetrieved;
use super::store::StoreError;
use crate::platform::bus::Handle;
use crate::platform::http::{self, FetchError};

const MIN_SITEMAP_DELAY_SECONDS: u64 = 1;
const MAX_SITEMAP_DELAY_SECONDS: u64 = 5;

fn sitemap_delay(rng: &mut fastrand::Rng) -> Duration {
    Duration::from_secs(rng.u64(MIN_SITEMAP_DELAY_SECONDS..=MAX_SITEMAP_DELAY_SECONDS))
}

/// The storage this slice needs.
pub trait Store: Send + Sync + 'static {
    fn store_documents(
        &self,
        documents: &[RetrievedSitemapDocument],
    ) -> Result<Vec<StoredSitemapDocument>, StoreError>;
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

/// Looks up the sitemaps a retailer's crawl starts from, and the child
/// sitemaps it downloads.
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
        let mut rng = fastrand::Rng::new();
        self.handle_with_pause(command, || std::thread::sleep(sitemap_delay(&mut rng)))
    }
}

impl<S: Store, F: FetchSitemapBody, R: RootSitemaps> Handler<S, F, R> {
    fn handle_with_pause(
        &self,
        command: RetrieveSitemap,
        pause: impl FnMut(),
    ) -> Result<Vec<SitemapRetrieved>, RetrieveSitemapError> {
        let retrieval_id = command.retrieval_id();
        let retailer_code = command.retailer_code();

        let Some(root_urls) = self.retailers.root_sitemaps(&retailer_code) else {
            return Err(RetrieveSitemapError::NoSitemapConfig(retailer_code));
        };

        // A problem with one sitemap file skips that file; the crawl goes on.
        let greylisted = |url: &str| self.retailers.sitemap_greylisted(retailer_code, url);
        let (documents, not_greylisted, errors) =
            crawl(&self.fetcher, &root_urls, greylisted, pause);
        for url in &not_greylisted {
            tracing::info!(%retrieval_id, ?retailer_code, url, "sitemap file not greylisted, not downloaded");
        }
        for error in &errors {
            tracing::warn!(%retrieval_id, ?retailer_code, "sitemap file skipped: {error}");
        }

        let documents = RetrievedSitemapDocument::from_documents(retrieval_id, documents);
        let documents = self.store.store_documents(&documents)?;
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
    #[test]
    fn random_delays_are_whole_seconds_and_include_both_bounds() {
        let mut rng = fastrand::Rng::with_seed(42);
        let seconds: std::collections::HashSet<_> = (0..1000)
            .map(|_| {
                let delay = super::sitemap_delay(&mut rng);
                assert_eq!(delay.subsec_nanos(), 0);
                assert!((1..=5).contains(&delay.as_secs()));
                delay.as_secs()
            })
            .collect();
        assert_eq!(seconds, (1..=5).collect());
    }

    use std::collections::HashMap;
    use std::sync::Mutex;

    use super::*;

    const ROOT: &str = "https://shop.test/sitemap.xml";
    const PRODUCTS: &str = "https://shop.test/sitemap_products.xml";
    const PAGES: &str = "https://shop.test/sitemap_pages.xml";

    #[derive(Default)]
    struct FakeStore {
        documents: Mutex<Vec<String>>,
        fail: bool,
    }

    impl Store for FakeStore {
        fn store_documents(
            &self,
            documents: &[RetrievedSitemapDocument],
        ) -> Result<Vec<StoredSitemapDocument>, StoreError> {
            if self.fail {
                return Err(StoreError::InvalidBodySize);
            }
            let urls = documents
                .iter()
                .map(|document| document.document.url.clone());
            self.documents.lock().unwrap().extend(urls);
            Ok(documents
                .iter()
                .cloned()
                .map(|sitemap| StoredSitemapDocument {
                    sitemap,
                    object_storage_id: uuid::Uuid::now_v7(),
                })
                .collect())
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

    /// Starts from [`ROOT`] and downloads every child sitemap except `.0`.
    struct AllBut(&'static str);

    impl RootSitemaps for AllBut {
        fn root_sitemaps(&self, _: &RetailerCode) -> Option<Vec<String>> {
            Some(vec![ROOT.to_string()])
        }

        fn sitemap_greylisted(&self, _: RetailerCode, url: &str) -> bool {
            url != self.0
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

        let events = handler.handle_with_pause(command, || {}).unwrap();

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

        let events = handler.handle_with_pause(command(), || {}).unwrap();

        assert_eq!(events[0].document_count, 2);
    }

    #[test]
    fn does_not_fetch_a_child_that_is_not_greylisted() {
        let pages = Pages(HashMap::from([
            (ROOT, index(&[PRODUCTS, PAGES])),
            (PRODUCTS, urlset("https://shop.test/products/a")),
            (PAGES, urlset("https://shop.test/pages/about")),
        ]));
        let handler = Handler::new(FakeStore::default(), pages, AllBut(PAGES));

        let events = handler.handle_with_pause(command(), || {}).unwrap();

        assert_eq!(events[0].document_count, 2);
        assert_eq!(
            *handler.store.documents.lock().unwrap(),
            vec![ROOT, PRODUCTS]
        );
    }

    #[test]
    fn fails_when_the_retailer_has_no_sitemaps() {
        let handler = handler(Pages(HashMap::new()), Roots(None));

        let error = handler.handle_with_pause(command(), || {}).unwrap_err();

        assert!(matches!(
            error,
            RetrieveSitemapError::NoSitemapConfig(RetailerCode::UgreenCa)
        ));
        assert!(handler.store.documents.lock().unwrap().is_empty());
    }
    #[test]
    fn storage_failure_does_not_publish_a_retrieved_event() {
        let mut handler = handler(
            Pages(HashMap::from([(ROOT, urlset("https://shop.test/a"))])),
            Roots(Some(vec![ROOT.into()])),
        );
        handler.store.fail = true;
        assert!(matches!(
            handler.handle_with_pause(command(), || {}),
            Err(RetrieveSitemapError::Store(_))
        ));
    }
}
