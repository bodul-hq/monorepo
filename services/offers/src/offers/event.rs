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
    OfferImageAdded(OfferImageAddedEvent),
    OfferTitleChanged(OfferTitleChangedEvent),
    OfferPriceChanged(OfferPriceChangedEvent),
}

impl ApplicationEvent for RetailerOfferEvent {
    fn event_type(&self) -> &'static str {
        match self {
            Self::OfferCreated(event) => event.event_type(),
            Self::OfferImageAdded(event) => event.event_type(),
            Self::OfferTitleChanged(event) => event.event_type(),
            Self::OfferPriceChanged(event) => event.event_type(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct OfferCreatedEvent {
    #[serde(default)]
    pub images: Vec<String>,
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
            Self::OfferImageAdded(event) => event.id.to_string(),
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

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct OfferImageAddedEvent {
    pub id: RetailerOfferId,
    pub image_url: String,
    pub position: u32,
}
pub const OFFER_IMAGE_ADDED_EVENT: &str = "OfferImageAdded";
impl ApplicationEvent for OfferImageAddedEvent {
    fn event_type(&self) -> &'static str {
        OFFER_IMAGE_ADDED_EVENT
    }
}

/// Events contain validated, ordered unique images; filtering happens at import.
pub(crate) fn guard_images(images: &[String]) -> Result<(), String> {
    if images.len() as u64 > u64::from(u32::MAX) + 1 {
        return Err(err_invalid_images());
    }
    let (filtered, skipped) = shared::images::filter(images.to_vec());
    if skipped != 0 || filtered != images {
        return Err(err_invalid_images());
    }
    Ok(())
}

fn err_invalid_images() -> String {
    "creation images must be valid, unique URLs within position capacity".into()
}

/// Preserve the exact Testimony envelope revision without changing the payload.
#[derive(serde::Serialize)]
#[serde(transparent)]
pub struct SequencedOfferEvent {
    pub event: RetailerOfferEvent,
    #[serde(skip)]
    pub sequence: u64,
}
impl ApplicationEvent for SequencedOfferEvent {
    fn event_type(&self) -> &'static str {
        self.event.event_type()
    }
    fn sequence(&self) -> Option<u64> {
        Some(self.sequence)
    }
}
