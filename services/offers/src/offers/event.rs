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
    OfferIdentifiersChanged(OfferIdentifiersChangedEvent),
    OfferLocalizationChanged(OfferLocalizationChangedEvent),
    OfferImageAdded(OfferImageAddedEvent),
    OfferTitleChanged(OfferTitleChangedEvent),
    OfferPriceChanged(OfferPriceChangedEvent),
    OfferScrapedAtUpdated(OfferScrapedAtUpdatedEvent),
}

impl ApplicationEvent for RetailerOfferEvent {
    fn event_type(&self) -> &'static str {
        match self {
            Self::OfferCreated(event) => event.event_type(),
            Self::OfferIdentifiersChanged(event) => event.event_type(),
            Self::OfferLocalizationChanged(event) => event.event_type(),
            Self::OfferImageAdded(event) => event.event_type(),
            Self::OfferTitleChanged(event) => event.event_type(),
            Self::OfferPriceChanged(event) => event.event_type(),
            Self::OfferScrapedAtUpdated(event) => event.event_type(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct OfferCreatedEvent {
    #[serde(flatten)]
    pub identifiers: retailer_sauce::identifier::ProductIdentifiers,
    #[serde(default)]
    pub localization: retailer_sauce::localization::Localization,
    #[serde(default)]
    pub scraped_at: Option<chrono::DateTime<chrono::Utc>>,
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
            Self::OfferIdentifiersChanged(event) => event.id.to_string(),
            Self::OfferLocalizationChanged(event) => event.id.to_string(),
            Self::OfferImageAdded(event) => event.id.to_string(),
            Self::OfferTitleChanged(event) => event.id.to_string(),
            Self::OfferPriceChanged(event) => event.id.to_string(),
            Self::OfferScrapedAtUpdated(event) => event.id.to_string(),
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

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct OfferScrapedAtUpdatedEvent {
    pub id: RetailerOfferId,
    pub scraped_at: DateTime<Utc>,
}
pub const OFFER_SCRAPED_AT_UPDATED_EVENT: &str = "OfferScrapedAtUpdated";
impl ApplicationEvent for OfferScrapedAtUpdatedEvent {
    fn event_type(&self) -> &'static str {
        OFFER_SCRAPED_AT_UPDATED_EVENT
    }
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

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct OfferLocalizationChangedEvent {
    pub id: RetailerOfferId,
    pub localization: retailer_sauce::localization::Localization,
}
pub const OFFER_LOCALIZATION_CHANGED_EVENT: &str = "OfferLocalizationChanged";
impl ApplicationEvent for OfferLocalizationChangedEvent {
    fn event_type(&self) -> &'static str {
        OFFER_LOCALIZATION_CHANGED_EVENT
    }
}

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct OfferIdentifiersChangedEvent {
    pub id: RetailerOfferId,
    #[serde(
        flatten,
        deserialize_with = "retailer_sauce::identifier::complete_identifiers"
    )]
    pub identifiers: retailer_sauce::identifier::ProductIdentifiers,
}
pub const OFFER_IDENTIFIERS_CHANGED_EVENT: &str = "OfferIdentifiersChanged";
impl ApplicationEvent for OfferIdentifiersChangedEvent {
    fn event_type(&self) -> &'static str {
        OFFER_IDENTIFIERS_CHANGED_EVENT
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identifiers_do_not_capture_other_untagged_events_or_accept_partial_sets() {
        let id = "00000000-0000-0000-0000-000000000001";
        for (fields, kind) in [
            (
                serde_json::json!({"title":"changed"}),
                OFFER_TITLE_CHANGED_EVENT,
            ),
            (
                serde_json::json!({"localization":{"target_market":null,"language":null}}),
                OFFER_LOCALIZATION_CHANGED_EVENT,
            ),
            (
                serde_json::json!({"scraped_at":"2026-10-26T12:00:00Z"}),
                OFFER_SCRAPED_AT_UPDATED_EVENT,
            ),
            (
                serde_json::json!({"gtin":null,"sku":null,"external_identifiers":{}}),
                OFFER_IDENTIFIERS_CHANGED_EVENT,
            ),
        ] {
            let mut payload = fields;
            payload["id"] = serde_json::json!(id);
            let event: RetailerOfferEvent = serde_json::from_value(payload).unwrap();
            assert_eq!(event.event_type(), kind);
        }
        assert!(
            serde_json::from_value::<OfferIdentifiersChangedEvent>(
                serde_json::json!({"id":id,"sku":"partial"})
            )
            .is_err()
        );
    }
}
