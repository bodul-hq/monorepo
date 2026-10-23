use bodul_shared::retailer::RetailerCode;
use chrono::{DateTime, Utc};
use kernel::ApplicationEvent;
use money::Money;
use testimony::prelude::HasAggregateId;

use super::retailer_offer_id::RetailerOfferId;

#[derive(Clone, serde::Serialize, serde::Deserialize)]
#[serde(untagged)]
pub enum RetailerOfferEvent {
    OfferCreated(OfferCreatedEvent),
    OfferTitleChanged(OfferTitleChangedEvent),
    OfferPriceChanged(OfferPriceChangedEvent),
}

impl ApplicationEvent for RetailerOfferEvent {
    fn event_type(&self) -> &'static str {
        match self {
            Self::OfferCreated(event) => event.event_type(),
            Self::OfferTitleChanged(event) => event.event_type(),
            Self::OfferPriceChanged(event) => event.event_type(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct OfferCreatedEvent {
    pub id: RetailerOfferId,
    pub retailer_code: RetailerCode,
    pub title: String,
    pub price: Money,
    pub url: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct OfferTitleChangedEvent {
    pub id: RetailerOfferId,
    pub title: String,
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct OfferPriceChangedEvent {
    pub id: RetailerOfferId,
    pub price: Money,
}

impl HasAggregateId for RetailerOfferEvent {
    fn aggregate_id(&self) -> String {
        match self {
            Self::OfferCreated(event) => event.id.to_string(),
            Self::OfferTitleChanged(event) => event.id.to_string(),
            Self::OfferPriceChanged(event) => event.id.to_string(),
        }
    }
}

pub const OFFER_CREATED_EVENT: &str = "OfferCreated";
pub const OFFER_TITLE_CHANGED_EVENT: &str = "OfferTitleChanged";
pub const OFFER_PRICE_CHANGED_EVENT: &str = "OfferPriceChangedEvent";

impl ApplicationEvent for OfferCreatedEvent {
    fn event_type(&self) -> &'static str {
        OFFER_CREATED_EVENT
    }
}
impl ApplicationEvent for OfferTitleChangedEvent {
    fn event_type(&self) -> &'static str {
        OFFER_TITLE_CHANGED_EVENT
    }
}
impl ApplicationEvent for OfferPriceChangedEvent {
    fn event_type(&self) -> &'static str {
        OFFER_PRICE_CHANGED_EVENT
    }
}
