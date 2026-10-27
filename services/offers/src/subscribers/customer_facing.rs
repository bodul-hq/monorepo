use diesel::{RunQueryDsl, sql_types};
use kernel::{EventError, EventSubscriberPort, NewEventEnvelope};
use testimony_infra::prelude::DbPool;

use crate::offers::event::{
    OFFER_CREATED_EVENT, OFFER_IMAGE_ADDED_EVENT, OFFER_PRICE_CHANGED_EVENT,
    OFFER_SCRAPED_AT_UPDATED_EVENT, OFFER_TITLE_CHANGED_EVENT,
};

pub struct CustomerFacingSubscriber {
    pool: DbPool,
}

impl CustomerFacingSubscriber {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
}

impl EventSubscriberPort for CustomerFacingSubscriber {
    fn handle(&self, envelope: &NewEventEnvelope) -> Result<(), EventError> {
        if !matches!(
            envelope.event_type.as_str(),
            OFFER_CREATED_EVENT
                | OFFER_TITLE_CHANGED_EVENT
                | OFFER_PRICE_CHANGED_EVENT
                | OFFER_IMAGE_ADDED_EVENT
                | OFFER_SCRAPED_AT_UPDATED_EVENT
                | crate::offers::event::OFFER_LOCALIZATION_CHANGED_EVENT
                | crate::offers::event::OFFER_IDENTIFIERS_CHANGED_EVENT
                | crate::offers::event::OFFER_PRICING_CHANGED_EVENT
                | crate::offers::event::OFFER_AVAILABILITY_CHANGED_EVENT
                | crate::offers::event::OFFER_ITEM_CONDITION_CHANGED_EVENT
                | crate::offers::event::OFFER_CONTENT_CHANGED_EVENT
        ) {
            return Ok(());
        }
        let metadata = envelope
            .metadata
            .as_ref()
            .ok_or_else(err_missing_metadata)?;
        let mut meta = serde_json::json!({
            "event_id": metadata.event_id,
            "message_id": metadata.event_id,
            "correlation_id": metadata.correlation_id,
            "causation_id": metadata.causation_id,
            "event_type": envelope.event_type,
            "routing_key": "customer_facing",
            "source": metadata.source,
            "content_type": "application/json",
        });
        if let Some(sequence) = metadata.sequence {
            meta["sequence"] = serde_json::json!(sequence);
        }
        let mut connection = self.pool.get().map_err(err_outbox_write_failed)?;
        // The source event ID makes re-delivery of the same event idempotent.
        diesel::sql_query(
            "INSERT INTO outbox_entries (id, payload, meta) VALUES ($1, $2, $3) \
             ON CONFLICT (id) DO NOTHING",
        )
        .bind::<sql_types::Uuid, _>(metadata.event_id)
        .bind::<sql_types::Text, _>(&envelope.payload)
        .bind::<sql_types::Jsonb, _>(meta)
        .execute(&mut connection)
        .map_err(err_outbox_write_failed)?;
        Ok(())
    }
}

fn err_missing_metadata() -> EventError {
    EventError::SubscriberExecution(
        "event metadata is required for customer-facing outbox delivery".to_owned(),
    )
}

fn err_outbox_write_failed(error: impl std::fmt::Display) -> EventError {
    EventError::SubscriberExecution(format!("customer-facing outbox write failed: {error}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn requires_metadata_for_offer_events_and_ignores_unrelated_events() {
        let subscriber = CustomerFacingSubscriber::new(crate::test_pool());
        for event_type in [
            crate::offers::event::OFFER_PRICING_CHANGED_EVENT,
            crate::offers::event::OFFER_AVAILABILITY_CHANGED_EVENT,
            crate::offers::event::OFFER_ITEM_CONDITION_CHANGED_EVENT,
            crate::offers::event::OFFER_CONTENT_CHANGED_EVENT,
            OFFER_IMAGE_ADDED_EVENT,
            OFFER_CREATED_EVENT,
            OFFER_TITLE_CHANGED_EVENT,
            OFFER_SCRAPED_AT_UPDATED_EVENT,
            OFFER_PRICE_CHANGED_EVENT,
        ] {
            let envelope = NewEventEnvelope {
                event_type: event_type.to_owned(),
                payload: "{}".to_owned(),
                metadata: None,
            };
            assert!(matches!(
                subscriber.handle(&envelope),
                Err(EventError::SubscriberExecution(_))
            ));
        }
        subscriber
            .handle(&NewEventEnvelope {
                event_type: "Unrelated".to_owned(),
                payload: "{}".to_owned(),
                metadata: None,
            })
            .unwrap();
    }

    #[test]
    fn propagates_outbox_storage_failure() {
        let subscriber = CustomerFacingSubscriber::new(crate::test_pool());
        let envelope = NewEventEnvelope {
            event_type: OFFER_CREATED_EVENT.to_owned(),
            payload: "{}".to_owned(),
            metadata: Some(kernel::NewEventMetadata {
                sequence: None,
                event_id: uuid::Uuid::new_v4(),
                correlation_id: None,
                causation_id: None,
                source: None,
            }),
        };
        assert!(matches!(
            subscriber.handle(&envelope),
            Err(EventError::SubscriberExecution(_))
        ));
    }
}
