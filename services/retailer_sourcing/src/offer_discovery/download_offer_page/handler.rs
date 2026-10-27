use std::time::Duration;

use super::command::DownloadOfferPage;
use super::entity::RawOffer;
use super::event::OfferPageWasDownloaded;
use super::store::StoreError;
use crate::platform::bus::Handle;
use crate::platform::http::{self, FetchError};

const DOWNLOAD_OFFER_PAGE_DELAY: Duration = Duration::from_secs(6);

/// The storage this slice needs.
pub trait Store: Send + Sync + 'static {
    /// Stores the page, replacing any page stored for the offer before.
    fn store_raw_offer(
        &self,
        entity: &RawOffer,
    ) -> Result<crate::platform::object_storage::OfferStorageMetadata, StoreError>;
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

#[derive(Debug, thiserror::Error)]
pub enum DownloadOfferPageError {
    #[error("offer page download failed for {url}: {error}")]
    Fetch { url: String, error: FetchError },

    #[error(transparent)]
    Store(#[from] StoreError),
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
    type Event = OfferPageWasDownloaded;
    type Error = DownloadOfferPageError;

    fn handle(
        &self,
        command: DownloadOfferPage,
    ) -> Result<Vec<OfferPageWasDownloaded>, DownloadOfferPageError> {
        self.handle_with_sleep(command, std::thread::sleep)
    }
}

impl<S: Store, F: FetchOfferPage> Handler<S, F> {
    fn handle_with_sleep(
        &self,
        command: DownloadOfferPage,
        sleep: impl FnOnce(Duration),
    ) -> Result<Vec<OfferPageWasDownloaded>, DownloadOfferPageError> {
        let download_url =
            retailer_data_extractor::offer_download_url(command.retailer_code, &command.url);
        // Pacing is per handler invocation; concurrent workers may still fetch together.
        sleep(DOWNLOAD_OFFER_PAGE_DELAY);
        // A failed download fails the command, so it is retried later.
        let body =
            self.fetcher
                .fetch(download_url)
                .map_err(|error| DownloadOfferPageError::Fetch {
                    url: download_url.to_owned(),
                    error,
                })?;

        let scraped_at =
            chrono::DateTime::from_timestamp_micros(chrono::Utc::now().timestamp_micros());
        let entity = RawOffer {
            scraped_at,
            offer_id: command.offer_id,
            retailer_code: command.retailer_code,
            url: command.url.clone(),
            body,
        };
        let stored = self.store.store_raw_offer(&entity)?;
        tracing::info!(
            object_storage_id = %stored.object_storage_id,
            offer_id = %command.offer_id,
            url = %command.url,
            bytes = entity.body.len(),
            "offer page downloaded"
        );

        Ok(vec![OfferPageWasDownloaded {
            storage: Some(stored),
            scraped_at,
            offer_id: command.offer_id,
            consolidated_sitemap_id: command.consolidated_sitemap_id,
            retailer_code: command.retailer_code,
            url: command.url,
        }])
    }
}

#[cfg(test)]
mod tests {
    use bodul_shared::retailer::RetailerCode;
    use std::sync::Mutex;
    use uuid::Uuid;

    use super::*;

    const URL: &str = "https://shop.test/products/a";

    #[derive(Default)]
    struct FakeStore {
        fail: bool,
        log: Mutex<Vec<String>>,
    }

    impl FakeStore {
        fn log(&self, entry: String) -> Result<(), StoreError> {
            self.log.lock().unwrap().push(entry);
            Ok(())
        }
    }

