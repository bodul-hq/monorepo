use crate::platform::object_storage::Stored;
use bodul_shared::retailer::RetailerCode;
use serde_json::Value;
use uuid::Uuid;

use super::command::ExtractOfferPage;
use super::entity::{ExtractedOffer, RawOffer};
use super::event::OfferPageWasExtracted;
use super::store::StoreError;
use crate::offer_discovery::Retailers;
use crate::platform::bus::Handle;

/// The storage this slice needs.
pub trait Store: Send + Sync + 'static {
    /// The offer's downloaded page.
    fn raw_offer(&self, offer_id: Uuid) -> Result<Option<RawOffer>, StoreError>;

    /// Stores the extracted JSON, replacing any stored for the offer before.
    fn store_extracted_offer(
        &self,
        entity: &ExtractedOffer,
    ) -> Result<Stored<ExtractedOffer>, StoreError>;
}

/// The retailer step this slice runs.
pub trait Extract: Send + Sync + 'static {
    fn extract(&self, retailer_code: RetailerCode, input: &str) -> Result<String, String>;
}

impl Extract for Retailers {
    fn extract(&self, retailer_code: RetailerCode, input: &str) -> Result<String, String> {
        Retailers::extract(*self, retailer_code, input)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum HandlerError {
    #[error(transparent)]
    Store(#[from] StoreError),

    #[error("no downloaded page for offer {0}")]
    MissingInput(Uuid),

    #[error("extracting offer {offer_id} failed: {message}")]
    Step { offer_id: Uuid, message: String },

    #[error("the extracted JSON of offer {offer_id} is not JSON: {source}")]
    NotJson {
        offer_id: Uuid,
        source: serde_json::Error,
    },
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

impl<S: Store, R: Extract> Handle for Handler<S, R> {
    type Command = ExtractOfferPage;
    type Event = OfferPageWasExtracted;
    type Error = HandlerError;

    fn handle(
        &self,
        command: ExtractOfferPage,
    ) -> Result<Vec<OfferPageWasExtracted>, HandlerError> {
        let input = self
            .store
            .raw_offer(command.offer_id)?
            .ok_or(HandlerError::MissingInput(command.offer_id))?;

        // A failure fails the command, so the bus retries it later.
        tracing::debug!(object_storage_id = %input.object_storage_id, "input object loaded");
        let output = self
            .retailers
            .extract(command.retailer_code, &input.body)
            .map_err(|message| HandlerError::Step {
                offer_id: command.offer_id,
                message,
            })?;

        let document: Value =
            serde_json::from_str(&output).map_err(|source| HandlerError::NotJson {
                offer_id: command.offer_id,
                source,
            })?;

        let stored = self.store.store_extracted_offer(&ExtractedOffer {
            offer_id: input.offer_id,
            document,
        })?;
        tracing::info!(
            object_storage_id = %stored.object_storage_id,
            offer_id = %command.offer_id,
            url = %command.url,
            bytes = output.len(),
            "offer page extracted"
        );

        Ok(vec![OfferPageWasExtracted {
            offer_id: command.offer_id,
            consolidated_sitemap_id: command.consolidated_sitemap_id,
            retailer_code: command.retailer_code,
            url: command.url,
        }])
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use super::*;

    const URL: &str = "https://shop.test/products/a";

    #[derive(Default)]
    struct FakeStore {
        fail: bool,
        input: Option<&'static str>,
        stored: Mutex<Vec<(Uuid, Value)>>,
    }

    impl FakeStore {
        fn with_input() -> Self {
            Self {
                input: Some("<html>offer</html>"),
                ..Self::default()
            }
        }
    }

    impl Store for FakeStore {
        fn raw_offer(&self, offer_id: Uuid) -> Result<Option<RawOffer>, StoreError> {
            Ok(self.input.map(|input| RawOffer {
                object_storage_id: uuid::Uuid::now_v7(),
                offer_id,
                body: input.to_string(),
            }))
        }

        fn store_extracted_offer(
            &self,
            entity: &ExtractedOffer,
        ) -> Result<Stored<ExtractedOffer>, StoreError> {
            if self.fail {
                return Err(StoreError::ObjectStorage(
                    crate::platform::object_storage::Error::InvalidRecord,
                ));
            }

            self.stored
                .lock()
                .unwrap()
                .push((entity.offer_id, entity.document.clone()));
            Ok(Stored {
                entity: entity.clone(),
                object_storage_id: uuid::Uuid::now_v7(),
            })
        }
    }

    /// Returns one output, or fails every call.
    struct FakeStep(Option<&'static str>);

    impl Extract for FakeStep {
        fn extract(&self, _: RetailerCode, _: &str) -> Result<String, String> {
            self.0
                .map(str::to_string)
                .ok_or_else(|| "broken".to_string())
        }
    }

    fn command() -> ExtractOfferPage {
        ExtractOfferPage {
            offer_id: Uuid::now_v7(),
            consolidated_sitemap_id: Uuid::now_v7(),
            retailer_code: RetailerCode::AnkerAu,
            url: URL.to_string(),
        }
    }

    #[test]
    fn extracts_and_stores_the_page() {
        let handler = Handler::new(
            FakeStore::with_input(),
            FakeStep(Some(r#"{"name":"offer"}"#)),
        );
        let command = command();

        let events = handler.handle(command.clone()).unwrap();

        assert_eq!(
            events,
            vec![OfferPageWasExtracted {
                offer_id: command.offer_id,
                consolidated_sitemap_id: command.consolidated_sitemap_id,
                retailer_code: RetailerCode::AnkerAu,
                url: URL.to_string(),
            }]
        );
        assert_eq!(
            *handler.store.stored.lock().unwrap(),
            vec![(
                command.offer_id,
                serde_json::from_str::<Value>(r#"{"name":"offer"}"#).unwrap()
            )]
        );
    }

    #[test]
    fn fails_when_the_input_is_missing() {
        let handler = Handler::new(FakeStore::default(), FakeStep(Some(r#"{"name":"offer"}"#)));
        let command = command();

        let error = handler.handle(command.clone()).unwrap_err();

        assert!(matches!(error, HandlerError::MissingInput(id) if id == command.offer_id));
        assert!(handler.store.stored.lock().unwrap().is_empty());
    }

    #[test]
    fn fails_when_the_retailer_step_fails() {
        let handler = Handler::new(FakeStore::with_input(), FakeStep(None));

        let error = handler.handle(command()).unwrap_err();

        assert!(matches!(error, HandlerError::Step { ref message, .. } if message == "broken"));
        assert!(handler.store.stored.lock().unwrap().is_empty());
    }

    #[test]
    fn fails_when_the_output_is_not_json() {
        let handler = Handler::new(FakeStore::with_input(), FakeStep(Some("not json")));

        let error = handler.handle(command()).unwrap_err();

        assert!(matches!(error, HandlerError::NotJson { .. }));
        assert!(handler.store.stored.lock().unwrap().is_empty());
    }

    #[test]
    fn storage_failure_prevents_success_events() {
        let mut store = FakeStore::with_input();
        store.fail = true;
        let handler = Handler::new(store, FakeStep(Some("{}")));
        assert!(handler.handle(command()).is_err());
    }
}
