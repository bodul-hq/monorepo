use serde::{Deserialize, Serialize};
use testimony::prelude::HasAggregateId;

use crate::offers::event::{
    OfferCreatedEvent, OfferImageAddedEvent, OfferPriceChangedEvent, OfferTitleChangedEvent,
    RetailerOfferEvent,
};

#[derive(Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum OfferEventRecord {
    #[serde(rename = "OfferCreated.v2", alias = "OfferCreated.v1")]
    Created(OfferCreatedEvent),
    #[serde(rename = "OfferImageAdded.v1")]
    ImageAdded(OfferImageAddedEvent),
    #[serde(rename = "OfferTitleChanged.v1")]
    TitleChanged(OfferTitleChangedEvent),
    #[serde(rename = "OfferPriceChanged.v1")]
    PriceChanged(OfferPriceChangedEvent),
}

impl From<&RetailerOfferEvent> for OfferEventRecord {
    fn from(event: &RetailerOfferEvent) -> Self {
        match event {
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
            Self::ImageAdded(event) => event.id.to_string(),
            Self::Created(event) => event.id.to_string(),
            Self::TitleChanged(event) => event.id.to_string(),
            Self::PriceChanged(event) => event.id.to_string(),
        }
    }
}
