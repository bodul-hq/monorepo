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
    #[serde(
        rename = "OfferCreated.v5",
        alias = "OfferCreated.v4",
        alias = "OfferCreated.v3",
        alias = "OfferCreated.v1",
        alias = "OfferCreated.v2"
    )]
    Created(OfferCreatedEvent),
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
            OfferEventRecord::Created(event) => Self::OfferCreated(event),
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
            Self::Created(event) => event.id.to_string(),
            Self::TitleChanged(event) => event.id.to_string(),
            Self::PriceChanged(event) => event.id.to_string(),
        }
    }
}
