use bodul_shared::retailer::RetailerCode;
use chrono::{DateTime, Utc};
use kernel::ApplicationEvent;
use money::Money;
use testimony::prelude::HasAggregateId;

use super::retailer_offer_id::RetailerOfferId;

#[derive(Clone, serde::Serialize, serde::Deserialize)]
#[serde(untagged)]
pub enum RetailerOfferEvent {
    OfferCreated(Box<OfferCreatedEvent>),
    OfferPricingChanged(OfferPricingChangedEvent),
    OfferAvailabilityChanged(OfferAvailabilityChangedEvent),
    OfferItemConditionChanged(OfferItemConditionChangedEvent),
    OfferContentChanged(OfferContentChangedEvent),
    OfferContentsChanged(OfferContentsChangedEvent),
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
            Self::OfferPricingChanged(event) => event.event_type(),
            Self::OfferAvailabilityChanged(event) => event.event_type(),
            Self::OfferItemConditionChanged(event) => event.event_type(),
            Self::OfferContentChanged(event) => event.event_type(),
            Self::OfferContentsChanged(event) => event.event_type(),
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
#[serde(from = "OfferCreatedWire")]
pub struct OfferCreatedEvent {
    #[serde(flatten)]
    pub facts: OfferFacts,
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

#[derive(serde::Deserialize)]
#[serde(untagged)]
enum OfferCreatedWire {
    Current(CurrentOfferCreatedEvent),
    Legacy(LegacyOfferCreatedEvent),
}

#[derive(serde::Deserialize)]
struct CurrentOfferCreatedEvent {
    #[serde(flatten)]
    pub facts: OfferFacts,
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

/// Explicit historical DTO: versions 1–8 carried a singular description object.
#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct LegacyOfferCreatedEvent {
    #[serde(flatten)]
    pub facts: LegacyOfferFacts,
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

#[derive(Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct LegacyOfferFacts {
    #[serde(
        default,
        deserialize_with = "retailer_sauce::observation::nullable_object"
    )]
    pub was_price: Option<Money>,
    #[serde(default)]
    pub availability: Option<retailer_sauce::availability::Availability>,
    #[serde(default)]
    pub item_condition: Option<retailer_sauce::item_condition::ItemCondition>,
    #[serde(default)]
    pub content: retailer_sauce::structuralized::content::Content,
}
impl From<OfferCreatedWire> for OfferCreatedEvent {
    fn from(value: OfferCreatedWire) -> Self {
        match value {
            OfferCreatedWire::Current(value) => value.into(),
            OfferCreatedWire::Legacy(value) => value.into(),
        }
    }
}
impl From<CurrentOfferCreatedEvent> for OfferCreatedEvent {
    fn from(mut value: CurrentOfferCreatedEvent) -> Self {
        retailer_sauce::offer_content::sort(&mut value.facts.content);
        Self {
            facts: value.facts,
            identifiers: value.identifiers,
            localization: value.localization,
            scraped_at: value.scraped_at,
            images: value.images,
            id: value.id,
            retailer_code: value.retailer_code,
            title: value.title,
            price: value.price,
            url: value.url,
            created_at: value.created_at,
        }
    }
}
impl From<LegacyOfferCreatedEvent> for OfferCreatedEvent {
    fn from(value: LegacyOfferCreatedEvent) -> Self {
        let content = retailer_sauce::offer_content::singleton(
            value.title.clone(),
            value.localization.language.clone(),
            value.facts.content.description,
        );
        let facts = OfferFacts {
            was_price: value.facts.was_price,
            availability: value.facts.availability,
            item_condition: value.facts.item_condition,
            content,
        };
        Self {
            facts,
            identifiers: value.identifiers,
            localization: value.localization,
            scraped_at: value.scraped_at,
            images: value.images,
            id: value.id,
            retailer_code: value.retailer_code,
            title: value.title,
            price: value.price,
            url: value.url,
            created_at: value.created_at,
        }
    }
}
impl OfferCreatedEvent {
    pub fn guard(&self) -> Result<(), String> {
        self.facts.guard()?;
        self.localization.guard()?;
        self.identifiers.guard()?;
        retailer_sauce::offer_content::guard(&self.facts.content, &self.localization.language)?;
        if retailer_sauce::offer_content::selected(
            &self.facts.content,
            &self.localization.language,
        )?
        .title
            != self.title
        {
            return Err(err_selected_title());
        }
        guard_images(&self.images)
    }
}
fn err_selected_title() -> String {
    "title must match selected content".into()
}
pub(crate) fn current_creation<'de, D>(deserializer: D) -> Result<Box<OfferCreatedEvent>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value = <CurrentOfferCreatedEvent as serde::Deserialize>::deserialize(deserializer)?;
    Ok(Box::new(value.into()))
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
            Self::OfferPricingChanged(event) => event.id.to_string(),
            Self::OfferAvailabilityChanged(event) => event.id.to_string(),
            Self::OfferItemConditionChanged(event) => event.id.to_string(),
            Self::OfferContentChanged(event) => event.id.to_string(),
            Self::OfferContentsChanged(event) => event.id.to_string(),
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

