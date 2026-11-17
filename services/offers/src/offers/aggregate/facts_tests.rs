use super::super::event::OfferFacts;
use super::super::event::OfferPriceChangedEvent;
use super::tests::aggregate;
use super::*;
use kernel::ApplicationEvent;
use money::Currency;
use offer_content::singleton;
use retailer_sauce::{
    availability::Availability, item_condition::ItemCondition, observation::Observation,
    structuralized::content::Content,
};
use serde_json::Value;
use serde_json::from_str;
use serde_json::from_value as decode_json_value;
use serde_json::json;
use serde_json::to_string;
use serde_json::to_value;

fn observed() -> OfferObservation {
    OfferObservation {
        was_price: Observation::Observed(Some(Money::new(2999, Currency::USD))),
        availability: Observation::Observed(Some(Availability::Available)),
        item_condition: Observation::Observed(Some(ItemCondition::New)),
        content: Some(Content {
            description: Some(" <p>Exact &amp; source ž</p>\n".into()),
        }),
        ..Default::default()
    }
}

fn update(
    offer: &mut RetailerOfferAggregate,
    price: i64,
    observation: OfferObservation,
) -> Result<(), RetailerOfferError> {
    offer.update_with_observation(
        "Charger".into(),
        Money::new(price, Currency::USD),
        vec![],
        observation,
    )
}

#[test]
fn facts_change_independently_and_replay_without_reinterpreting_source() {
    let mut offer = aggregate();
    update(&mut offer, 1999, observed()).unwrap();
    assert_eq!(
        offer
            .uncommitted_events
            .iter()
            .map(|e| e.event().event_type())
            .collect::<Vec<_>>(),
        [
            "OfferCreated",
            "OfferContentsChanged",
            "OfferPricingChanged",
            "OfferAvailabilityChanged",
            "OfferItemConditionChanged"
        ]
    );
    let facts = offer.state.facts.clone();
    update(&mut offer, 1999, observed()).unwrap();
    update(&mut offer, 1999, OfferObservation::default()).unwrap();
    assert_eq!(offer.sequence, 5);
    assert_eq!(offer.state.facts, facts);
    let restored = RetailerOfferAggregate::from_history(
        offer.state.id().clone(),
        offer.uncommitted_events.clone(),
    );
    assert_eq!(restored.state, offer.state);
    assert!(restored.uncommitted_events.is_empty());
    let clear = OfferObservation {
        was_price: Observation::Observed(None),
        availability: Observation::Observed(None),
        item_condition: Observation::Observed(None),
        content: Some(Content::default()),
        ..Default::default()
    };
    update(&mut offer, 1999, clear).unwrap();
    assert_eq!(offer.sequence, 9);
    assert_eq!(
        offer.state.facts,
        OfferFacts {
            content: singleton("Charger".into(), None, None),
            ..Default::default()
        }
    );
}

#[test]
fn sale_changes_are_atomic_and_invalid_observations_leave_state_unchanged() {
    let mut offer = aggregate();
    update(&mut offer, 1999, observed()).unwrap();
    let original = RetailerOfferAggregate::from_history(
        offer.state.id().clone(),
        offer.uncommitted_events.clone(),
    )
    .state;
    for was in [
        Money::new(1999, Currency::USD),
        Money::new(2999, Currency::EUR),
    ] {
        let observation = OfferObservation {
            was_price: Observation::Observed(Some(was)),
            ..observed()
        };
        assert!(update(&mut offer, 1999, observation).is_err());
        assert_eq!(offer.state, original);
        assert_eq!(offer.sequence, 5);
    }
    // An old import cannot leave a comparison price below the new payable price.
    update(&mut offer, 3999, OfferObservation::default()).unwrap();
    let RetailerOfferEvent::OfferPricingChanged(event) =
        offer.uncommitted_events.last().unwrap().event()
    else {
        panic!("expected pricing change")
    };
    assert_eq!(event.pricing.price.amount(), 3999);
    assert!(event.pricing.was_price.is_none());
    assert!(offer.state.facts.was_price.is_none());
}

#[test]
fn legacy_price_event_changes_only_price_during_replay() {
    let mut offer = aggregate();
    update(&mut offer, 1999, observed()).unwrap();
    let facts = offer.state.facts.clone();
    let legacy = OfferPriceChangedEvent {
        id: offer.state.id().clone(),
        price: Money::new(3999, Currency::EUR),
    };
    offer.uncommitted_events.push(EventEnvelope::new(
        RetailerOfferEvent::OfferPriceChanged(legacy),
        6,
    ));
    let restored =
        RetailerOfferAggregate::from_history(offer.state.id().clone(), offer.uncommitted_events);
    assert_eq!(restored.state.facts, facts);
    assert_eq!(restored.state.price(), &Money::new(3999, Currency::EUR));
}

