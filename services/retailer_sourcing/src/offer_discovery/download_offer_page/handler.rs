use chrono::{DateTime, Duration, Utc};
use uuid::Uuid;

use super::command::DownloadOfferPage;
use super::event::{DownloadOutcome, OfferPageDownloadSkipped, OfferPageWasDownloaded};
use super::store::StoreError;
use crate::platform::bus::Handle;
use crate::platform::http::{self, FetchError};

/// How recently a URL must have been downloaded for it not to be fetched again.
const REDOWNLOAD_WINDOW_HOURS: i64 = 6;

/// The storage this slice needs.
pub trait Store: Send + Sync + 'static {
    /// The most recent page stored for `url` since `since`, if any.
    fn recent_raw_offer(&self, url: &str, since: DateTime<Utc>)
    -> Result<Option<Uuid>, StoreError>;

    /// Stores the page, replacing any page stored for the offer before.
    fn store_raw_offer(&self, offer_id: Uuid, url: &str, body: &str) -> Result<(), StoreError>;
}

/// Fetches the body of one offer page.
pub trait FetchOfferPage: Send + Sync + 'static {
    fn fetch(&self, url: &str) -> Result<String, FetchError>;
}

impl FetchOfferPage for http::Client {
    fn fetch(&self, url: &str) -> Result<String, FetchError> {
        self.get(url)
    }
}

pub struct Handler<S, F> {
    store: S,
    fetcher: F,
}

impl<S, F> Handler<S, F> {
    pub fn new(store: S, fetcher: F) -> Self {
        Self { store, fetcher }
    }
}

impl<S: Store, F: FetchOfferPage> Handle for Handler<S, F> {
    type Command = DownloadOfferPage;
    type Event = DownloadOutcome;
    type Error = StoreError;

    fn handle(&self, command: DownloadOfferPage) -> Result<Vec<DownloadOutcome>, StoreError> {
        let since = Utc::now() - Duration::hours(REDOWNLOAD_WINDOW_HOURS);
        if let Some(raw_offer_id) = self.store.recent_raw_offer(&command.url, since)? {
            tracing::info!(
                offer_id = %command.offer_id,
                url = %command.url,
                "offer page skipped: downloaded in the last {REDOWNLOAD_WINDOW_HOURS} hours"
            );
            return Ok(vec![DownloadOutcome::Skipped(OfferPageDownloadSkipped {
                offer_id: command.offer_id,
                consolidated_sitemap_id: command.consolidated_sitemap_id,
                retailer_code: command.retailer_code,
                url: command.url,
                raw_offer_id,
            })]);
        }

        let body = match self.fetcher.fetch(&command.url) {
            Ok(body) => body,
            Err(error) => {
                // The failure is logged and the command is done, not retried.
                tracing::warn!(
                    offer_id = %command.offer_id,
                    url = %command.url,
                    "offer page download failed: {error}"
                );
                return Ok(Vec::new());
            }
        };

        self.store
            .store_raw_offer(command.offer_id, &command.url, &body)?;
        tracing::info!(
            offer_id = %command.offer_id,
            url = %command.url,
            bytes = body.len(),
            "offer page downloaded"
        );

        Ok(vec![DownloadOutcome::Downloaded(OfferPageWasDownloaded {
            offer_id: command.offer_id,
            consolidated_sitemap_id: command.consolidated_sitemap_id,
            retailer_code: command.retailer_code,
            url: command.url,
        })])
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use bodul_shared::retailer::RetailerCode;

    use super::*;

    const URL: &str = "https://shop.test/products/a";

    #[derive(Default)]
    struct FakeStore {
        recent: Option<Uuid>,
        log: Mutex<Vec<String>>,
    }

    impl FakeStore {
        fn log(&self, entry: String) -> Result<(), StoreError> {
            self.log.lock().unwrap().push(entry);
            Ok(())
        }
    }

    impl Store for FakeStore {
        fn recent_raw_offer(&self, _: &str, _: DateTime<Utc>) -> Result<Option<Uuid>, StoreError> {
            Ok(self.recent)
        }

        fn store_raw_offer(&self, _: Uuid, url: &str, body: &str) -> Result<(), StoreError> {
            self.log(format!("stored {url}: {body}"))
        }
    }

    /// Serves one page, or fails every request.
    struct Page(Option<&'static str>);

    impl FetchOfferPage for Page {
        fn fetch(&self, _: &str) -> Result<String, FetchError> {
            self.0.map(str::to_string).ok_or_else(|| FetchError {
                message: "403 Forbidden".to_string(),
            })
        }
    }

    fn command() -> DownloadOfferPage {
        DownloadOfferPage {
            offer_id: Uuid::now_v7(),
            consolidated_sitemap_id: Uuid::now_v7(),
            retailer_code: RetailerCode::UgreenCa,
            url: URL.to_string(),
        }
    }

    #[test]
    fn downloads_and_stores_the_page() {
        let handler = Handler::new(FakeStore::default(), Page(Some("<html>offer</html>")));
        let command = command();

        let events = handler.handle(command.clone()).unwrap();

        assert_eq!(
            events,
            vec![DownloadOutcome::Downloaded(OfferPageWasDownloaded {
                offer_id: command.offer_id,
                consolidated_sitemap_id: command.consolidated_sitemap_id,
                retailer_code: RetailerCode::UgreenCa,
                url: URL.to_string(),
            })]
        );
        assert_eq!(
            *handler.store.log.lock().unwrap(),
            vec![format!("stored {URL}: <html>offer</html>")]
        );
    }

    #[test]
    fn skips_a_url_downloaded_recently() {
        let raw_offer_id = Uuid::now_v7();
        let store = FakeStore {
            recent: Some(raw_offer_id),
            ..FakeStore::default()
        };
        let handler = Handler::new(store, Page(Some("unused")));

        let events = handler.handle(command()).unwrap();

        assert!(matches!(
            &events[..],
            [DownloadOutcome::Skipped(skipped)] if skipped.raw_offer_id == raw_offer_id
        ));
        assert!(handler.store.log.lock().unwrap().is_empty());
    }

    #[test]
    fn completes_the_command_when_the_download_fails() {
        let handler = Handler::new(FakeStore::default(), Page(None));

        let events = handler.handle(command()).unwrap();

        assert!(events.is_empty());
        assert!(handler.store.log.lock().unwrap().is_empty());
    }
}
