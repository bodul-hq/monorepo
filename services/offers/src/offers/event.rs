use chrono::DateTime as ChronoDateTime;
use chrono::Utc as ChronoUtc;
use chrono::{DateTime, Utc};
use kernel::ApplicationEvent;
use money::Money;
use retailer_sauce::availability::Availability;
use retailer_sauce::identifier::ProductIdentifiers;
use retailer_sauce::identifier::complete_identifiers as retailer_sauce_identifier_complete_identifiers;
use retailer_sauce::item_condition::ItemCondition;
use retailer_sauce::localization::Localization;
use retailer_sauce::observation::nullable_object as retailer_sauce_observation_nullable_object;
use retailer_sauce::observation::object as retailer_sauce_observation_object;
use retailer_sauce::observation::required_nullable as retailer_sauce_observation_required_nullable;
use retailer_sauce::offer_content::Content as OfferContentContent;
use retailer_sauce::offer_content::guard as offer_content_guard;
use retailer_sauce::offer_content::selected as offer_content_selected;
use retailer_sauce::offer_content::singleton;
use retailer_sauce::offer_content::sort;
use retailer_sauce::structuralized::content::Content as ContentContent;
use serde::Deserialize;
use serde::Deserializer;
use serde::Serialize;
use shared::images::filter;
use shared::retailer::RetailerCode;
use testimony::prelude::HasAggregateId;

use super::retailer_offer_id::RetailerOfferId;

#[derive(Clone, Serialize, Deserialize)]
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

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(from = "OfferCreatedWire")]
pub struct OfferCreatedEvent {
    #[serde(flatten)]
    pub facts: OfferFacts,
    #[serde(flatten)]
    pub identifiers: ProductIdentifiers,
    #[serde(default)]
    pub localization: Localization,
    #[serde(default)]
    pub scraped_at: Option<ChronoDateTime<ChronoUtc>>,
    #[serde(default)]
    pub images: Vec<String>,
    pub id: RetailerOfferId,
    pub retailer_code: RetailerCode,
    pub title: String,
    pub price: Money,
    pub url: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum OfferCreatedWire {
    Current(CurrentOfferCreatedEvent),
    Legacy(LegacyOfferCreatedEvent),
}

#[derive(Deserialize)]
struct CurrentOfferCreatedEvent {
    #[serde(flatten)]
    pub facts: OfferFacts,
    #[serde(flatten)]
    pub identifiers: ProductIdentifiers,
    #[serde(default)]
    pub localization: Localization,
    #[serde(default)]
    pub scraped_at: Option<ChronoDateTime<ChronoUtc>>,
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
#[derive(Clone, Serialize, Deserialize)]
pub struct LegacyOfferCreatedEvent {
    #[serde(flatten)]
    pub facts: LegacyOfferFacts,
    #[serde(flatten)]
    pub identifiers: ProductIdentifiers,
    #[serde(default)]
    pub localization: Localization,
    #[serde(default)]
    pub scraped_at: Option<ChronoDateTime<ChronoUtc>>,
    #[serde(default)]
    pub images: Vec<String>,
    pub id: RetailerOfferId,
    pub retailer_code: RetailerCode,
    pub title: String,
    pub price: Money,
    pub url: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct LegacyOfferFacts {
    #[serde(
        default,
        deserialize_with = "retailer_sauce_observation_nullable_object"
    )]
    pub was_price: Option<Money>,
    #[serde(default)]
    pub availability: Option<Availability>,
    #[serde(default)]
    pub item_condition: Option<ItemCondition>,
    #[serde(default)]
    pub content: ContentContent,
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
        sort(&mut value.facts.content);
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
        let content = singleton(
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
        offer_content_guard(&self.facts.content, &self.localization.language)?;
        if offer_content_selected(&self.facts.content, &self.localization.language)?.title
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
    D: Deserializer<'de>,
{
    let value = <CurrentOfferCreatedEvent as Deserialize>::deserialize(deserializer)?;
    Ok(Box::new(value.into()))
}

#[derive(Clone, Serialize, Deserialize)]
pub struct OfferTitleChangedEvent {
    pub id: RetailerOfferId,
    pub title: String,
}

#[derive(Clone, Serialize, Deserialize)]
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

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
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
    let (filtered, skipped) = filter(images.to_vec());
    if skipped != 0 || filtered != images {
        return Err(err_invalid_images());
    }
    Ok(())
}

fn err_invalid_images() -> String {
    "creation images must be valid, unique URLs within position capacity".into()
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
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
#[derive(Serialize)]
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

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct OfferLocalizationChangedEvent {
    pub id: RetailerOfferId,
    pub localization: Localization,
}
pub const OFFER_LOCALIZATION_CHANGED_EVENT: &str = "OfferLocalizationChanged";
impl ApplicationEvent for OfferLocalizationChangedEvent {
    fn event_type(&self) -> &'static str {
        OFFER_LOCALIZATION_CHANGED_EVENT
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct OfferIdentifiersChangedEvent {
    pub id: RetailerOfferId,
    #[serde(
        flatten,
        deserialize_with = "retailer_sauce_identifier_complete_identifiers"
    )]
    pub identifiers: ProductIdentifiers,
}
pub const OFFER_IDENTIFIERS_CHANGED_EVENT: &str = "OfferIdentifiersChanged";
impl ApplicationEvent for OfferIdentifiersChangedEvent {
    fn event_type(&self) -> &'static str {
        OFFER_IDENTIFIERS_CHANGED_EVENT
    }
}

#[cfg(test)]
mod tests {
    use serde_json::from_value as decode_json_value;

