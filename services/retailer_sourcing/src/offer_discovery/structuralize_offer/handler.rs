use crate::platform::object_storage::Stored;
use bodul_shared::retailer::RetailerCode;
use serde_json::Value;
use uuid::Uuid;

use super::command::StructuralizeOffer;
use super::entity::{NormalizedOffer, StructuralizedOffer};
use super::event::OfferWasStructuralized;
use super::store::StoreError;
use crate::offer_discovery::Retailers;
use crate::platform::bus::Handle;

/// The storage this slice needs.
pub trait Store: Send + Sync + 'static {
    /// The offer's normalized document.
    fn normalized_offer(&self, offer_id: Uuid) -> Result<Option<NormalizedOffer>, StoreError>;

    /// Stores the structuralized document, replacing any stored for the offer before.
    fn store_structuralized_offer(
        &self,
        entity: &StructuralizedOffer,
    ) -> Result<Stored<StructuralizedOffer>, StoreError>;
}

/// The retailer step this slice runs.
pub trait Structuralize: Send + Sync + 'static {
    fn structuralize(
        &self,
        retailer_code: RetailerCode,
        input: &str,
    ) -> Result<Option<String>, String>;
}

impl Structuralize for Retailers {
    fn structuralize(
        &self,
        retailer_code: RetailerCode,
        input: &str,
    ) -> Result<Option<String>, String> {
        Retailers::structuralize(*self, retailer_code, input)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum HandlerError {
    #[error(transparent)]
    Store(#[from] StoreError),

    #[error("no normalized document for offer {0}")]
    MissingInput(Uuid),

    #[error("structuralizing offer {offer_id} failed: {message}")]
    Step { offer_id: Uuid, message: String },

    #[error("the structuralized document of offer {offer_id} is not JSON: {source}")]
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

impl<S: Store, R: Structuralize> Handle for Handler<S, R> {
    type Command = StructuralizeOffer;
    type Event = OfferWasStructuralized;
    type Error = HandlerError;

    fn handle(
        &self,
        command: StructuralizeOffer,
    ) -> Result<Vec<OfferWasStructuralized>, HandlerError> {
        let input = self
            .store
            .normalized_offer(command.offer_id)?
            .ok_or(HandlerError::MissingInput(command.offer_id))?;

        // A failure fails the command, so the bus retries it later.
        tracing::debug!(object_storage_id = %input.object_storage_id, "input object loaded");
        let output = self
            .retailers
            .structuralize(command.retailer_code, &input.document.to_string())
            .map_err(|message| HandlerError::Step {
                offer_id: command.offer_id,
                message,
            })?;
        let Some(output) = output else {
            // The retailer produces no document for this offer: nothing to
            // store, and nothing follows.
            tracing::info!(
                offer_id = %command.offer_id,
                url = %command.url,
                "offer not structuralized: the retailer produces no structuralized document"
            );
            return Ok(Vec::new());
        };

        let document: Value =
            serde_json::from_str(&output).map_err(|source| HandlerError::NotJson {
                offer_id: command.offer_id,
                source,
            })?;

        let stored = self
            .store
            .store_structuralized_offer(&StructuralizedOffer {
                offer_id: input.offer_id,
                document,
            })?;
        tracing::info!(
            object_storage_id = %stored.object_storage_id,
            offer_id = %command.offer_id,
            url = %command.url,
            bytes = output.len(),
            "offer structuralized"
        );

        Ok(vec![OfferWasStructuralized {
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
                input: Some(r#"{"title":"offer"}"#),
                ..Self::default()
            }
        }
    }

    impl Store for FakeStore {
        fn normalized_offer(&self, offer_id: Uuid) -> Result<Option<NormalizedOffer>, StoreError> {
            Ok(self.input.map(|input| NormalizedOffer {
                object_storage_id: uuid::Uuid::now_v7(),
                offer_id,
                document: serde_json::from_str(input).unwrap(),
            }))
        }

        fn store_structuralized_offer(
            &self,
            entity: &StructuralizedOffer,
        ) -> Result<Stored<StructuralizedOffer>, StoreError> {
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
    struct FakeStep(Option<Option<&'static str>>);

    impl Structuralize for FakeStep {
        fn structuralize(&self, _: RetailerCode, _: &str) -> Result<Option<String>, String> {
            self.0
                .map(|output| output.map(str::to_string))
                .ok_or_else(|| "broken".to_string())
        }
    }

    fn command() -> StructuralizeOffer {
        StructuralizeOffer {
            offer_id: Uuid::now_v7(),
            consolidated_sitemap_id: Uuid::now_v7(),
            retailer_code: RetailerCode::AnkerAu,
            url: URL.to_string(),
        }
    }

    #[test]
    fn structuralizes_and_stores_the_offer() {
        let handler = Handler::new(
            FakeStore::with_input(),
            FakeStep(Some(Some(r#"{"variations":[]}"#))),
        );
        let command = command();

        let events = handler.handle(command.clone()).unwrap();

        assert_eq!(
            events,
            vec![OfferWasStructuralized {
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
                serde_json::from_str::<Value>(r#"{"variations":[]}"#).unwrap()
            )]
        );
    }

    #[test]
    fn fails_when_the_input_is_missing() {
        let handler = Handler::new(
            FakeStore::default(),
            FakeStep(Some(Some(r#"{"variations":[]}"#))),
        );
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
        let handler = Handler::new(FakeStore::with_input(), FakeStep(Some(Some("not json"))));

        let error = handler.handle(command()).unwrap_err();

        assert!(matches!(error, HandlerError::NotJson { .. }));
        assert!(handler.store.stored.lock().unwrap().is_empty());
    }

    #[test]
    fn completes_without_an_event_when_the_retailer_produces_nothing() {
        let handler = Handler::new(FakeStore::with_input(), FakeStep(Some(None)));

        let events = handler.handle(command()).unwrap();

        assert!(events.is_empty());
        assert!(handler.store.stored.lock().unwrap().is_empty());
    }

    #[test]
    fn storage_failure_prevents_success_events() {
        let mut store = FakeStore::with_input();
        store.fail = true;
        let handler = Handler::new(store, FakeStep(Some(Some("{}"))));
        assert!(handler.handle(command()).is_err());
    }
}
