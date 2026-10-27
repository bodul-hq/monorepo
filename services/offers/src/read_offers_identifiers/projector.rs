use super::{
    entity::RetailerOfferIdentifiers,
    repository::{RetailerOfferIdentifiersRepository, RetailerOfferIdentifiersRepositoryError},
};
use crate::offers::event::{OFFER_CREATED_EVENT, OfferCreatedEvent};
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
                self.repository
                    .insert(&RetailerOfferIdentifiers {
                        id: event.id,
                        retailer_code: event.retailer_code,
                        url: event.url,
                    })
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
    }

    #[test]
    fn maps_supported_events_and_rejects_malformed_payloads() {
        let projector = RetailerOfferIdentifiersProjector::new(RecordingRepository::default());
        let event = OfferCreatedEvent {
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
        let expected = RetailerOfferIdentifiers {
            id: event.id.clone(),
            retailer_code: event.retailer_code,
            url: event.url.clone(),
        };
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