    use serde_json::json;

    use super::*;

    #[test]
    fn identifiers_do_not_capture_other_untagged_events_or_accept_partial_sets() {
        let id = "00000000-0000-0000-0000-000000000001";
        for (fields, kind) in [
            (json!({"title":"changed"}), OFFER_TITLE_CHANGED_EVENT),
            (
                json!({"localization":{"target_market":null,"language":null}}),
                OFFER_LOCALIZATION_CHANGED_EVENT,
            ),
            (
                json!({"scraped_at":"2026-10-26T12:00:00Z"}),
                OFFER_SCRAPED_AT_UPDATED_EVENT,
            ),
            (
                json!({"gtin":null,"sku":null,"external_identifiers":{}}),
                OFFER_IDENTIFIERS_CHANGED_EVENT,
            ),
        ] {
            let mut payload = fields;
            payload["id"] = json!(id);
            let event: RetailerOfferEvent = decode_json_value(payload).unwrap();
            assert_eq!(event.event_type(), kind);
        }
        assert!(
            decode_json_value::<OfferIdentifiersChangedEvent>(json!({"id":id,"sku":"partial"}))
                .is_err()
        );
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct OfferFacts {
    #[serde(
        default,
        deserialize_with = "retailer_sauce_observation_nullable_object"
    )]
    pub was_price: Option<Money>,
    #[serde(default)]
    pub availability: Option<Availability>,
    #[serde(default)]
    pub item_condition: Option<ItemCondition>,
    pub content: Vec<OfferContentContent>,
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

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Pricing {
    #[serde(deserialize_with = "retailer_sauce_observation_object")]
    pub price: Money,
    #[serde(deserialize_with = "retailer_sauce_observation_nullable_object")]
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

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct OfferPricingChangedEvent {
    pub id: RetailerOfferId,
    #[serde(deserialize_with = "retailer_sauce_observation_object")]
    pub pricing: Pricing,
}
pub const OFFER_PRICING_CHANGED_EVENT: &str = "OfferPricingChanged";
impl ApplicationEvent for OfferPricingChangedEvent {
    fn event_type(&self) -> &'static str {
        OFFER_PRICING_CHANGED_EVENT
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct OfferAvailabilityChangedEvent {
    pub id: RetailerOfferId,
    #[serde(deserialize_with = "retailer_sauce_observation_required_nullable")]
    pub availability: Option<Availability>,
}
pub const OFFER_AVAILABILITY_CHANGED_EVENT: &str = "OfferAvailabilityChanged";
impl ApplicationEvent for OfferAvailabilityChangedEvent {
    fn event_type(&self) -> &'static str {
        OFFER_AVAILABILITY_CHANGED_EVENT
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct OfferItemConditionChangedEvent {
    pub id: RetailerOfferId,
    #[serde(deserialize_with = "retailer_sauce_observation_required_nullable")]
    pub item_condition: Option<ItemCondition>,
}
pub const OFFER_ITEM_CONDITION_CHANGED_EVENT: &str = "OfferItemConditionChanged";
impl ApplicationEvent for OfferItemConditionChangedEvent {
    fn event_type(&self) -> &'static str {
        OFFER_ITEM_CONDITION_CHANGED_EVENT
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct OfferContentChangedEvent {
    pub id: RetailerOfferId,
    pub content: ContentContent,
}
pub const OFFER_CONTENT_CHANGED_EVENT: &str = "OfferContentChanged";
impl ApplicationEvent for OfferContentChangedEvent {
    fn event_type(&self) -> &'static str {
        OFFER_CONTENT_CHANGED_EVENT
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct OfferContentsChangedEvent {
    pub id: RetailerOfferId,
    pub content: Vec<OfferContentContent>,
    #[serde(deserialize_with = "retailer_sauce_observation_required_nullable")]
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
        offer_content_guard(&self.content, &self.language)
    }
}
