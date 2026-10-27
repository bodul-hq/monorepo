use super::{
    entity::RetailerOffer,
    repository::{RetailerOfferRepository, RetailerOfferRepositoryError},
};
use crate::offers::event::{
    OFFER_CREATED_EVENT, OfferCreatedEvent, OfferPriceChangedEvent, OfferTitleChangedEvent,
};
use kernel::{EventError, EventSubscriberPort, NewEventEnvelope};

pub struct RetailerOfferProjector<R> {
    repository: R,
}

impl<R: RetailerOfferRepository> RetailerOfferProjector<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }
}

impl<R: RetailerOfferRepository> EventSubscriberPort for RetailerOfferProjector<R> {
    fn handle(&self, envelope: &NewEventEnvelope) -> Result<(), EventError> {
        match envelope.event_type.as_str() {
            crate::offers::event::OFFER_SCRAPED_AT_UPDATED_EVENT => {
                let event: crate::offers::event::OfferScrapedAtUpdatedEvent =
                    serde_json::from_str(&envelope.payload).map_err(err_invalid_event)?;
                self.repository
                    .update_scraped_at(&event.id, event.scraped_at)
                    .map_err(err_projection_failed)
            }
            crate::offers::event::OFFER_IMAGE_ADDED_EVENT => {
                let event = serde_json::from_str(&envelope.payload).map_err(err_invalid_event)?;
                self.repository
                    .add_image(&event)
                    .map_err(err_projection_failed)
            }
            OFFER_CREATED_EVENT => {
                let event: OfferCreatedEvent =
                    serde_json::from_str(&envelope.payload).map_err(err_invalid_event)?;

                let entity = RetailerOffer {
                    scraped_at: event.scraped_at,
                    images: event.images,
                    id: event.id,
                    retailer_code: event.retailer_code,
                    url: event.url,
                    title: event.title,
                    price: event.price,
                    created_at: event.created_at,
                };

                self.repository
                    .insert(&entity)
                    .map_err(err_projection_failed)
            }
            crate::offers::event::OFFER_TITLE_CHANGED_EVENT => {
                let event: OfferTitleChangedEvent =
                    serde_json::from_str(&envelope.payload).map_err(err_invalid_event)?;
                self.repository
                    .update_title(&event.id, &event.title)
                    .map_err(err_projection_failed)
            }
            crate::offers::event::OFFER_PRICE_CHANGED_EVENT => {
                let event: OfferPriceChangedEvent =
                    serde_json::from_str(&envelope.payload).map_err(err_invalid_event)?;
                self.repository
                    .update_price(&event.id, &event.price)
                    .map_err(err_projection_failed)
            }
            _ => Ok(()),
        }
    }
}

fn err_invalid_event(error: serde_json::Error) -> EventError {
    EventError::Conversion(error.to_string())
}

fn err_projection_failed(error: RetailerOfferRepositoryError) -> EventError {
    EventError::SubscriberExecution(error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::offers::event::RetailerOfferEvent;
    use crate::offers::retailer_offer_id::RetailerOfferId;
    use std::sync::Mutex;

    #[derive(Default)]
    struct RecordingRepository(Mutex<Option<RetailerOffer>>);

    impl RetailerOfferRepository for RecordingRepository {
        fn update_scraped_at(
            &self,
            id: &RetailerOfferId,
            scraped_at: chrono::DateTime<chrono::Utc>,
        ) -> Result<(), RetailerOfferRepositoryError> {
            let mut state = self.0.lock().unwrap();
            let entity = state.as_mut().unwrap();
            assert_eq!(&entity.id, id);
            entity.scraped_at = entity.scraped_at.max(Some(scraped_at));
            Ok(())
        }

        fn images(&self, _: &RetailerOfferId) -> Result<Vec<String>, RetailerOfferRepositoryError> {
            Ok(Vec::new())
        }
        fn add_image(
            &self,
            _: &crate::offers::event::OfferImageAddedEvent,
        ) -> Result<(), RetailerOfferRepositoryError> {
            Ok(())
        }

        fn insert(&self, entity: &RetailerOffer) -> Result<(), RetailerOfferRepositoryError> {
            *self.0.lock().unwrap() = Some(entity.clone());
            Ok(())
        }
        fn update_title(
            &self,
            id: &RetailerOfferId,
            title: &str,
        ) -> Result<(), RetailerOfferRepositoryError> {
            let mut state = self.0.lock().unwrap();
            let entity = state.as_mut().unwrap();
            assert_eq!(&entity.id, id);
            entity.title = title.to_owned();
            Ok(())
        }
        fn update_price(
            &self,
            id: &RetailerOfferId,
            price: &money::Money,
        ) -> Result<(), RetailerOfferRepositoryError> {
            let mut state = self.0.lock().unwrap();
            let entity = state.as_mut().unwrap();
            assert_eq!(&entity.id, id);
            entity.price = price.clone();
            Ok(())
        }
    }

    #[test]
    fn maps_supported_events_and_rejects_malformed_payloads() {
        let projector = RetailerOfferProjector::new(RecordingRepository::default());
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
            payload: serde_json::to_string(&RetailerOfferEvent::OfferCreated(event.clone()))
                .unwrap(),
            metadata: None,
        };
        let unavailable = RetailerOfferProjector::new(
            crate::read_offer::repository::PostgresRetailerOfferRepository::new(crate::test_pool()),
        );
        assert!(matches!(
            unavailable.handle(&envelope),
            Err(EventError::SubscriberExecution(_))
        ));
        projector.handle(&envelope).unwrap();
        let mut expected = RetailerOffer {
            scraped_at: None,
            images: Vec::new(),
            id: event.id.clone(),
            retailer_code: event.retailer_code,
            url: event.url.clone(),
            title: event.title.clone(),
            price: event.price.clone(),
            created_at: event.created_at,
        };
        assert_eq!(
            projector.repository.0.lock().unwrap().as_ref(),
            Some(&expected)
        );
        envelope.event_type = crate::offers::event::OFFER_TITLE_CHANGED_EVENT.to_owned();
        envelope.payload = serde_json::to_string(&RetailerOfferEvent::OfferTitleChanged(
            OfferTitleChangedEvent {
                id: event.id.clone(),
                title: "New title".to_owned(),
            },
        ))
        .unwrap();
        projector.handle(&envelope).unwrap();
        expected.title = "New title".to_owned();
        assert_eq!(
            projector.repository.0.lock().unwrap().as_ref(),
            Some(&expected)
        );
        envelope.event_type = crate::offers::event::OFFER_PRICE_CHANGED_EVENT.to_owned();
        expected.price = money::Money::new(1200, money::Currency::EUR);
        envelope.payload = serde_json::to_string(&RetailerOfferEvent::OfferPriceChanged(
            OfferPriceChangedEvent {
                id: event.id,
                price: expected.price.clone(),
            },
        ))
        .unwrap();
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