#[test]
fn event_records_keep_all_versions_and_do_not_confuse_nullable_events() {
    use crate::infra::records::OfferEventRecord;
    let mut offer = aggregate();
    update(&mut offer, 1999, observed()).unwrap();
    for envelope in &offer.uncommitted_events {
        let wire = to_value(envelope.event()).unwrap();
        let parsed: RetailerOfferEvent = decode_json_value(wire).unwrap();
        assert_eq!(parsed.event_type(), envelope.event().event_type());
        let record = OfferEventRecord::from(envelope.event());
        let parsed: OfferEventRecord = decode_json_value(to_value(record).unwrap()).unwrap();
        let parsed: RetailerOfferEvent = parsed.into();
        assert_eq!(parsed.event_type(), envelope.event().event_type());
    }
    let created = to_value(OfferEventRecord::from(offer.uncommitted_events[0].event())).unwrap();
    for version in 1..=8 {
        let mut old = created.clone();
        old["type"] = json!(format!("OfferCreated.v{version}"));
        for field in ["was_price", "availability", "item_condition", "content"] {
            old["data"].as_object_mut().unwrap().remove(field);
        }
        let parsed: OfferEventRecord = decode_json_value(old).unwrap();
        let RetailerOfferEvent::OfferCreated(parsed) = RetailerOfferEvent::from(parsed) else {
            panic!("expected creation")
        };
        assert_eq!(
            parsed.facts,
            OfferFacts {
                content: singleton("Charger".into(), None, None),
                ..Default::default()
            }
        );
    }
    for (event_type, field) in [
        ("OfferPricingChanged.v1", "pricing"),
        ("OfferAvailabilityChanged.v1", "availability"),
        ("OfferItemConditionChanged.v1", "item_condition"),
        ("OfferContentChanged.v1", "content"),
    ] {
        let mut wire = json!({"type":event_type,"data":{"id":"offer-1"}});
        assert!(decode_json_value::<OfferEventRecord>(wire.clone()).is_err());
        wire["data"][field] = Value::Null;
        assert_eq!(
            decode_json_value::<OfferEventRecord>(wire).is_ok(),
            matches!(field, "availability" | "item_condition")
        );
    }
}

fn state_snapshot(state: &RetailerOfferState) -> Value {
    json!({
        "id":state.id(), "retailer_code":state.retailer_code(), "url":state.url(),
        "title":state.title(), "price":state.price(), "images":state.images(),
        "created_at":state.created_at(), "scraped_at":state.scraped_at(),
        "identifiers":state.identifiers, "localization":state.localization, "facts":state.facts
    })
}

fn populated_offer() -> RetailerOfferAggregate {
    let observation = OfferObservation {
        identifiers: decode_json_value(json!({
            "gtin":"0012345678905", "sku":" SKU ž ",
            "external_identifiers":{"shopify_handle":"charger"}
        }))
        .unwrap(),
        localization: Some(
            decode_json_value(json!({
                "target_market":{"kind":"country","code":"US"},"language":"en"
            }))
            .unwrap(),
        ),
        scraped_at: Some("2026-10-27T12:00:00Z".parse().unwrap()),
        ..observed()
    };
    RetailerOfferAggregate::create_with_observation(
        RetailerOfferId::new("offer-1"),
        RetailerCode::AnkerCom,
        "Charger".into(),
        Money::new(1999, Currency::USD),
        "https://example.com/charger".into(),
        vec!["https://example.com/image.png".into()],
        observation,
    )
    .unwrap()
}

fn assert_exact_record_replay(offer: &RetailerOfferAggregate) {
    use crate::infra::records::OfferEventRecord;
    let history = offer
        .uncommitted_events
        .iter()
        .map(|envelope| {
            let original = to_value(envelope.event()).unwrap();
            let record = OfferEventRecord::from(envelope.event());
            let stored = to_string(&record).unwrap();
            let restored: OfferEventRecord = from_str(&stored).unwrap();
            let event: RetailerOfferEvent = restored.into();
            assert_eq!(to_value(&event).unwrap(), original);
            EventEnvelope::new(event, envelope.sequence())
        })
        .collect();
    let restored = RetailerOfferAggregate::from_history(offer.state.id().clone(), history);
    assert_eq!(restored.state, offer.state);
    assert_eq!(restored.sequence, offer.sequence);
    assert!(restored.uncommitted_events.is_empty());
}

fn field_observation(field: &str, value: Value) -> OfferObservation {
    let mut observation = OfferObservation::default();
    match field {
        "availability" => {
            observation.availability = Observation::Observed(decode_json_value(value).unwrap())
        }
        "item_condition" => {
            observation.item_condition = Observation::Observed(decode_json_value(value).unwrap())
        }
        "content" => observation.content = Some(decode_json_value(value).unwrap()),
        _ => panic!("unsupported test fact"),
    }
    observation
}