#[derive(Clone, Debug, Default, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct OfferFacts {
    #[serde(
        default,
        deserialize_with = "retailer_sauce::observation::nullable_object"
    )]
    pub was_price: Option<Money>,
    #[serde(default)]
    pub availability: Option<retailer_sauce::availability::Availability>,
    #[serde(default)]
    pub item_condition: Option<retailer_sauce::item_condition::ItemCondition>,
    pub content: Vec<retailer_sauce::offer_content::Content>,
}
impl OfferFacts {
    pub fn guard(&self) -> Result<(), String> {
        if let Some(value) = &self.availability {
            value.guard()?;
        }
        if let Some(value) = &self.item_condition {
            value.guard()?;
        }
        for content in &self.content {
            content.guard()?;
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Pricing {
    #[serde(deserialize_with = "retailer_sauce::observation::object")]
    pub price: Money,
    #[serde(deserialize_with = "retailer_sauce::observation::nullable_object")]
    pub was_price: Option<Money>,
}
impl Pricing {
    pub fn guard(&self) -> Result<(), String> {
        guard_was_price(&self.price, self.was_price.as_ref())
    }
}

pub fn guard_was_price(price: &Money, was_price: Option<&Money>) -> Result<(), String> {
    if was_price
        .is_some_and(|was| was.currency() != price.currency() || was.amount() <= price.amount())
    {
        return Err(err_invalid_was_price());
    }
    Ok(())
}
fn err_invalid_was_price() -> String {
    "was_price must use the current currency and be greater than price".into()
}

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct OfferPricingChangedEvent {
    pub id: RetailerOfferId,
    #[serde(deserialize_with = "retailer_sauce::observation::object")]
    pub pricing: Pricing,
}
pub const OFFER_PRICING_CHANGED_EVENT: &str = "OfferPricingChanged";
impl ApplicationEvent for OfferPricingChangedEvent {
    fn event_type(&self) -> &'static str {
        OFFER_PRICING_CHANGED_EVENT
    }
}

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct OfferAvailabilityChangedEvent {
    pub id: RetailerOfferId,
    #[serde(deserialize_with = "retailer_sauce::observation::required_nullable")]
    pub availability: Option<retailer_sauce::availability::Availability>,
}
pub const OFFER_AVAILABILITY_CHANGED_EVENT: &str = "OfferAvailabilityChanged";
impl ApplicationEvent for OfferAvailabilityChangedEvent {
    fn event_type(&self) -> &'static str {
        OFFER_AVAILABILITY_CHANGED_EVENT
    }
}

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct OfferItemConditionChangedEvent {
    pub id: RetailerOfferId,
    #[serde(deserialize_with = "retailer_sauce::observation::required_nullable")]
    pub item_condition: Option<retailer_sauce::item_condition::ItemCondition>,
}
pub const OFFER_ITEM_CONDITION_CHANGED_EVENT: &str = "OfferItemConditionChanged";
impl ApplicationEvent for OfferItemConditionChangedEvent {
    fn event_type(&self) -> &'static str {
        OFFER_ITEM_CONDITION_CHANGED_EVENT
    }
}

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct OfferContentChangedEvent {
    pub id: RetailerOfferId,
    pub content: retailer_sauce::structuralized::content::Content,
}
pub const OFFER_CONTENT_CHANGED_EVENT: &str = "OfferContentChanged";
impl ApplicationEvent for OfferContentChangedEvent {
    fn event_type(&self) -> &'static str {
        OFFER_CONTENT_CHANGED_EVENT
    }
}

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct OfferContentsChangedEvent {
    pub id: RetailerOfferId,
    pub content: Vec<retailer_sauce::offer_content::Content>,
    #[serde(deserialize_with = "retailer_sauce::observation::required_nullable")]
    pub language: Option<String>,
}
pub const OFFER_CONTENTS_CHANGED_EVENT: &str = "OfferContentsChanged";
impl ApplicationEvent for OfferContentsChangedEvent {
    fn event_type(&self) -> &'static str {
        OFFER_CONTENTS_CHANGED_EVENT
    }
}
impl OfferContentsChangedEvent {
    pub fn guard(&self) -> Result<(), String> {
        retailer_sauce::offer_content::guard(&self.content, &self.language)
    }
}
