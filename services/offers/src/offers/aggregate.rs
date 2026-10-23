use bodul_shared::prelude::RetailerCode;
use chrono::Utc;
use money::Money;
use testimony::prelude::{Aggregate, EventEnvelope};

use super::retailer_offer_id::RetailerOfferId;

use super::{
    event::{
        OfferCreatedEvent, OfferPriceChangedEvent, OfferTitleChangedEvent, RetailerOfferEvent,
    },
    state::RetailerOfferState,
};

pub struct RetailerOfferAggregate {
    pub state: RetailerOfferState,
    pub sequence: u64,
    pub uncommitted_events: Vec<EventEnvelope<RetailerOfferEvent>>,
}

impl RetailerOfferAggregate {
    pub fn get_just_events(&self) -> Vec<RetailerOfferEvent> {
        self.uncommitted_events
            .iter()
            .map(|envelope| envelope.event().clone())
            .collect()
    }
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum RetailerOfferError {
    #[error("offer ID cannot change")]
    IdMismatch,
    #[error("retailer code cannot change")]
    RetailerCodeMismatch,
    #[error("offer URL cannot change")]
    UrlMismatch,
    #[error("offer sequence exhausted")]
    SequenceOverflow,
}

impl RetailerOfferAggregate {
    pub fn create(
        id: RetailerOfferId,
        retailer_code: RetailerCode,
        title: String,
        price: Money,
        url: String,
    ) -> Self {
        let event = OfferCreatedEvent {
            id,
            retailer_code,
            title,
            price,
            url,
            created_at: Utc::now(),
        };
        let state = RetailerOfferState::new(
            event.id.clone(),
            event.retailer_code,
            event.title.clone(),
            event.price.clone(),
            event.url.clone(),
            event.created_at,
        );
        let sequence = 1;

        Self {
            state,
            sequence,
            uncommitted_events: vec![EventEnvelope::new(
                RetailerOfferEvent::OfferCreated(event),
                sequence,
            )],
        }
    }

    pub fn update(&mut self, title: String, price: Money) -> Result<(), RetailerOfferError> {
        let title_changed = title != self.state.title();
        let price_changed = &price != self.state.price();
        // Reserve sequence capacity for all new events before changing state,
        // so an overflow rejects the whole update without partial changes.
        self.sequence
            .checked_add(u64::from(title_changed) + u64::from(price_changed))
            .ok_or(RetailerOfferError::SequenceOverflow)?;

        if title_changed {
            let event = OfferTitleChangedEvent {
                id: self.state.id().clone(),
                title,
            };
            self.state.change_title(event.title.clone());
            self.sequence += 1;
            self.uncommitted_events.push(EventEnvelope::new(
                RetailerOfferEvent::OfferTitleChanged(event),
                self.sequence,
            ));
        }
        if price_changed {
            let event = OfferPriceChangedEvent {
                id: self.state.id().clone(),
                price,
            };
            self.state.change_price(event.price.clone());
            self.sequence += 1;
            self.uncommitted_events.push(EventEnvelope::new(
                RetailerOfferEvent::OfferPriceChanged(event),
                self.sequence,
            ));
        }
        Ok(())
    }
}

impl Aggregate for RetailerOfferAggregate {
    type Event = RetailerOfferEvent;
    type Id = RetailerOfferId;

    /// Replays a complete, ordered stream without recording new events.
    ///
    /// Panics if the stream is empty, does not start with a single creation
    /// event, contains another offer's events, or has noncontiguous sequences.
    fn from_history(id: Self::Id, events: Vec<EventEnvelope<Self::Event>>) -> Self {
        let mut events = events.into_iter();
        let first = events.next().expect("offer history cannot be empty");
        assert_eq!(
            first.sequence(),
            1,
            "offer history must start at sequence 1"
        );
        let RetailerOfferEvent::OfferCreated(created) = first.into_event() else {
            panic!("offer history must start with OfferCreated");
        };
        assert_eq!(
            created.id, id,
            "offer history ID must match the requested ID"
        );

        let mut aggregate = Self {
            state: RetailerOfferState::new(
                created.id,
                created.retailer_code,
                created.title,
                created.price,
                created.url,
                created.created_at,
            ),
            sequence: 1,
            uncommitted_events: Vec::new(),
        };

        for envelope in events {
            let sequence = envelope.sequence();
            let expected_sequence = aggregate
                .sequence
                .checked_add(1)
                .expect("offer history sequence exhausted");
            assert_eq!(
                sequence, expected_sequence,
                "offer history sequence must be contiguous"
            );
            match envelope.into_event() {
                RetailerOfferEvent::OfferCreated(_) => {
                    panic!("offer history must contain only one OfferCreated");
                }
                RetailerOfferEvent::OfferTitleChanged(event) => {
                    assert_eq!(event.id, id, "offer history ID must match the requested ID");
                    aggregate.state.change_title(event.title);
                }
                RetailerOfferEvent::OfferPriceChanged(event) => {
                    assert_eq!(event.id, id, "offer history ID must match the requested ID");
                    aggregate.state.change_price(event.price);
                }
            }
            aggregate.sequence = sequence;
        }

        aggregate
    }