#[test]
fn independent_transitions_preserve_neighbors_and_exact_historical_values() {
    let mut offer = populated_offer();
    let cases = [
        (
            "availability",
            "OfferAvailabilityChanged",
            json!("unavailable"),
        ),
        (
            "availability",
            "OfferAvailabilityChanged",
            json!("in_stock"),
        ),
        (
            "availability",
            "OfferAvailabilityChanged",
            json!("out_of_stock"),
        ),
        (
            "availability",
            "OfferAvailabilityChanged",
            json!("sold_out"),
        ),
        (
            "availability",
            "OfferAvailabilityChanged",
            json!({"other":"http://schema.org/OutOfStock"}),
        ),
        ("availability", "OfferAvailabilityChanged", json!(null)),
        (
            "availability",
            "OfferAvailabilityChanged",
            json!("available"),
        ),
        (
            "item_condition",
            "OfferItemConditionChanged",
            json!({"other":" Refurbished ž "}),
        ),
        (
            "item_condition",
            "OfferItemConditionChanged",
            json!({"other":"http://schema.org/NewCondition"}),
        ),
        ("item_condition", "OfferItemConditionChanged", json!("new")),
        ("item_condition", "OfferItemConditionChanged", json!(null)),
        ("item_condition", "OfferItemConditionChanged", json!("new")),
        ("content", "OfferContentsChanged", json!({"description":""})),
        (
            "content",
            "OfferContentsChanged",
            json!({"description":" \t\n "}),
        ),
        (
            "content",
            "OfferContentsChanged",
            json!({"description":" <p>ž &amp; source</p>\n"}),
        ),
        (
            "content",
            "OfferContentsChanged",
            json!({"description":null}),
        ),
        (
            "content",
            "OfferContentsChanged",
            json!({"description":"<script>source text</script>"}),
        ),
    ];
    for (field, event_type, value) in cases {
        let mut expected = state_snapshot(&offer.state);
        if field == "content" {
            expected["facts"]["content"][0]["description"] = value["description"].clone();
        } else {
            expected["facts"][field] = value.clone();
        }
        let before = offer.sequence;
        let observation = field_observation(field, value.clone());
        update(&mut offer, 1999, observation).unwrap();
        assert_eq!(offer.sequence, before + 1, "{field}");
        assert_eq!(state_snapshot(&offer.state), expected, "{field}");
        let event = offer.uncommitted_events.last().unwrap().event();
        assert_eq!(event.event_type(), event_type);
        assert_eq!(
            to_value(event).unwrap(),
            if field == "content" {
                json!({"id":"offer-1","content":expected["facts"]["content"],"language":"en"})
            } else {
                json!({"id":"offer-1",field:value})
            }
        );
        let same_observation = field_observation(field, value);
        update(&mut offer, 1999, same_observation).unwrap();
        update(&mut offer, 1999, OfferObservation::default()).unwrap();
        assert_eq!(offer.sequence, before + 1);
        assert_eq!(state_snapshot(&offer.state), expected);
        assert_exact_record_replay(&offer);
    }
}

#[test]
fn comparison_only_and_combined_pricing_changes_keep_exact_large_amounts() {
    let mut offer = populated_offer();
    let mut expected = state_snapshot(&offer.state);
    for (price, comparison) in [
        (
            Money::new(1999, Currency::USD),
            Some(Money::new(3999, Currency::USD)),
        ),
        (
            Money::new(i64::MAX - 1, Currency::JPY),
            Some(Money::new(i64::MAX, Currency::JPY)),
        ),
        (
            Money::new(i64::MAX - 2, Currency::JPY),
            Some(Money::new(i64::MAX - 1, Currency::JPY)),
        ),
        (Money::new(i64::MAX - 2, Currency::JPY), None),
    ] {
        let before = offer.sequence;
        let observation = OfferObservation {
            was_price: Observation::Observed(comparison.clone()),
            ..Default::default()
        };
        offer
            .update_with_observation("Charger".into(), price.clone(), vec![], observation)
            .unwrap();
        expected["price"] = to_value(&price).unwrap();
        expected["facts"]["was_price"] = to_value(&comparison).unwrap();
        assert_eq!(state_snapshot(&offer.state), expected);
        assert_eq!(offer.sequence, before + 1);
        let event = offer.uncommitted_events.last().unwrap().event();
        assert_eq!(event.event_type(), "OfferPricingChanged");
        assert_eq!(
            to_value(event).unwrap(),
            json!({
                "id":"offer-1", "pricing":{"price":price,"was_price":comparison}
            })
        );
        offer
            .update_with_observation("Charger".into(), price, vec![], OfferObservation::default())
            .unwrap();
        assert_eq!(offer.sequence, before + 1);
        assert_exact_record_replay(&offer);
    }
}

#[test]
fn invalid_fact_rejects_the_whole_observation_before_any_unrelated_change() {
    let mut offer = populated_offer();
    let expected = state_snapshot(&offer.state);
    let before = offer.sequence;
    for (field, value) in [
        ("availability", json!({"other":" \t"})),
        ("item_condition", json!({"other":"invalid\0condition"})),
        ("content", json!({"description":"invalid\0description"})),
    ] {
        let observation = field_observation(field, value);
        assert!(
            offer
                .update_with_observation(
                    "Changed title".into(),
                    Money::new(999, Currency::USD),
                    vec!["https://example.com/second.png".into()],
                    observation,
                )
                .is_err()
        );
        assert_eq!(state_snapshot(&offer.state), expected);
        assert_eq!(offer.sequence, before);
        assert_eq!(offer.uncommitted_events.len() as u64, before);
    }
}
