use super::*;
use crate::infra::records::OfferEventRecord;
use kernel::ApplicationEvent;
use money::Currency;
use retailer_sauce::localization::Localization;
use serde_json::json;

fn observe(language: Option<&str>, description: Option<Option<&str>>) -> OfferObservation {
    OfferObservation {
        localization: Some(Localization {
            target_market: None,
            language: language.map(str::to_owned),
        }),
        content: description.map(
            |description| retailer_sauce::structuralized::content::Content {
                description: description.map(str::to_owned),
            },
        ),
        ..Default::default()
    }
}
fn update(offer: &mut RetailerOfferAggregate, title: &str, observation: OfferObservation) {
    offer
        .update_with_observation(
            title.into(),
            Money::new(1999, Currency::USD),
            vec![],
            observation,
        )
        .unwrap();
}
fn assert_replay(offer: &RetailerOfferAggregate) {
    let history = offer
        .uncommitted_events
        .iter()
        .map(|event| {
            let record = serde_json::to_string(&OfferEventRecord::from(event.event())).unwrap();
            let record: OfferEventRecord = serde_json::from_str(&record).unwrap();
            EventEnvelope::new(record.into(), event.sequence())
        })
        .collect();
    let replay = RetailerOfferAggregate::from_history(offer.state.id().clone(), history);
    assert_eq!(replay.state, offer.state);
    assert_eq!(replay.sequence, offer.sequence);
    assert!(replay.uncommitted_events.is_empty());
}

#[test]
fn languages_merge_without_copying_descriptions_and_selection_only_is_an_event() {
    let mut offer = super::tests::aggregate();
    update(
        &mut offer,
        " English ž ",
        observe(Some("en-US"), Some(Some(" <p>Exact</p>\n"))),
    );
    update(&mut offer, "Français", observe(Some("fr"), None));
    assert_eq!(
        offer_content::selected(&offer.state.facts.content, &Some("fr".into()))
            .unwrap()
            .description,
        None
    );
    update(&mut offer, "Français", observe(Some("fr"), Some(Some(""))));
    update(&mut offer, "Français", OfferObservation::default());
    assert_eq!(offer.sequence, 4);
    let content = offer.state.facts.content.clone();
    update(&mut offer, " English ž ", observe(Some("en-US"), None));
    assert_eq!(offer.sequence, 5);
    assert_eq!(offer.state.facts.content, content);
    assert_eq!(
        offer
            .uncommitted_events
            .last()
            .unwrap()
            .event()
            .event_type(),
        "OfferContentsChanged"
    );
    update(&mut offer, "Unknown", observe(None, None));
    assert_eq!(offer.state.facts.content[0].description, None);
    update(&mut offer, "Français", observe(Some("fr"), Some(None)));
    assert_eq!(
        offer_content::selected(&offer.state.facts.content, &Some("fr".into()))
            .unwrap()
            .description,
        None
    );
    assert_eq!(
        offer
            .state
            .facts
            .content
            .iter()
            .map(|value| value.language.as_deref())
            .collect::<Vec<_>>(),
        [None, Some("en-US"), Some("fr")]
    );
    assert_replay(&offer);
}

#[test]
fn combined_market_change_follows_content_and_reserves_capacity_atomically() {
    let mut offer = super::tests::aggregate();
    let observation = || {
        let mut observation = observe(Some("de"), Some(Some("Details")));
        observation.localization.as_mut().unwrap().target_market =
            serde_json::from_value(json!({"kind":"country","code":"DE"})).unwrap();
        observation
    };
    offer.sequence = i64::MAX as u64 - 1;
    let before = format!("{:?}", offer.state);
    assert_eq!(
        offer.update_with_observation(
            "Deutsch".into(),
            Money::new(1999, Currency::USD),
            vec![],
            observation()
        ),
        Err(RetailerOfferError::SequenceOverflow)
    );
    assert_eq!(format!("{:?}", offer.state), before);
    assert_eq!(offer.uncommitted_events.len(), 1);
    offer.sequence = 1;
    update(&mut offer, "Deutsch", observation());
    assert_eq!(
        offer
            .uncommitted_events
            .iter()
            .map(|event| event.event().event_type())
            .collect::<Vec<_>>(),
        [
            "OfferCreated",
            "OfferContentsChanged",
            "OfferLocalizationChanged"
        ]
    );
    assert_replay(&offer);
    for observation in [
        observe(Some("en_US"), None),
        observe(None, Some(Some("bad\0"))),
    ] {
        let before = format!("{:?}", offer.state);
        let sequence = offer.sequence;
        assert!(
            offer
                .update_with_observation(
                    "Changed".into(),
                    Money::new(1999, Currency::USD),
                    vec![],
                    observation
                )
                .is_err()
        );
        assert_eq!(format!("{:?}", offer.state), before);
        assert_eq!(offer.sequence, sequence);
    }
}

