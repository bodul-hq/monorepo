use bodul_shared::prelude::RetailerCode;
use chrono::Utc;
use money::Money;
use testimony::prelude::{Aggregate, EventEnvelope};

use super::retailer_offer_id::RetailerOfferId;

use super::{
    event::{
        OfferCreatedEvent, OfferImageAddedEvent, OfferPriceChangedEvent, OfferTitleChangedEvent,
        RetailerOfferEvent,
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
    #[error("offer image position exhausted")]
    PositionOverflow,
}

impl RetailerOfferAggregate {
    pub fn create(
        id: RetailerOfferId,
        retailer_code: RetailerCode,
        title: String,
        price: Money,
        url: String,
        images: Vec<String>,
    ) -> Result<Self, RetailerOfferError> {
        let (images, skipped) = shared::images::filter(images);
        if skipped > 0 {
            eprintln!("skipped invalid offer images: {skipped}");
        }
        Self::guard_capacity(0, 0, images.len() as u64, 1)?;
        let event = OfferCreatedEvent {
            images,
            id,
            retailer_code,
            title,
            price,
            url,
            created_at: Utc::now(),
        };
        let mut state = RetailerOfferState::new(
            event.id.clone(),
            event.retailer_code,
            event.title.clone(),
            event.price.clone(),
            event.url.clone(),
            event.created_at,
        );
        for image in &event.images {
            state.add_image(image.clone());
        }
        let sequence = 1;

        Ok(Self {
            state,
            sequence,
            uncommitted_events: vec![EventEnvelope::new(
                RetailerOfferEvent::OfferCreated(event),
                sequence,
            )],
        })
    }

    fn guard_capacity(
        sequence: u64,
        existing_images: u64,
        new_images: u64,
        field_changes: u64,
    ) -> Result<(), RetailerOfferError> {
        let total = existing_images
            .checked_add(new_images)
            .ok_or(RetailerOfferError::PositionOverflow)?;
        if total > u64::from(u32::MAX) + 1 {
            return Err(RetailerOfferError::PositionOverflow);
        }
        sequence
            .checked_add(field_changes)
            .and_then(|sequence| sequence.checked_add(new_images))
            .filter(|sequence| *sequence <= i64::MAX as u64)
            .ok_or(RetailerOfferError::SequenceOverflow)?;
        Ok(())
    }

    pub fn update(&mut self, title: String, price: Money) -> Result<(), RetailerOfferError> {
        self.update_with_images(title, price, Vec::new())
    }

    pub fn update_with_images(
        &mut self,
        title: String,
        price: Money,
        images: Vec<String>,
    ) -> Result<(), RetailerOfferError> {
        let (images, skipped) = shared::images::filter(images);
        if skipped > 0 {
            eprintln!("skipped invalid offer images: {skipped}");
        }
        let images: Vec<_> = images
            .into_iter()
            .filter(|image| !self.state.images().contains(image))
            .collect();
        let title_changed = title != self.state.title();
        let price_changed = &price != self.state.price();
        // Reserve sequence capacity for all new events before changing state,
        // so an overflow rejects the whole update without partial changes.
        Self::guard_capacity(
            self.sequence,
            self.state.images().len() as u64,
            images.len() as u64,
            u64::from(title_changed) + u64::from(price_changed),
        )?;

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
        for image_url in images {
            let event = OfferImageAddedEvent {
                id: self.state.id().clone(),
                position: self.state.images().len() as u32,
                image_url,
            };
            self.state.add_image(event.image_url.clone());
            self.sequence += 1;
            self.uncommitted_events.push(EventEnvelope::new(
                RetailerOfferEvent::OfferImageAdded(event),
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

        super::event::guard_images(&created.images).expect("invalid creation images in history");
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

        for image in created.images {
            aggregate.state.add_image(image);
        }

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
                RetailerOfferEvent::OfferImageAdded(event) => {
                    assert_eq!(event.id, id, "offer history ID must match the requested ID");
                    assert_eq!(
                        u64::from(event.position),
                        aggregate.state.images().len() as u64,
                        "image position must be contiguous"
                    );
                    assert!(
                        shared::images::is_valid(&event.image_url),
                        "invalid image history URL"
                    );
                    assert!(
                        !aggregate.state.images().contains(&event.image_url),
                        "duplicate image history URL"
                    );
                    aggregate.state.add_image(event.image_url);
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
            Vec::new(),
        )
        .unwrap()
    }

    fn update_command() -> ImportOfferDetailsCommand {
        ImportOfferDetailsCommand {
            images: Vec::new(),
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

        let aggregate =
            RetailerOfferAggregate::create(id, retailer_code, title, price, url, Vec::new())
                .unwrap();

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
            images: Vec::new(),
            id: RetailerOfferId::new("offer-1"),
            retailer_code: RetailerCode::AnkerCom,
            title: "Charger".into(),
            price: Money::new(1999, Currency::USD),
            url: "https://example.com/charger".into(),
            created_at: *expected_state.created_at(),
        };
        assert_eq!(event, &expected_event);
    }

    #[test]
    fn images_append_after_other_changes_and_replay_without_new_events() {
        use kernel::ApplicationEvent;
        let mut offer = aggregate();
        let a = "https://shop.example/A?x=1&y=2".to_owned();
        let b = "https://shop.example/B".to_owned();
        let c = "https://shop.example/C".to_owned();
        offer
            .update_with_images(
                "Charger".into(),
                Money::new(1999, Currency::USD),
                vec![a.clone(), a.clone(), "/bad".into(), b.clone()],
            )
            .unwrap();
        assert_eq!(offer.state.images(), &[a.clone(), b.clone()]);
        assert_eq!(
            offer
                .get_just_events()
                .iter()
                .map(ApplicationEvent::event_type)
                .collect::<Vec<_>>(),
            ["OfferCreated", "OfferImageAdded", "OfferImageAdded"]
        );
        offer
            .update_with_images(
                "New title".into(),
                Money::new(2000, Currency::USD),
                vec![b.clone(), c.clone(), a.clone(), c.clone()],
            )
            .unwrap();
        assert_eq!(
            offer
                .get_just_events()
                .iter()
                .map(ApplicationEvent::event_type)
                .collect::<Vec<_>>(),
            [
                "OfferCreated",
                "OfferImageAdded",
                "OfferImageAdded",
                "OfferTitleChanged",
                "OfferPriceChangedEvent",
                "OfferImageAdded"
            ]
        );
        let event = offer.get_just_events().pop().unwrap();
        let payload = serde_json::to_value(&event).unwrap();
        assert_eq!(
            payload,
            serde_json::json!({"id":"offer-1","image_url":c,"position":2})
        );
        let record = crate::infra::records::OfferEventRecord::from(&event);
        let record = serde_json::to_value(record).unwrap();
        assert_eq!(record["type"], "OfferImageAdded.v1");
        assert_eq!(record["data"], payload);
        let mut replay = RetailerOfferAggregate::from_history(
            offer.state.id().clone(),
            offer.uncommitted_events.clone(),
        );
        assert_eq!(replay.state, offer.state);
        replay
            .update_with_images(
                "New title".into(),
                Money::new(2000, Currency::USD),
                vec![c, b, a],
            )
            .unwrap();
        replay
            .update("New title".into(), Money::new(2000, Currency::USD))
            .unwrap();
        assert!(replay.uncommitted_events.is_empty());
    }

    #[test]
    fn image_history_rejects_conflicts_and_update_overflow_is_atomic() {
        let mut offer = aggregate();
        offer.sequence = u64::MAX - 1;
        let before = format!("{:?}", offer.state);
        assert_eq!(
            offer.update_with_images(
                "Changed".into(),
                Money::new(1999, Currency::USD),
                vec!["https://shop.example/A".into()]
            ),
            Err(RetailerOfferError::SequenceOverflow)
        );
        assert_eq!(format!("{:?}", offer.state), before);
        assert_eq!(offer.uncommitted_events.len(), 1);
        for (url, position) in [("https://shop.example/A", 1), ("/bad", 0)] {
            let mut offer = aggregate();
            let event = OfferImageAddedEvent {
                id: offer.state.id().clone(),
                image_url: url.into(),
                position,
            };
            offer.uncommitted_events.push(EventEnvelope::new(
                RetailerOfferEvent::OfferImageAdded(event),
                2,
            ));
            assert!(
                std::panic::catch_unwind(|| RetailerOfferAggregate::from_history(
                    offer.state.id().clone(),
                    offer.uncommitted_events
                ))
                .is_err()
            );
        }
        let mut offer = aggregate();
        let event = OfferImageAddedEvent {
            id: offer.state.id().clone(),
            image_url: "https://shop.example/A".into(),
            position: 0,
        };
        offer.uncommitted_events.push(EventEnvelope::new(
            RetailerOfferEvent::OfferImageAdded(event.clone()),
            2,
        ));
        offer.uncommitted_events.push(EventEnvelope::new(
            RetailerOfferEvent::OfferImageAdded(OfferImageAddedEvent {
                position: 1,
                ..event
            }),
            3,
        ));
        assert!(
            std::panic::catch_unwind(|| RetailerOfferAggregate::from_history(
                offer.state.id().clone(),
                offer.uncommitted_events
            ))
            .is_err()
        );
    }

    #[test]
    fn image_position_capacity_includes_u32_max_and_rejects_overflow() {
        assert_eq!(
            RetailerOfferAggregate::guard_capacity(1, u64::from(u32::MAX), 1, 0),
            Ok(())
        );
        assert_eq!(
            RetailerOfferAggregate::guard_capacity(1, u64::from(u32::MAX) + 1, 1, 0),
            Err(RetailerOfferError::PositionOverflow)
        );
        assert_eq!(
            RetailerOfferAggregate::guard_capacity(1, u64::MAX, 1, 0),
            Err(RetailerOfferError::PositionOverflow)
        );
    }

    #[test]
    fn creation_carries_images_and_updates_add_only_unseen_images() {
        let a = "https://shop.example/A?x=1&y=2".to_owned();
        let b = "https://shop.example/B".to_owned();
        let c = "https://shop.example/C".to_owned();
        let id = RetailerOfferId::new("offer-1");
        let mut offer = RetailerOfferAggregate::create(
            id.clone(),
            RetailerCode::AnkerAu,
            "Charger".into(),
            Money::new(1999, Currency::AUD),
            "https://shop.example/charger".into(),
            vec![a.clone(), a.clone(), "/bad".into(), b.clone()],
        )
        .unwrap();
        assert_eq!(offer.sequence, 1);
        assert_eq!(offer.uncommitted_events.len(), 1);
        assert_eq!(offer.state.images(), &[a.clone(), b.clone()]);
        let RetailerOfferEvent::OfferCreated(event) = offer.uncommitted_events[0].event() else {
            panic!("expected only creation");
        };
        assert_eq!(event.images, [a.clone(), b.clone()]);
        let serialized = serde_json::to_value(crate::infra::records::OfferEventRecord::from(
            offer.uncommitted_events[0].event(),
        ))
        .unwrap();
        assert_eq!(serialized["type"], "OfferCreated.v2");
        assert_eq!(serialized["data"]["images"], serde_json::json!([a, b]));
        let replay =
            RetailerOfferAggregate::from_history(id.clone(), offer.uncommitted_events.clone());
        assert_eq!(replay.state, offer.state);
        offer
            .update_with_images(
                "Charger".into(),
                Money::new(1999, Currency::AUD),
                vec![b.clone(), c.clone(), a.clone(), c.clone()],
            )
            .unwrap();
        assert_eq!(offer.uncommitted_events.len(), 2);
        let RetailerOfferEvent::OfferImageAdded(added) = offer.uncommitted_events[1].event() else {
            panic!("expected image addition");
        };
        assert_eq!(added.image_url, c);
        assert_eq!(added.position, 2);
        let mut replay = RetailerOfferAggregate::from_history(id, offer.uncommitted_events.clone());
        assert_eq!(replay.state, offer.state);
        replay
            .update_with_images(
                "Charger".into(),
                Money::new(1999, Currency::AUD),
                vec![c, b, a],
            )
            .unwrap();
        replay
            .update("Charger".into(), Money::new(1999, Currency::AUD))
            .unwrap();
        assert!(replay.uncommitted_events.is_empty());
    }

    #[test]
    fn old_created_records_without_images_replay_with_legacy_image_events() {
        let original = aggregate();
        let mut record = serde_json::to_value(crate::infra::records::OfferEventRecord::from(
            original.uncommitted_events[0].event(),
        ))
        .unwrap();
        record["type"] = serde_json::json!("OfferCreated.v1");
        record["data"].as_object_mut().unwrap().remove("images");
        let decoded: crate::infra::records::OfferEventRecord =
            serde_json::from_value(record.clone()).unwrap();
        let id = original.state.id().clone();
        let image = OfferImageAddedEvent {
            id: id.clone(),
            image_url: "https://shop.example/legacy".into(),
            position: 0,
        };
        let history = vec![
            EventEnvelope::new(decoded.into(), 1),
            EventEnvelope::new(RetailerOfferEvent::OfferImageAdded(image), 2),
        ];
        let replay = RetailerOfferAggregate::from_history(id.clone(), history);
        assert_eq!(replay.state.images(), &["https://shop.example/legacy"]);
        assert!(replay.uncommitted_events.is_empty());
        for images in [
            serde_json::json!(["/bad"]),
            serde_json::json!(["https://shop.example/A", "https://shop.example/A"]),
        ] {
            record["data"]["images"] = images;
            let event: crate::infra::records::OfferEventRecord =
                serde_json::from_value(record.clone()).unwrap();
            let id = id.clone();
            assert!(
                std::panic::catch_unwind(|| RetailerOfferAggregate::from_history(
                    id,
                    vec![EventEnvelope::new(event.into(), 1)]
                ))
                .is_err()
            );
        }
    }
}

#[cfg(test)]
mod sequence_bound_tests {
    use super::*;
    #[test]
    fn bigint_limit_rejects_whole_batch_before_mutation() {
        let mut offer = RetailerOfferAggregate::create(
            RetailerOfferId::new("bounded"),
            RetailerCode::AnkerAu,
            "Charger".into(),
            Money::new(100, money::Currency::AUD),
            "https://shop.example/a".into(),
            vec![],
        )
        .unwrap();
        offer.sequence = i64::MAX as u64 - 1;
        let before = format!("{:?}", offer.state);
        assert_eq!(
            offer.update("Changed".into(), Money::new(200, money::Currency::AUD)),
            Err(RetailerOfferError::SequenceOverflow)
        );
        assert_eq!(format!("{:?}", offer.state), before);
        assert_eq!(offer.uncommitted_events.len(), 1);
        offer
            .update("Changed".into(), Money::new(100, money::Currency::AUD))
            .unwrap();
        assert_eq!(offer.sequence, i64::MAX as u64);
    }
}
