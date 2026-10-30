use crate::offers::event::OfferIdentifiersChangedEvent;
use serde::{Deserialize, Serialize};
use testimony::prelude::HasAggregateId;

use crate::offers::event::{
    OfferCreatedEvent, OfferImageAddedEvent, OfferLocalizationChangedEvent, OfferPriceChangedEvent,
    OfferScrapedAtUpdatedEvent, OfferTitleChangedEvent, RetailerOfferEvent,
};

#[derive(Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum OfferEventRecord {
    #[serde(rename = "OfferContentsChanged.v1")]
    ContentsChanged(crate::offers::event::OfferContentsChangedEvent),
    #[serde(rename = "OfferContentChanged.v1")]
    ContentChanged(crate::offers::event::OfferContentChangedEvent),
    #[serde(rename = "OfferItemConditionChanged.v1")]
    ItemConditionChanged(crate::offers::event::OfferItemConditionChangedEvent),
    #[serde(rename = "OfferAvailabilityChanged.v1")]
    AvailabilityChanged(crate::offers::event::OfferAvailabilityChangedEvent),
    #[serde(rename = "OfferPricingChanged.v1")]
    PricingChanged(crate::offers::event::OfferPricingChangedEvent),
    #[serde(
        rename = "OfferCreated.v8",
        alias = "OfferCreated.v7",
        alias = "OfferCreated.v6",
        alias = "OfferCreated.v5",
        alias = "OfferCreated.v4",
        alias = "OfferCreated.v3",
        alias = "OfferCreated.v1",
        alias = "OfferCreated.v2"
    )]
    LegacyCreated(Box<crate::offers::event::LegacyOfferCreatedEvent>),
    #[serde(rename = "OfferCreated.v9")]
    LegacyRegionCreated(
        #[serde(deserialize_with = "crate::offers::event::current_creation")]
        Box<OfferCreatedEvent>,
    ),
    #[serde(rename = "OfferCreated.v10")]
    Created(
        #[serde(deserialize_with = "crate::offers::event::current_creation")]
        Box<OfferCreatedEvent>,
    ),
    #[serde(rename = "OfferIdentifiersChanged.v1")]
    IdentifiersChanged(OfferIdentifiersChangedEvent),
    #[serde(rename = "OfferLocalizationChanged.v1")]
    LegacyLocalizationChanged(OfferLocalizationChangedEvent),
    #[serde(rename = "OfferLocalizationChanged.v2")]
    LocalizationChanged(OfferLocalizationChangedEvent),
    #[serde(rename = "OfferImageAdded.v1")]
    ImageAdded(OfferImageAddedEvent),
    #[serde(rename = "OfferTitleChanged.v1")]
    TitleChanged(OfferTitleChangedEvent),
    #[serde(rename = "OfferPriceChanged.v1")]
    PriceChanged(OfferPriceChangedEvent),
    #[serde(rename = "OfferScrapedAtUpdated.v1")]
    ScrapedAtUpdated(OfferScrapedAtUpdatedEvent),
}

impl From<&RetailerOfferEvent> for OfferEventRecord {
    fn from(event: &RetailerOfferEvent) -> Self {
        match event {
            RetailerOfferEvent::OfferIdentifiersChanged(event) => {
                Self::IdentifiersChanged(event.clone())
            }
            RetailerOfferEvent::OfferLocalizationChanged(event) => {
                let mut event = event.clone();
                event.localization = event.localization.canonicalized();
                Self::LocalizationChanged(event)
            }
            RetailerOfferEvent::OfferScrapedAtUpdated(event) => {
                Self::ScrapedAtUpdated(event.clone())
            }
            RetailerOfferEvent::OfferImageAdded(event) => Self::ImageAdded(event.clone()),
            RetailerOfferEvent::OfferPricingChanged(event) => Self::PricingChanged(event.clone()),
            RetailerOfferEvent::OfferAvailabilityChanged(event) => {
                Self::AvailabilityChanged(event.clone())
            }
            RetailerOfferEvent::OfferItemConditionChanged(event) => {
                Self::ItemConditionChanged(event.clone())
            }
            RetailerOfferEvent::OfferContentChanged(event) => Self::ContentChanged(event.clone()),
            RetailerOfferEvent::OfferContentsChanged(event) => Self::ContentsChanged(event.clone()),
            RetailerOfferEvent::OfferCreated(event) => {
                let mut event = event.clone();
                event.localization = event.localization.clone().canonicalized();
                Self::Created(event)
            }
            RetailerOfferEvent::OfferTitleChanged(event) => Self::TitleChanged(event.clone()),
            RetailerOfferEvent::OfferPriceChanged(event) => Self::PriceChanged(event.clone()),
        }
    }
}