#[test]
fn actual_old_creation_records_and_mixed_history_keep_exact_scalar_tuples() {
    let data = json!({"id":"legacy","retailer_code":"AnkerCom","title":" Old ž ","price":{"amount":1999,"currency":"USD"},"url":"https://example.com/legacy","created_at":"2020-01-02T03:04:05Z"});
    for version in 1..=8 {
        let mut historical = data.clone();
        if version >= 5 {
            historical["localization"] = json!({"target_market":null,"language":"en-US"});
        }
        if version == 8 {
            historical["content"] = json!({"description":" <p>old &amp; text</p>\n"});
        }
        let record: OfferEventRecord = serde_json::from_value(
            json!({"type":format!("OfferCreated.v{version}"),"data":historical.clone()}),
        )
        .unwrap();
        let old: OfferCreatedEvent = serde_json::from_value(historical).unwrap();
        assert_eq!(
            old.facts.content,
            offer_content::singleton(
                " Old ž ".into(),
                if version >= 5 {
                    Some("en-US".into())
                } else {
                    None
                },
                if version == 8 {
                    Some(" <p>old &amp; text</p>\n".into())
                } else {
                    None
                }
            )
        );
        let mut offer = RetailerOfferAggregate::from_history(
            RetailerOfferId::new("legacy"),
            vec![EventEnvelope::new(record.into(), 1)],
        );
        update(
            &mut offer,
            "Français",
            observe(Some("fr"), Some(Some("French"))),
        );
        // Stitch the genuinely old creation to newly persisted changes.
        let created = RetailerOfferEvent::OfferCreated(Box::new(old));
        let mut history = vec![EventEnvelope::new(created, 1)];
        history.extend(offer.uncommitted_events.clone());
        for (index, payload) in [
            json!({"type":"OfferTitleChanged.v1","data":{"id":"legacy","title":" Corrected "}}),
            json!({"type":"OfferContentChanged.v1","data":{"id":"legacy","content":{"description":" Exact legacy "}}}),
            json!({"type":"OfferLocalizationChanged.v1","data":{"id":"legacy","localization":{"target_market":null,"language":"en-US"}}}),
        ].into_iter().enumerate() {
            let record: OfferEventRecord = serde_json::from_value(payload).unwrap();
            history.push(EventEnvelope::new(record.into(), index as u64 + 3));
        }
        let replay = RetailerOfferAggregate::from_history(RetailerOfferId::new("legacy"), history);
        let selected =
            offer_content::selected(&replay.state.facts.content, &Some("en-US".into())).unwrap();
        assert_eq!(selected.title, " Corrected ");
        assert_eq!(selected.description.as_deref(), Some(" Exact legacy "));
        assert!(
            !replay
                .state
                .facts
                .content
                .iter()
                .any(|entry| entry.language.as_deref() == Some("fr"))
        );
        assert_eq!(
            replay.state.facts.content.len(),
            if version >= 5 { 1 } else { 2 }
        );
    }
    assert!(
        serde_json::from_value::<OfferEventRecord>(json!({"type":"OfferCreated.v9","data":data}))
            .is_err()
    );
}

#[test]
fn full_snapshot_records_require_complete_valid_lists_before_replay() {
    use crate::offers::event::OfferContentsChangedEvent;
    for value in [
        json!({"id":"offer","content":[]}),
        json!({"id":"offer","content":null,"language":null}),
    ] {
        assert!(serde_json::from_value::<OfferContentsChangedEvent>(value).is_err());
    }
    for (content, language) in [
        (json!([]), json!(null)),
        (
            json!([{"language":null,"title":"T","description":null},{"language":null,"title":"T","description":null}]),
            json!(null),
        ),
        (
            json!([{"language":null,"title":"T","description":null}]),
            json!("en"),
        ),
    ] {
        let event: OfferContentsChangedEvent =
            serde_json::from_value(json!({"id":"offer","content":content,"language":language}))
                .unwrap();
        assert!(event.guard().is_err());
    }
}
