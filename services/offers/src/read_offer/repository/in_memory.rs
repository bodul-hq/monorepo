use super::{
    RetailerOffer, RetailerOfferRepository, RetailerOfferRepositoryError, err_insert, validate,
};
use std::{collections::HashMap, sync::Mutex};
use uuid::Uuid;

#[derive(Default)]
pub struct InMemoryRetailerOfferRepository {
    entries: Mutex<HashMap<Uuid, RetailerOffer>>,
    images: Mutex<HashMap<Uuid, std::collections::BTreeMap<u32, String>>>,
}

impl InMemoryRetailerOfferRepository {
    pub fn new() -> Self {
        Self::default()
    }
}

impl RetailerOfferRepository for InMemoryRetailerOfferRepository {
    fn update_identifiers(
        &self,
        id: &crate::offers::retailer_offer_id::RetailerOfferId,
        identifiers: &retailer_sauce::identifier::ProductIdentifiers,
    ) -> Result<(), RetailerOfferRepositoryError> {
        identifiers.guard().map_err(super::err_update)?;
        let id = Uuid::parse_str(id.value()).map_err(super::err_update)?;
        let mut entries = self.entries.lock().map_err(super::err_update)?;
        let entity = entries.get_mut(&id).ok_or_else(super::err_offer_missing)?;
        entity.identifiers = identifiers.clone();
        Ok(())
    }
    fn update_localization(
        &self,
        id: &crate::offers::retailer_offer_id::RetailerOfferId,
        localization: &retailer_sauce::localization::Localization,
    ) -> Result<(), RetailerOfferRepositoryError> {
        localization.guard().map_err(super::err_update)?;
        let id = Uuid::parse_str(id.value()).map_err(super::err_update)?;
        let mut entries = self.entries.lock().map_err(super::err_update)?;
        let entity = entries.get_mut(&id).ok_or_else(super::err_offer_missing)?;
        entity.localization = localization.clone();
        Ok(())
    }
    fn update_scraped_at(
        &self,
        id: &super::RetailerOfferId,
        scraped_at: chrono::DateTime<chrono::Utc>,
    ) -> Result<(), RetailerOfferRepositoryError> {
        let id = Uuid::parse_str(id.value()).map_err(super::err_update)?;
        let mut entries = self.entries.lock().map_err(super::err_update)?;
        let entity = entries.get_mut(&id).ok_or_else(super::err_offer_missing)?;
        entity.scraped_at = entity.scraped_at.max(Some(scraped_at));
        Ok(())
    }

    fn images(
        &self,
        id: &super::RetailerOfferId,
    ) -> Result<Vec<String>, RetailerOfferRepositoryError> {
        let id = Uuid::parse_str(id.value()).map_err(super::err_update)?;
        Ok(self
            .images
            .lock()
            .map_err(super::err_update)?
            .get(&id)
            .map(|images| images.values().cloned().collect())
            .unwrap_or_default())
    }
    fn add_image(
        &self,
        event: &crate::offers::event::OfferImageAddedEvent,
    ) -> Result<(), RetailerOfferRepositoryError> {
        if !shared::images::is_valid(&event.image_url) {
            return Err(super::err_update("invalid image URL"));
        }
        let id = Uuid::parse_str(event.id.value()).map_err(super::err_update)?;
        let mut entries = self.entries.lock().map_err(super::err_update)?;
        let entity = entries.get_mut(&id).ok_or_else(super::err_offer_missing)?;
        let mut all = self.images.lock().map_err(super::err_update)?;
        let images = all.entry(id).or_default();
        if images.get(&event.position) == Some(&event.image_url) {
            return Ok(());
        }
        if images.contains_key(&event.position)
            || images.values().any(|url| url == &event.image_url)
        {
            return Err(super::err_update("conflicting image addition"));
        }
        images.insert(event.position, event.image_url.clone());
        entity.images = images.values().cloned().collect();
        Ok(())
    }

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
        let mut all_images = self.images.lock().map_err(err_insert)?;
        let images = entity
            .images
            .iter()
            .cloned()
            .enumerate()
            .map(|(position, image)| (position as u32, image))
            .collect();
        entries.insert(id, entity.clone());
        all_images.insert(id, images);
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
            identifiers: Default::default(),
            localization: Default::default(),
            scraped_at: None,
            images: Vec::new(),
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
