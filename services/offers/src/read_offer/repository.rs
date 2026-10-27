mod in_memory;
mod postgres;

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
        content: &[retailer_sauce::offer_content::Content],
        language: &Option<String>,
    ) -> Result<(), RetailerOfferRepositoryError>;
    fn update_content(
        &self,
        id: &RetailerOfferId,
        content: &retailer_sauce::structuralized::content::Content,
    ) -> Result<(), RetailerOfferRepositoryError>;
    fn update_item_condition(
        &self,
        id: &RetailerOfferId,
        item_condition: &Option<retailer_sauce::item_condition::ItemCondition>,
    ) -> Result<(), RetailerOfferRepositoryError>;
    fn update_availability(
        &self,
        id: &RetailerOfferId,
        availability: &Option<retailer_sauce::availability::Availability>,
    ) -> Result<(), RetailerOfferRepositoryError>;
    fn update_pricing(
        &self,
        id: &RetailerOfferId,
        pricing: &crate::offers::event::Pricing,
    ) -> Result<(), RetailerOfferRepositoryError>;
    fn update_identifiers(
        &self,
        id: &crate::offers::retailer_offer_id::RetailerOfferId,
        identifiers: &retailer_sauce::identifier::ProductIdentifiers,
    ) -> Result<(), RetailerOfferRepositoryError>;
    fn update_localization(
        &self,
        id: &crate::offers::retailer_offer_id::RetailerOfferId,
        localization: &retailer_sauce::localization::Localization,
    ) -> Result<(), RetailerOfferRepositoryError>;
    fn update_scraped_at(
        &self,
        id: &RetailerOfferId,
        scraped_at: chrono::DateTime<chrono::Utc>,
    ) -> Result<(), RetailerOfferRepositoryError>;
    fn add_image(
        &self,
        event: &crate::offers::event::OfferImageAddedEvent,
    ) -> Result<(), RetailerOfferRepositoryError>;
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

fn err_update(error: impl std::fmt::Display) -> RetailerOfferRepositoryError {
    RetailerOfferRepositoryError(error.to_string())
}

fn err_offer_missing() -> RetailerOfferRepositoryError {
    err_update("offer has not been inserted")
}

fn err_insert(error: impl std::fmt::Display) -> RetailerOfferRepositoryError {
    RetailerOfferRepositoryError(error.to_string())
}

fn validate(entity: &RetailerOffer) -> Result<uuid::Uuid, RetailerOfferRepositoryError> {
    entity.facts.guard().map_err(err_insert)?;
    retailer_sauce::offer_content::guard(&entity.facts.content, &entity.localization.language)
        .map_err(err_insert)?;
    let selected = retailer_sauce::offer_content::selected(
        &entity.facts.content,
        &entity.localization.language,
    )
    .map_err(err_insert)?;
    if selected.title != entity.title {
        return Err(err_insert("title must match selected content"));
    }
    entity.identifiers.guard().map_err(err_insert)?;
    entity.localization.guard().map_err(err_insert)?;
    crate::offers::event::guard_images(&entity.images).map_err(err_insert)?;
    let id = uuid::Uuid::parse_str(entity.id.value()).map_err(err_insert)?;
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
    use super::*;
    use crate::offers::retailer_offer_id::RetailerOfferId;

    pub(crate) fn assert_insert_contract(repository: &dyn RetailerOfferRepository) {
        let entity = RetailerOffer {
            facts: crate::offers::event::OfferFacts {
                content: retailer_sauce::offer_content::singleton("Charger".into(), None, None),
                ..Default::default()
            },
            identifiers: Default::default(),
            localization: Default::default(),
            scraped_at: None,
            images: vec![
                "https://shop.example/A".into(),
                "https://shop.example/B".into(),
            ],
            id: RetailerOfferId::new(uuid::Uuid::new_v4().to_string()),
            retailer_code: bodul_shared::retailer::RetailerCode::from_str("ankerau").unwrap(),
            url: "https://example.com/Charger?color=red#details".to_owned(),
            title: "Charger".to_owned(),
            price: money::Money::new(1999, money::Currency::AUD),
            created_at: chrono::DateTime::from_timestamp(1_700_000_000, 123_000_000).unwrap(),
        };
        repository.insert(&entity).unwrap();
        assert_eq!(repository.images(&entity.id).unwrap(), entity.images);
        assert!(repository.insert(&entity).is_err());
        let mut other = entity.clone();
        other.url.push_str("-other");
        assert!(repository.insert(&other).is_err());
        other = entity.clone();
        other.id = RetailerOfferId::new(uuid::Uuid::new_v4().to_string());
        assert!(repository.insert(&other).is_err());
        other.url = entity.url.to_lowercase();
        repository.insert(&other).unwrap();
        other.id = RetailerOfferId::new("invalid");
        assert!(repository.insert(&other).is_err());
        other.id = RetailerOfferId::new(uuid::Uuid::new_v4().to_string());
        other.url = "x".repeat(2049);
        assert!(repository.insert(&other).is_err());
    }

    #[test]
    fn in_memory_insert_contract() {
        assert_insert_contract(&InMemoryRetailerOfferRepository::new());
    }
}