impl From<OfferEventRecord> for RetailerOfferEvent {
    fn from(record: OfferEventRecord) -> Self {
        match record {
            OfferEventRecord::IdentifiersChanged(event) => Self::OfferIdentifiersChanged(event),
            OfferEventRecord::LegacyLocalizationChanged(event)
            | OfferEventRecord::LocalizationChanged(event) => Self::OfferLocalizationChanged(event),
            OfferEventRecord::ScrapedAtUpdated(event) => Self::OfferScrapedAtUpdated(event),
            OfferEventRecord::ImageAdded(event) => Self::OfferImageAdded(event),
            OfferEventRecord::PricingChanged(event) => Self::OfferPricingChanged(event),
            OfferEventRecord::AvailabilityChanged(event) => Self::OfferAvailabilityChanged(event),
            OfferEventRecord::ItemConditionChanged(event) => Self::OfferItemConditionChanged(event),
            OfferEventRecord::ContentChanged(event) => Self::OfferContentChanged(event),
            OfferEventRecord::LegacyRegionCreated(event) | OfferEventRecord::Created(event) => {
                Self::OfferCreated(event)
            }
            OfferEventRecord::LegacyCreated(event) => Self::OfferCreated(Box::new((*event).into())),
            OfferEventRecord::ContentsChanged(event) => Self::OfferContentsChanged(event),
            OfferEventRecord::TitleChanged(event) => Self::OfferTitleChanged(event),
            OfferEventRecord::PriceChanged(event) => Self::OfferPriceChanged(event),
        }
    }
}

impl HasAggregateId for OfferEventRecord {
    fn aggregate_id(&self) -> String {
        match self {
            Self::IdentifiersChanged(event) => event.id.to_string(),
            Self::LegacyLocalizationChanged(event) | Self::LocalizationChanged(event) => {
                event.id.to_string()
            }
            Self::ScrapedAtUpdated(event) => event.id.to_string(),
            Self::ImageAdded(event) => event.id.to_string(),
            Self::PricingChanged(event) => event.id.to_string(),
            Self::AvailabilityChanged(event) => event.id.to_string(),
            Self::ItemConditionChanged(event) => event.id.to_string(),
            Self::ContentChanged(event) => event.id.to_string(),
            Self::LegacyRegionCreated(event) | Self::Created(event) => event.id.to_string(),
            Self::LegacyCreated(event) => event.id.to_string(),
            Self::ContentsChanged(event) => event.id.to_string(),
            Self::TitleChanged(event) => event.id.to_string(),
            Self::PriceChanged(event) => event.id.to_string(),
        }
    }
}

#[cfg(test)]
mod regional_versions {
    use super::*;
    use serde_json::json;

    #[test]
    fn historical_region_records_keep_their_encoding_and_new_records_are_versioned() {
        let localization =
            json!({"target_market":{"kind":"region","label":"European Union"},"language":"en"});
        let change = json!({"type":"OfferLocalizationChanged.v1","data":{"id":"example","localization":localization}});
        let old: OfferEventRecord = serde_json::from_value(change.clone()).unwrap();
        assert_eq!(serde_json::to_value(&old).unwrap(), change);
        let mut event: RetailerOfferEvent = old.into();
        if let RetailerOfferEvent::OfferLocalizationChanged(value) = &mut event {
            value.localization = value.localization.clone().canonicalized();
        }
        let new = serde_json::to_value(OfferEventRecord::from(&event)).unwrap();
        assert_eq!(new["type"], "OfferLocalizationChanged.v2");
        assert_eq!(
            new["data"]["localization"]["target_market"],
            json!({"kind":"region","code":"eu"})
        );
        let created = json!({"type":"OfferCreated.v9","data":{
            "id":"example","retailer_code":"AnkerAu","title":"Charger","url":"https://example.com/a",
            "price":{"amount":100,"currency":"AUD"},"created_at":"2026-10-29T00:00:00Z",
            "localization":localization,"content":[{"language":"en","title":"Charger","description":null}]
        }});
        let old: OfferEventRecord = serde_json::from_value(created).unwrap();
        let old_json = serde_json::to_value(&old).unwrap();
        assert_eq!(old_json["type"], "OfferCreated.v9");
        assert_eq!(
            old_json["data"]["localization"]["target_market"]["label"],
            "European Union"
        );
        let event: RetailerOfferEvent = old.into();
        assert_eq!(
            serde_json::to_value(OfferEventRecord::from(&event)).unwrap()["type"],
            "OfferCreated.v10"
        );
    }
}
