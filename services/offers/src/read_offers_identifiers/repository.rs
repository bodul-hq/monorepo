mod in_memory;
mod postgres;

pub use in_memory::InMemoryRetailerOfferIdentifiersRepository;
pub use postgres::PostgresRetailerOfferIdentifiersRepository;

use super::entity::RetailerOfferIdentifiers;
use crate::offers::retailer_offer_id::RetailerOfferId;
use retailer_sauce::identifier::ProductIdentifiers;

#[derive(Debug, thiserror::Error)]
#[error("offer identity read model operation failed: {0}")]
pub struct RetailerOfferIdentifiersRepositoryError(String);

/// Inserts a new read model; duplicate IDs or retailer/URL pairs are errors.
pub trait RetailerOfferIdentifiersRepository: Send + Sync {
    fn insert(
        &self,
        entity: &RetailerOfferIdentifiers,
    ) -> Result<(), RetailerOfferIdentifiersRepositoryError>;

    /// Applies the event's complete decided set; fails if the identity is missing.
    fn update_identifiers(
        &self,
        id: &RetailerOfferId,
        identifiers: &ProductIdentifiers,
    ) -> Result<(), RetailerOfferIdentifiersRepositoryError>;
}

fn err_insert(error: impl std::fmt::Display) -> RetailerOfferIdentifiersRepositoryError {
    RetailerOfferIdentifiersRepositoryError(error.to_string())
}

fn err_update(error: impl std::fmt::Display) -> RetailerOfferIdentifiersRepositoryError {
    RetailerOfferIdentifiersRepositoryError(error.to_string())
}

fn err_identity_missing() -> RetailerOfferIdentifiersRepositoryError {
    err_update("offer identity has not been inserted")
}

fn validate_update(
    id: &RetailerOfferId,
    identifiers: &ProductIdentifiers,
) -> Result<uuid::Uuid, RetailerOfferIdentifiersRepositoryError> {
    identifiers.guard().map_err(err_update)?;
    uuid::Uuid::parse_str(id.value()).map_err(err_update)
}

fn validate(
    entity: &RetailerOfferIdentifiers,
) -> Result<uuid::Uuid, RetailerOfferIdentifiersRepositoryError> {
    entity.identifiers.guard().map_err(err_insert)?;
    let id = uuid::Uuid::parse_str(entity.id.value()).map_err(err_insert)?;
    if entity.url.len() > 2048 || entity.url.contains('\0') {
        return Err(err_insert(
            "URL must be at most 2048 bytes and contain no NUL",
        ));
    }
    Ok(id)
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::offers::retailer_offer_id::RetailerOfferId;

    pub(crate) fn assert_insert_contract(repository: &dyn RetailerOfferIdentifiersRepository) {
        let entity = RetailerOfferIdentifiers {
            identifiers: Default::default(),
            id: RetailerOfferId::new(uuid::Uuid::new_v4().to_string()),
            retailer_code: bodul_shared::retailer::RetailerCode::from_str("ankerau").unwrap(),
            url: "https://example.com/Charger?color=red#details".to_owned(),
        };
        repository.insert(&entity).unwrap();
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
        other.url = "https://example.com/invalid-identifiers".into();
        for value in ["", "contains\0nul"] {
            other.identifiers.sku = Some(retailer_sauce::identifier::Sku {
                value: value.into(),
            });
            assert!(repository.insert(&other).is_err());
        }
    }

    #[test]
    fn in_memory_insert_contract() {
        assert_insert_contract(&InMemoryRetailerOfferIdentifiersRepository::new());
    }
}
