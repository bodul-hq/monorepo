use super::{
    entity::RetailerOfferIdentifiers,
    repository::{RetailerOfferIdentifiersRepository, RetailerOfferIdentifiersRepositoryError},
};
use crate::offers::event::{
    OFFER_CREATED_EVENT, OFFER_IDENTIFIERS_CHANGED_EVENT, OfferCreatedEvent,
    OfferIdentifiersChangedEvent,
};
use kernel::{EventError, EventSubscriberPort, NewEventEnvelope};

pub struct RetailerOfferIdentifiersProjector<R> {
    repository: R,
}

impl<R: RetailerOfferIdentifiersRepository> RetailerOfferIdentifiersProjector<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }
}

impl<R: RetailerOfferIdentifiersRepository> EventSubscriberPort
    for RetailerOfferIdentifiersProjector<R>
{
    fn handle(&self, envelope: &NewEventEnvelope) -> Result<(), EventError> {
        match envelope.event_type.as_str() {
            OFFER_CREATED_EVENT => {
                let event: OfferCreatedEvent =
                    serde_json::from_str(&envelope.payload).map_err(err_invalid_event)?;
                let entity = RetailerOfferIdentifiers {
                    identifiers: event.identifiers,
                    id: event.id,
                    retailer_code: event.retailer_code,
                    url: event.url,
                };
                self.repository
                    .insert(&entity)
                    .map_err(err_projection_failed)
            }
            OFFER_IDENTIFIERS_CHANGED_EVENT => {
                let event: OfferIdentifiersChangedEvent =
                    serde_json::from_str(&envelope.payload).map_err(err_invalid_event)?;
                self.repository
                    .update_identifiers(&event.id, &event.identifiers)
                    .map_err(err_projection_failed)
            }
            _ => Ok(()),
        }
    }
}

fn err_invalid_event(error: serde_json::Error) -> EventError {
    EventError::Conversion(error.to_string())
}

fn err_projection_failed(error: RetailerOfferIdentifiersRepositoryError) -> EventError {
    EventError::SubscriberExecution(error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::offers::retailer_offer_id::RetailerOfferId;
    use std::sync::Mutex;

    #[derive(Default)]
    struct RecordingRepository(Mutex<Option<RetailerOfferIdentifiers>>);

    impl RetailerOfferIdentifiersRepository for RecordingRepository {
        fn insert(
            &self,
            entity: &RetailerOfferIdentifiers,
        ) -> Result<(), RetailerOfferIdentifiersRepositoryError> {
            *self.0.lock().unwrap() = Some(entity.clone());
            Ok(())
        }

        fn update_identifiers(
            &self,
            id: &RetailerOfferId,
            identifiers: &retailer_sauce::identifier::ProductIdentifiers,
        ) -> Result<(), RetailerOfferIdentifiersRepositoryError> {
            let mut state = self.0.lock().unwrap();
            let entity = state.as_mut().expect("creation is projected first");
            assert_eq!(&entity.id, id);
            entity.identifiers = identifiers.clone();
            Ok(())
        }
    }

    #[test]
    fn missing_change_target_fails_without_creating_a_partial_identity() {
        let projector = RetailerOfferIdentifiersProjector::new(
            crate::read_offers_identifiers::repository::InMemoryRetailerOfferIdentifiersRepository::new(),
        );
        let id = RetailerOfferId::new(uuid::Uuid::new_v4().to_string());
        let update = OfferIdentifiersChangedEvent {
            id: id.clone(),
            identifiers: Default::default(),
        };
        let envelope = NewEventEnvelope {
            event_type: OFFER_IDENTIFIERS_CHANGED_EVENT.into(),
            payload: serde_json::to_string(&update).unwrap(),
            metadata: None,
        };
        let error = projector.handle(&envelope).unwrap_err();
        assert!(matches!(error, EventError::SubscriberExecution(_)));
        assert!(
            error
                .to_string()
                .contains("offer identity has not been inserted")
        );
        // Insertion still succeeds with the same ID, proving failure created no row.
        let identity = RetailerOfferIdentifiers {
            identifiers: Default::default(),
            id,
            retailer_code: bodul_shared::retailer::RetailerCode::AnkerAu,
            url: "https://example.com/new-identity".into(),
        };
        projector.repository.insert(&identity).unwrap();
        projector.handle(&envelope).unwrap();
    }

    #[test]
    fn maps_supported_events_and_rejects_malformed_payloads() {
        let projector = RetailerOfferIdentifiersProjector::new(RecordingRepository::default());
        let identifiers = serde_json::from_value(serde_json::json!({
            "gtin":"0012345678905", "sku":"original", "external_identifiers": {
                "shopify_product_gid":"001", "shopify_product_variation_gid":"002", "shopify_handle":"charger"
            }
        })).unwrap();
        let event = OfferCreatedEvent {
            facts: Default::default(),
            identifiers,
            localization: Default::default(),
            scraped_at: None,
            images: Vec::new(),
            id: RetailerOfferId::new(uuid::Uuid::new_v4().to_string()),
            retailer_code: bodul_shared::retailer::RetailerCode::from_str("ankerau").unwrap(),
            title: "Charger".to_owned(),
            price: money::Money::new(1999, money::Currency::AUD),
            url: "https://example.com/Charger?q=red#details".to_owned(),
            created_at: chrono::Utc::now(),
        };
        let mut envelope = NewEventEnvelope {
            event_type: OFFER_CREATED_EVENT.to_owned(),
            payload: serde_json::to_string(&event).unwrap(),
            metadata: None,
        };
        let unavailable = RetailerOfferIdentifiersProjector::new(
            crate::read_offers_identifiers::repository::PostgresRetailerOfferIdentifiersRepository::new(crate::test_pool()),
        );
        assert!(matches!(
            unavailable.handle(&envelope),
            Err(EventError::SubscriberExecution(_))
        ));
        projector.handle(&envelope).unwrap();
        let mut expected = RetailerOfferIdentifiers {
            identifiers: event.identifiers.clone(),
            id: event.id.clone(),
            retailer_code: event.retailer_code,
            url: event.url.clone(),
        };
        assert_eq!(
            projector.repository.0.lock().unwrap().as_ref(),
            Some(&expected)
        );
        let update: OfferIdentifiersChangedEvent = serde_json::from_value(serde_json::json!({
            "id":event.id, "gtin":"0012345678905", "sku":"SKU-001",
            "external_identifiers":{"shopify_product_gid":"gid://shopify/Product/001", "shopify_product_variation_gid":null,"shopify_handle":" Exact handle "}
        })).unwrap();
        expected.identifiers = update.identifiers.clone();
        envelope.event_type = OFFER_IDENTIFIERS_CHANGED_EVENT.to_owned();
        envelope.payload = serde_json::to_string(&update).unwrap();
        projector.handle(&envelope).unwrap();
        assert_eq!(
            projector.repository.0.lock().unwrap().as_ref(),
            Some(&expected)
        );
        envelope.payload = "{}".to_owned();
        assert!(matches!(
            projector.handle(&envelope),
            Err(EventError::Conversion(_))
        ));
        envelope.event_type = "Unrelated".to_owned();
        projector.handle(&envelope).unwrap();
        assert_eq!(
            projector.repository.0.lock().unwrap().as_ref(),
            Some(&expected)
        );
    }
}