    impl Store for FakeStore {
        fn store_raw_offer(
            &self,
            entity: &RawOffer,
        ) -> Result<crate::platform::object_storage::OfferStorageMetadata, StoreError> {
            if self.fail {
                return Err(StoreError::ObjectStorage(
                    crate::platform::object_storage::Error::InvalidRecord,
                ));
            }

            self.log(format!(
                "stored {:?} {}: {}",
                entity.retailer_code, entity.url, entity.body
            ))?;
            Ok(crate::platform::object_storage::OfferStorageMetadata {
                scraped_at: entity.scraped_at,
                content_size: 0,
                external_id: Uuid::nil(),
                object_storage_id: Uuid::nil(),
            })
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
    fn every_attempt_waits_six_seconds_before_fetching_including_retries() {
        use std::sync::Arc;
        struct Attempts(Arc<Mutex<Vec<&'static str>>>);
        impl FetchOfferPage for Attempts {
            fn fetch(&self, _: &str) -> Result<String, FetchError> {
                self.0.lock().unwrap().push("fetch");
                Err(FetchError {
                    message: "retry me".into(),
                })
            }
        }
        let order = Arc::new(Mutex::new(Vec::new()));
        let handler = Handler::new(FakeStore::default(), Attempts(order.clone()));
        let command = command();
        for _ in 0..2 {
            assert!(
                handler
                    .handle_with_sleep(command.clone(), |duration| {
                        assert_eq!(duration, Duration::from_secs(6));
                        order.lock().unwrap().push("sleep");
                    })
                    .is_err()
            );
        }
        assert_eq!(*order.lock().unwrap(), ["sleep", "fetch", "sleep", "fetch"]);
        assert!(handler.store.log.lock().unwrap().is_empty());
    }

    #[test]
    fn successful_fetch_waits_once_before_fetch_and_store_even_if_storage_fails() {
        use std::sync::Arc;

        #[derive(Clone)]
        struct RecordedIo {
            order: Arc<Mutex<Vec<&'static str>>>,
            fail_store: bool,
        }
        impl FetchOfferPage for RecordedIo {
            fn fetch(&self, _: &str) -> Result<String, FetchError> {
                self.order.lock().unwrap().push("fetch");
                Ok("<html>offer</html>".into())
            }
        }
        impl Store for RecordedIo {
            fn store_raw_offer(
                &self,
                entity: &RawOffer,
            ) -> Result<crate::platform::object_storage::OfferStorageMetadata, StoreError>
            {
                self.order.lock().unwrap().push("store");
                assert_eq!(entity.body, "<html>offer</html>");
                FakeStore {
                    fail: self.fail_store,
                    ..Default::default()
                }
                .store_raw_offer(entity)
            }
        }

        for fail_store in [false, true] {
            let order = Arc::new(Mutex::new(Vec::new()));
            let io = RecordedIo {
                order: order.clone(),
                fail_store,
            };
            let handler = Handler::new(io.clone(), io);
            let result = handler.handle_with_sleep(command(), |duration| {
                assert_eq!(duration, Duration::from_secs(6));
                order.lock().unwrap().push("sleep");
            });
            assert_eq!(*order.lock().unwrap(), ["sleep", "fetch", "store"]);
            if fail_store {
                assert!(matches!(result, Err(DownloadOfferPageError::Store(_))));
            } else {
                assert_eq!(result.unwrap().len(), 1);
            }
        }
    }

    #[test]
    fn queued_retired_urls_download_replacements_without_changing_identity() {
        struct ReplacementPage;
        impl FetchOfferPage for ReplacementPage {
            fn fetch(&self, url: &str) -> Result<String, FetchError> {
                assert!(url.ends_with("/products/ugreen-nasync-dxp4800-pro-4-bay-nas"));
                Ok("<html>replacement</html>".into())
            }
        }
        for (retailer, region) in [
            (RetailerCode::UgreenNasDe, "de"),
            (RetailerCode::UgreenNasUk, "uk"),
        ] {
            let mut command = command();
            command.retailer_code = retailer;
            command.url = format!("https://ai-{region}.ugreen.com/products/dxp4800-pro");
            let handler = Handler::new(FakeStore::default(), ReplacementPage);
            let events = handler.handle_with_sleep(command.clone(), |_| {}).unwrap();
            assert_eq!(events[0].offer_id, command.offer_id);
            assert_eq!(events[0].url, command.url);
        }
    }

    #[test]
    fn downloads_and_stores_the_page() {
        let handler = Handler::new(FakeStore::default(), Page(Some("<html>offer</html>")));
        let command = command();

        let before = chrono::Utc::now();
        let events = handler.handle_with_sleep(command.clone(), |_| {}).unwrap();
        let time = events[0].scraped_at.unwrap();
        assert!(time.timestamp_micros() >= before.timestamp_micros());
        assert!(time <= chrono::Utc::now());
        assert_eq!(time.timestamp_subsec_nanos() % 1000, 0);
        assert_eq!(events[0].storage.unwrap().scraped_at, Some(time));

        assert_eq!(
            events,
            vec![OfferPageWasDownloaded {
                storage: events[0].storage,
                scraped_at: events[0].scraped_at,
                offer_id: command.offer_id,
                consolidated_sitemap_id: command.consolidated_sitemap_id,
                retailer_code: RetailerCode::UgreenCa,
                url: URL.to_string(),
            }]
        );
        assert_eq!(
            *handler.store.log.lock().unwrap(),
            vec![format!("stored UgreenCa {URL}: <html>offer</html>")]
        );
    }

    #[test]
    fn downloads_a_url_again_every_time() {
        let handler = Handler::new(FakeStore::default(), Page(Some("<html>offer</html>")));

        for _ in 0..2 {
            let events = handler.handle_with_sleep(command(), |_| {}).unwrap();
            assert_eq!(events.len(), 1);
        }

        assert_eq!(handler.store.log.lock().unwrap().len(), 2);
    }

    #[test]
    fn fails_the_command_when_the_download_fails() {
        let handler = Handler::new(FakeStore::default(), Page(None));

        let error = handler.handle_with_sleep(command(), |_| {}).unwrap_err();

        assert!(
            matches!(&error, DownloadOfferPageError::Fetch { url, .. } if url == URL),
            "{error:?}"
        );
        assert!(handler.store.log.lock().unwrap().is_empty());
    }

    #[test]
    fn storage_failure_prevents_success_events() {
        let store = FakeStore {
            fail: true,
            ..Default::default()
        };
        let handler = Handler::new(store, Page(Some("<html>offer</html>")));
        assert!(handler.handle_with_sleep(command(), |_| {}).is_err());
    }
}