    fn uncommitted_events(&self) -> &[EventEnvelope<Self::Event>] {
        &self.uncommitted_events
    }

    fn mark_events_committed(&mut self) {
        self.uncommitted_events.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::offers::command::ImportOfferDetailsCommand;
    use crate::offers::retailer_offer_id::RetailerOfferId;
    use bodul_shared::retailer::RetailerCode;
    use money::{Currency, Money};

    fn aggregate() -> RetailerOfferAggregate {
        RetailerOfferAggregate::create(
            RetailerOfferId::new("offer-1"),
            RetailerCode::AnkerCom,
            "Charger".into(),
            Money::new(1999, Currency::USD),
            "https://example.com/charger".into(),
        )
    }

    fn update_command() -> ImportOfferDetailsCommand {
        ImportOfferDetailsCommand {
            retailer_code: RetailerCode::AnkerCom,
            title: "Charger".into(),
            price: Money::new(1999, Currency::USD),
            url: "https://example.com/charger".into(),
        }
    }

    #[test]
    fn from_history_restores_state_without_uncommitted_events() {
        let mut original = aggregate();
        let id = original.state.id().clone();

        let created =
            RetailerOfferAggregate::from_history(id.clone(), original.uncommitted_events.clone());
        assert_eq!(created.state, original.state);
        assert_eq!(created.sequence, 1);
        assert!(created.uncommitted_events().is_empty());

        original
            .update("New charger".into(), Money::new(2499, Currency::EUR))
            .unwrap();
        original
            .update("Latest charger".into(), Money::new(2999, Currency::USD))
            .unwrap();

        let mut restored =
            RetailerOfferAggregate::from_history(id, original.uncommitted_events.clone());
        assert_eq!(restored.state, original.state);
        assert_eq!(restored.sequence, 5);
        assert!(restored.get_just_events().is_empty());

        restored
            .update("Latest charger".into(), Money::new(2999, Currency::USD))
            .unwrap();
        assert_eq!(restored.sequence, 5);
        assert!(restored.uncommitted_events().is_empty());

        restored
            .update("Final charger".into(), Money::new(3499, Currency::USD))
            .unwrap();
        assert_eq!(restored.sequence, 7);
        let sequences: Vec<_> = restored
            .uncommitted_events()
            .iter()
            .map(EventEnvelope::sequence)
            .collect();
        assert_eq!(sequences, vec![6, 7]);
    }

    #[test]
    fn from_history_rejects_invalid_streams() {
        let mut original = aggregate();
        original
            .update("New charger".into(), Money::new(2499, Currency::USD))
            .unwrap();
        let history = original.uncommitted_events;
        let created = history[0].clone();
        let title = history[1].clone();
        let price = history[2].clone();
        let other_id = RetailerOfferId::new("other-offer");
        let invalid_histories = [
            ("empty", vec![]),
            (
                "missing creation",
                vec![EventEnvelope::new(title.event().clone(), 1)],
            ),
            (
                "starts at zero",
                vec![EventEnvelope::new(created.event().clone(), 0)],
            ),
            (
                "starts at two",
                vec![EventEnvelope::new(created.event().clone(), 2)],
            ),
            (
                "duplicate creation",
                vec![
                    created.clone(),
                    EventEnvelope::new(created.event().clone(), 2),
                ],
            ),
            ("sequence gap", vec![created.clone(), price.clone()]),
            (
                "duplicate sequence",
                vec![
                    created.clone(),
                    title.clone(),
                    EventEnvelope::new(price.event().clone(), 2),
                ],
            ),
            ("out of order", vec![created.clone(), price, title]),
            (
                "wrong title ID",
                vec![
                    created.clone(),
                    EventEnvelope::new(
                        RetailerOfferEvent::OfferTitleChanged(OfferTitleChangedEvent {
                            id: other_id.clone(),
                            title: "Wrong offer".into(),
                        }),
                        2,
                    ),
                ],
            ),
            (
                "wrong price ID",
                vec![
                    created,
                    EventEnvelope::new(
                        RetailerOfferEvent::OfferPriceChanged(OfferPriceChangedEvent {
                            id: other_id.clone(),
                            price: Money::new(2499, Currency::USD),
                        }),
                        2,
                    ),
                ],
            ),
        ];

        for (case, events) in invalid_histories {
            assert!(
                std::panic::catch_unwind(|| {
                    RetailerOfferAggregate::from_history(RetailerOfferId::new("offer-1"), events)
                })
                .is_err(),
                "accepted {case} history"
            );
        }
        assert!(
            std::panic::catch_unwind(|| {
                RetailerOfferAggregate::from_history(other_id, history)
            })
            .is_err()
        );
    }

    #[test]
    fn update_emits_only_changed_fields_in_sequence() {
        for (title_changed, price_changed) in
            [(false, false), (true, false), (false, true), (true, true)]
        {
            let mut aggregate = aggregate();
            let created_at = *aggregate.state.created_at();
            let mut command = update_command();
            if title_changed {
                command.title = "New charger".into();
            }
            if price_changed {
                command.price = Money::new(2499, Currency::USD);
            }
            let expected_title = command.title.clone();
            let expected_price = command.price.clone();

            aggregate
                .update(command.title.clone(), command.price.clone())
                .unwrap();

            assert_eq!(aggregate.state.title(), expected_title);
            assert_eq!(aggregate.state.price(), &expected_price);
            assert_eq!(*aggregate.state.created_at(), created_at);
            let count = usize::from(title_changed) + usize::from(price_changed);
            assert_eq!(aggregate.uncommitted_events.len(), 1 + count);
            assert_eq!(aggregate.sequence, 1 + count as u64);
            for (index, envelope) in aggregate.uncommitted_events.iter().enumerate().skip(1) {
                assert_eq!(envelope.sequence(), index as u64 + 1);
                match envelope.event() {
                    RetailerOfferEvent::OfferTitleChanged(event) => {
                        assert!(title_changed);
                        assert_eq!(index, 1);
                        assert_eq!(event.id.value(), "offer-1");
                        assert_eq!(event.title, expected_title);
                    }
                    RetailerOfferEvent::OfferPriceChanged(event) => {
                        assert!(price_changed);
                        assert_eq!(index, 1 + usize::from(title_changed));
                        assert_eq!(event.id.value(), "offer-1");
                        assert_eq!(event.price, expected_price);
                    }
                    _ => panic!("unexpected creation event"),
                }
            }
        }
    }

    #[test]
    fn create_initializes_state_and_records_matching_creation_event() {
        let before = Utc::now();
        let id = RetailerOfferId::new("offer-1");
        let retailer_code = RetailerCode::AnkerCom;
        let title = "Charger".into();
        let price = Money::new(1999, Currency::USD);
        let url = "https://example.com/charger".into();

        let aggregate = RetailerOfferAggregate::create(id, retailer_code, title, price, url);

        let expected_state = RetailerOfferState::new(
            RetailerOfferId::new("offer-1"),
            RetailerCode::AnkerCom,
            "Charger",
            Money::new(1999, Currency::USD),
            "https://example.com/charger",
            *aggregate.state.created_at(),
        );
        assert_eq!(aggregate.state, expected_state);
        assert!(*aggregate.state.created_at() >= before);
        assert!(*aggregate.state.created_at() <= Utc::now());
        assert_eq!(aggregate.sequence, 1);
        assert_eq!(aggregate.uncommitted_events.len(), 1);
        let envelope = &aggregate.uncommitted_events[0];
        assert_eq!(envelope.sequence(), aggregate.sequence);
        let RetailerOfferEvent::OfferCreated(event) = envelope.event() else {
            panic!("expected an OfferCreated event");
        };
        let expected_event = OfferCreatedEvent {
            id: RetailerOfferId::new("offer-1"),
            retailer_code: RetailerCode::AnkerCom,
            title: "Charger".into(),
            price: Money::new(1999, Currency::USD),
            url: "https://example.com/charger".into(),
            created_at: *expected_state.created_at(),
        };
        assert_eq!(event, &expected_event);
    }
}
