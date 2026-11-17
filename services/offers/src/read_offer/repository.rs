mod in_memory;
mod postgres;

use crate::offers::event::OfferImageAddedEvent;
use crate::offers::event::Pricing;
use crate::offers::event::guard_images;
use chrono::DateTime;
use chrono::Utc;
use retailer_sauce::prelude::Availability;
use retailer_sauce::prelude::ItemCondition;
use retailer_sauce::prelude::Localization;
use retailer_sauce::prelude::OfferContent;
use retailer_sauce::prelude::ProductIdentifiers;
use retailer_sauce::prelude::StructuralizedContent;
use retailer_sauce::prelude::offer_content::guard as retailer_sauce_offer_content_guard;
use retailer_sauce::prelude::offer_content::selected as retailer_sauce_offer_content_selected;
use std::fmt::Display;
use uuid::Uuid;

pub use in_memory::InMemoryRetailerOfferRepository;
pub use postgres::PostgresRetailerOfferRepository;

use super::entity::RetailerOffer;
use crate::offers::retailer_offer_id::RetailerOfferId;
use money::Money;

#[derive(Debug, thiserror::Error)]
#[error("offer read model operation failed: {0}")]
pub struct RetailerOfferRepositoryError(String);

/// Inserts a new read model; duplicate IDs or retailer/URL pairs are errors.
pub trait RetailerOfferRepository: Send + Sync {
    fn update_contents(
        &self,
        id: &RetailerOfferId,
        content: &[OfferContent],
        language: &Option<String>,
    ) -> Result<(), RetailerOfferRepositoryError>;
    fn update_content(
        &self,
        id: &RetailerOfferId,
        content: &StructuralizedContent,
    ) -> Result<(), RetailerOfferRepositoryError>;
    fn update_item_condition(
        &self,
        id: &RetailerOfferId,
        item_condition: &Option<ItemCondition>,
    ) -> Result<(), RetailerOfferRepositoryError>;
    fn update_availability(
        &self,
        id: &RetailerOfferId,
        availability: &Option<Availability>,
    ) -> Result<(), RetailerOfferRepositoryError>;
    fn update_pricing(
        &self,
        id: &RetailerOfferId,
        pricing: &Pricing,
    ) -> Result<(), RetailerOfferRepositoryError>;
    fn update_identifiers(
        &self,
        id: &RetailerOfferId,
        identifiers: &ProductIdentifiers,
    ) -> Result<(), RetailerOfferRepositoryError>;
    fn update_localization(
        &self,
        id: &RetailerOfferId,
        localization: &Localization,
    ) -> Result<(), RetailerOfferRepositoryError>;
    fn update_scraped_at(
        &self,
        id: &RetailerOfferId,
        scraped_at: DateTime<Utc>,
    ) -> Result<(), RetailerOfferRepositoryError>;
    fn add_image(&self, event: &OfferImageAddedEvent) -> Result<(), RetailerOfferRepositoryError>;
    fn images(&self, id: &RetailerOfferId) -> Result<Vec<String>, RetailerOfferRepositoryError>;

    fn insert(&self, entity: &RetailerOffer) -> Result<(), RetailerOfferRepositoryError>;
    /// Fails if the offer has not been inserted yet.
    fn update_title(
        &self,
        id: &RetailerOfferId,
        title: &str,
    ) -> Result<(), RetailerOfferRepositoryError>;
    /// Replaces both amount and currency; fails if the offer is missing.
    fn update_price(
        &self,
        id: &RetailerOfferId,
        price: &Money,
    ) -> Result<(), RetailerOfferRepositoryError>;
}

fn err_update(error: impl Display) -> RetailerOfferRepositoryError {
    RetailerOfferRepositoryError(error.to_string())
}

fn err_offer_missing() -> RetailerOfferRepositoryError {
    err_update("offer has not been inserted")
}

fn err_insert(error: impl Display) -> RetailerOfferRepositoryError {
    RetailerOfferRepositoryError(error.to_string())
}

fn validate(entity: &RetailerOffer) -> Result<Uuid, RetailerOfferRepositoryError> {
    entity.facts.guard().map_err(err_insert)?;
    retailer_sauce_offer_content_guard(&entity.facts.content, &entity.localization.language)
        .map_err(err_insert)?;
    let selected =
        retailer_sauce_offer_content_selected(&entity.facts.content, &entity.localization.language)
            .map_err(err_insert)?;
    if selected.title != entity.title {
        return Err(err_insert("title must match selected content"));
    }
    entity.identifiers.guard().map_err(err_insert)?;
    entity.localization.guard().map_err(err_insert)?;
    guard_images(&entity.images).map_err(err_insert)?;
    let id = Uuid::parse_str(entity.id.value()).map_err(err_insert)?;
    if entity.url.len() > 2048 || entity.url.contains('\0') {
        return Err(err_insert(
            "URL must be at most 2048 bytes and contain no NUL",
        ));
    }
    if entity.title.contains('\0') {
        return Err(err_insert("title must contain no NUL"));
    }
    Ok(id)
}

#[cfg(test)]
pub(crate) mod tests {
    use crate::offers::event::OfferFacts;
    use chrono::DateTime as ChronoDateTime;
    use money::Currency;
    use money::Money as MoneyMoney;
    use retailer_sauce::prelude::offer_content::singleton;
    use shared::retailer::RetailerCode;
    use uuid::Uuid as UuidUuid;

    use super::*;
    use crate::offers::retailer_offer_id::RetailerOfferId;

    pub(crate) fn assert_insert_contract(repository: &dyn RetailerOfferRepository) {
        let entity = RetailerOffer {
            facts: OfferFacts {
                content: singleton("Charger".into(), None, None),
                ..Default::default()
            },
            identifiers: Default::default(),
            localization: Default::default(),
            scraped_at: None,
            images: vec![
                "https://shop.example/A".into(),
                "https://shop.example/B".into(),
            ],
            id: RetailerOfferId::new(UuidUuid::new_v4().to_string()),
            retailer_code: RetailerCode::from_str("ankerau").unwrap(),
            url: "https://example.com/Charger?color=red#details".to_owned(),
            title: "Charger".to_owned(),
            price: MoneyMoney::new(1999, Currency::AUD),
            created_at: ChronoDateTime::from_timestamp(1_700_000_000, 123_000_000).unwrap(),
        };
        repository.insert(&entity).unwrap();
        assert_eq!(repository.images(&entity.id).unwrap(), entity.images);
        assert!(repository.insert(&entity).is_err());
        let mut other = entity.clone();
        other.url.push_str("-other");
        assert!(repository.insert(&other).is_err());
        other = entity.clone();
        other.id = RetailerOfferId::new(UuidUuid::new_v4().to_string());
        assert!(repository.insert(&other).is_err());
        other.url = entity.url.to_lowercase();
        repository.insert(&other).unwrap();
        other.id = RetailerOfferId::new("invalid");
        assert!(repository.insert(&other).is_err());
        other.id = RetailerOfferId::new(UuidUuid::new_v4().to_string());
        other.url = "x".repeat(2049);
        assert!(repository.insert(&other).is_err());
    }

    #[test]
    fn in_memory_insert_contract() {
        assert_insert_contract(&InMemoryRetailerOfferRepository::new());
    }
}
