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
    Created(
        #[serde(deserialize_with = "crate::offers::event::current_creation")]
        Box<OfferCreatedEvent>,
    ),
    #[serde(rename = "OfferIdentifiersChanged.v1")]
    IdentifiersChanged(OfferIdentifiersChangedEvent),
    #[serde(rename = "OfferLocalizationChanged.v1")]
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
                Self::LocalizationChanged(event.clone())
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
            RetailerOfferEvent::OfferCreated(event) => Self::Created(event.clone()),
            RetailerOfferEvent::OfferTitleChanged(event) => Self::TitleChanged(event.clone()),
            RetailerOfferEvent::OfferPriceChanged(event) => Self::PriceChanged(event.clone()),
        }
    }
}

impl From<OfferEventRecord> for RetailerOfferEvent {
    fn from(record: OfferEventRecord) -> Self {
        match record {
            OfferEventRecord::IdentifiersChanged(event) => Self::OfferIdentifiersChanged(event),
            OfferEventRecord::LocalizationChanged(event) => Self::OfferLocalizationChanged(event),
            OfferEventRecord::ScrapedAtUpdated(event) => Self::OfferScrapedAtUpdated(event),
            OfferEventRecord::ImageAdded(event) => Self::OfferImageAdded(event),
            OfferEventRecord::PricingChanged(event) => Self::OfferPricingChanged(event),
            OfferEventRecord::AvailabilityChanged(event) => Self::OfferAvailabilityChanged(event),
            OfferEventRecord::ItemConditionChanged(event) => Self::OfferItemConditionChanged(event),
            OfferEventRecord::ContentChanged(event) => Self::OfferContentChanged(event),
            OfferEventRecord::Created(event) => Self::OfferCreated(event),
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
            Self::LocalizationChanged(event) => event.id.to_string(),
            Self::ScrapedAtUpdated(event) => event.id.to_string(),
            Self::ImageAdded(event) => event.id.to_string(),
            Self::PricingChanged(event) => event.id.to_string(),
            Self::AvailabilityChanged(event) => event.id.to_string(),
            Self::ItemConditionChanged(event) => event.id.to_string(),
            Self::ContentChanged(event) => event.id.to_string(),
            Self::Created(event) => event.id.to_string(),
            Self::LegacyCreated(event) => event.id.to_string(),
            Self::ContentsChanged(event) => event.id.to_string(),
            Self::TitleChanged(event) => event.id.to_string(),
            Self::PriceChanged(event) => event.id.to_string(),
        }
    }
}
