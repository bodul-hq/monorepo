use super::{
    RetailerOfferIdentifiers, RetailerOfferIdentifiersRepository,
    RetailerOfferIdentifiersRepositoryError, err_identity_missing, err_insert, err_update,
    validate, validate_update,
};
use std::{collections::HashMap, sync::Mutex};
use uuid::Uuid;

#[derive(Default)]
pub struct InMemoryRetailerOfferIdentifiersRepository {
    entries: Mutex<HashMap<Uuid, RetailerOfferIdentifiers>>,
}

impl InMemoryRetailerOfferIdentifiersRepository {
    pub fn new() -> Self {
        Self::default()
    }
}

impl RetailerOfferIdentifiersRepository for InMemoryRetailerOfferIdentifiersRepository {
    fn insert(
        &self,
        entity: &RetailerOfferIdentifiers,
    ) -> Result<(), RetailerOfferIdentifiersRepositoryError> {
        let id = validate(entity)?;
        let mut entries = self.entries.lock().map_err(err_insert)?;
        if entries.contains_key(&id)
            || entries.values().any(|existing| {
                existing.retailer_code == entity.retailer_code && existing.url == entity.url
            })
        {
            return Err(err_insert("offer ID or retailer/URL already exists"));
        }
        entries.insert(id, entity.clone());
        Ok(())
    }

    fn update_identifiers(
        &self,
        id: &crate::offers::retailer_offer_id::RetailerOfferId,
        identifiers: &retailer_sauce::identifier::ProductIdentifiers,
    ) -> Result<(), RetailerOfferIdentifiersRepositoryError> {
        let id = validate_update(id, identifiers)?;
        let mut entries = self.entries.lock().map_err(err_update)?;
        let entity = entries.get_mut(&id).ok_or_else(err_identity_missing)?;
        entity.identifiers = identifiers.clone();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::offers::retailer_offer_id::RetailerOfferId;
    use retailer_sauce::identifier::ProductIdentifiers;
    use serde_json::json;

    #[test]
    fn replaces_the_complete_set_and_rejects_invalid_updates_without_changes() {
        let repository = InMemoryRetailerOfferIdentifiersRepository::new();
        let uuid = Uuid::new_v4();
        let id = RetailerOfferId::new(uuid.to_string());
        let initial: ProductIdentifiers = serde_json::from_value(json!({
            "gtin":"0012345678905", "sku":" SKU 001 ",
            "external_identifiers":{"shopify_product_gid":"gid://shopify/Product/001", "shopify_product_variation_gid":"002", "shopify_handle":" Ž \\\" "}
        })).unwrap();
        let entity = RetailerOfferIdentifiers {
            identifiers: initial.clone(),
            id: id.clone(),
            retailer_code: bodul_shared::retailer::RetailerCode::AnkerAu,
            url: "https://example.com/Exact?Variant=001".into(),
        };
        repository.insert(&entity).unwrap();
        assert_eq!(repository.entries.lock().unwrap()[&uuid], entity);
        let mut invalid = initial;
        invalid.sku.as_mut().unwrap().value.clear();
        assert!(repository.update_identifiers(&id, &invalid).is_err());
        assert_eq!(repository.entries.lock().unwrap()[&uuid], entity);
        let replacement: ProductIdentifiers = serde_json::from_value(json!({"sku":"new"})).unwrap();
        for _ in 0..2 {
            repository.update_identifiers(&id, &replacement).unwrap();
        }
        let stored = repository.entries.lock().unwrap()[&uuid].clone();
        assert_eq!(stored.identifiers, replacement);
        assert_eq!(stored.url, entity.url);
        assert_eq!(stored.retailer_code, entity.retailer_code);
        repository
            .update_identifiers(&id, &ProductIdentifiers::default())
            .unwrap();
        assert_eq!(
            repository.entries.lock().unwrap()[&uuid].identifiers,
            ProductIdentifiers::default()
        );
        assert!(
            repository
                .update_identifiers(
                    &RetailerOfferId::new(Uuid::new_v4().to_string()),
                    &ProductIdentifiers::default()
                )
                .is_err()
        );
        assert!(
            repository
                .update_identifiers(
                    &RetailerOfferId::new("invalid"),
                    &ProductIdentifiers::default()
                )
                .is_err()
        );
    }
}
