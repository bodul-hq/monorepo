use super::{
    RetailerOffer, RetailerOfferRepository, RetailerOfferRepositoryError, err_insert, validate,
};
use std::{collections::HashMap, sync::Mutex};
use uuid::Uuid;

#[derive(Default)]
pub struct InMemoryRetailerOfferRepository {
    entries: Mutex<HashMap<Uuid, RetailerOffer>>,
}

impl InMemoryRetailerOfferRepository {
    pub fn new() -> Self {
        Self::default()
    }
}

impl RetailerOfferRepository for InMemoryRetailerOfferRepository {
    fn insert(&self, entity: &RetailerOffer) -> Result<(), RetailerOfferRepositoryError> {
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
    fn update_title(
        &self,
        id: &super::RetailerOfferId,
        title: &str,
    ) -> Result<(), RetailerOfferRepositoryError> {
        if title.contains('\0') {
            return Err(super::err_update("title must contain no NUL"));
        }
        let id = Uuid::parse_str(id.value()).map_err(super::err_update)?;
        let mut entries = self.entries.lock().map_err(super::err_update)?;
        let entity = entries.get_mut(&id).ok_or_else(super::err_offer_missing)?;
        entity.title = title.to_owned();
        Ok(())
    }

    fn update_price(
        &self,
        id: &super::RetailerOfferId,
        price: &money::Money,
    ) -> Result<(), RetailerOfferRepositoryError> {
        let id = Uuid::parse_str(id.value()).map_err(super::err_update)?;
        let mut entries = self.entries.lock().map_err(super::err_update)?;
        let entity = entries.get_mut(&id).ok_or_else(super::err_offer_missing)?;
        entity.price = price.clone();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::offers::retailer_offer_id::RetailerOfferId;
    use money::{Currency, Money};

    #[test]
    fn updates_preserve_other_fields_and_reject_missing_offers() {
        let repository = InMemoryRetailerOfferRepository::new();
        let mut expected = RetailerOffer {
            id: RetailerOfferId::new(Uuid::new_v4().to_string()),
            retailer_code: bodul_shared::retailer::RetailerCode::from_str("ankerau").unwrap(),
            title: "Charger".to_owned(),
            price: Money::new(1999, Currency::AUD),
            url: "https://example.com/charger".to_owned(),
            created_at: chrono::Utc::now(),
        };
        assert!(repository.update_title(&expected.id, "missing").is_err());
        assert!(
            repository
                .update_price(&expected.id, &expected.price)
                .is_err()
        );
        repository.insert(&expected).unwrap();
        repository.update_title(&expected.id, "New title").unwrap();
        expected.title = "New title".to_owned();
        let id = Uuid::parse_str(expected.id.value()).unwrap();
        assert_eq!(repository.entries.lock().unwrap().get(&id), Some(&expected));
        expected.price = Money::new(1200, Currency::EUR);
        repository
            .update_price(&expected.id, &expected.price)
            .unwrap();
        assert!(repository.update_title(&expected.id, "bad\0title").is_err());
        assert_eq!(repository.entries.lock().unwrap().get(&id), Some(&expected));
    }
}
