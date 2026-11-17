use super::RetailerOfferId as ImportedRetailerOfferId;
use super::err_offer_missing;
use super::err_update;
use crate::offers::event::OfferImageAddedEvent;
use crate::offers::event::Pricing;
use crate::offers::retailer_offer_id::RetailerOfferId as OffersRetailerOfferIdRetailerOfferId;
use chrono::DateTime;
use chrono::Utc;
use money::Money as MoneyMoney;
use offer_content::guard as offer_content_guard;
use offer_content::selected;
use offer_content::sort;
use retailer_sauce::prelude::Availability;
use retailer_sauce::prelude::ItemCondition;
use retailer_sauce::prelude::Localization;
use retailer_sauce::prelude::OfferContent;
use retailer_sauce::prelude::ProductIdentifiers;
use retailer_sauce::prelude::StructuralizedContent;
use retailer_sauce::prelude::legacy_description;
use retailer_sauce::prelude::legacy_language;
use retailer_sauce::prelude::legacy_title;
use shared::images::is_valid;
use std::collections::BTreeMap;

use super::{
    RetailerOffer, RetailerOfferRepository, RetailerOfferRepositoryError, err_insert, validate,
};
use retailer_sauce::prelude::offer_content;
use std::{collections::HashMap, sync::Mutex};
use uuid::Uuid;

#[derive(Default)]
pub struct InMemoryRetailerOfferRepository {
    entries: Mutex<HashMap<Uuid, RetailerOffer>>,
    images: Mutex<HashMap<Uuid, BTreeMap<u32, String>>>,
}

impl InMemoryRetailerOfferRepository {
    pub fn new() -> Self {
        Self::default()
    }
}

impl RetailerOfferRepository for InMemoryRetailerOfferRepository {
    fn update_contents(
        &self,
        id: &ImportedRetailerOfferId,
        content: &[OfferContent],
        language: &Option<String>,
    ) -> Result<(), RetailerOfferRepositoryError> {
        offer_content_guard(content, language).map_err(err_update)?;
        let mut content = content.to_vec();
        sort(&mut content);
        let title = selected(&content, language)
            .map_err(err_update)?
            .title
            .clone();
        let id = Uuid::parse_str(id.value()).map_err(err_update)?;
        let mut entries = self.entries.lock().map_err(err_update)?;
        let entity = entries.get_mut(&id).ok_or_else(err_offer_missing)?;
        entity.facts.content = content;
        entity.localization.language = language.clone();
        entity.title = title;
        Ok(())
    }
    fn update_content(
        &self,
        id: &ImportedRetailerOfferId,
        content: &StructuralizedContent,
    ) -> Result<(), RetailerOfferRepositoryError> {
        content.guard().map_err(err_update)?;
        let id = Uuid::parse_str(id.value()).map_err(err_update)?;
        let mut entries = self.entries.lock().map_err(err_update)?;
        let entity = entries.get_mut(&id).ok_or_else(err_offer_missing)?;
        entity.facts.content = legacy_description(
            &entity.facts.content,
            &entity.localization.language,
            content.description.clone(),
        )
        .map_err(err_update)?;
        Ok(())
    }
    fn update_item_condition(
        &self,
        id: &ImportedRetailerOfferId,
        item_condition: &Option<ItemCondition>,
    ) -> Result<(), RetailerOfferRepositoryError> {
        if let Some(value) = item_condition {
            value.guard().map_err(err_update)?;
        }
        let id = Uuid::parse_str(id.value()).map_err(err_update)?;
        let mut entries = self.entries.lock().map_err(err_update)?;
        let entity = entries.get_mut(&id).ok_or_else(err_offer_missing)?;
        entity.facts.item_condition = item_condition.clone();
        Ok(())
    }
    fn update_availability(
        &self,
        id: &ImportedRetailerOfferId,
        availability: &Option<Availability>,
    ) -> Result<(), RetailerOfferRepositoryError> {
        if let Some(value) = availability {
            value.guard().map_err(err_update)?;
        }
        let id = Uuid::parse_str(id.value()).map_err(err_update)?;
        let mut entries = self.entries.lock().map_err(err_update)?;
        let entity = entries.get_mut(&id).ok_or_else(err_offer_missing)?;
        entity.facts.availability = availability.clone();
        Ok(())
    }
    fn update_pricing(
        &self,
        id: &ImportedRetailerOfferId,
        pricing: &Pricing,
    ) -> Result<(), RetailerOfferRepositoryError> {
        pricing.guard().map_err(err_update)?;
        let id = Uuid::parse_str(id.value()).map_err(err_update)?;
        let mut entries = self.entries.lock().map_err(err_update)?;
        let entity = entries.get_mut(&id).ok_or_else(err_offer_missing)?;
        entity.price = pricing.price.clone();
        entity.facts.was_price = pricing.was_price.clone();
        Ok(())
    }
    fn update_identifiers(
        &self,
        id: &OffersRetailerOfferIdRetailerOfferId,
        identifiers: &ProductIdentifiers,
    ) -> Result<(), RetailerOfferRepositoryError> {
        identifiers.guard().map_err(err_update)?;
        let id = Uuid::parse_str(id.value()).map_err(err_update)?;
        let mut entries = self.entries.lock().map_err(err_update)?;
        let entity = entries.get_mut(&id).ok_or_else(err_offer_missing)?;
        entity.identifiers = identifiers.clone();
        Ok(())
    }
    fn update_localization(
        &self,
        id: &OffersRetailerOfferIdRetailerOfferId,
        localization: &Localization,
    ) -> Result<(), RetailerOfferRepositoryError> {
        localization.guard().map_err(err_update)?;
        let id = Uuid::parse_str(id.value()).map_err(err_update)?;
        let mut entries = self.entries.lock().map_err(err_update)?;
        let entity = entries.get_mut(&id).ok_or_else(err_offer_missing)?;
        let content = legacy_language(
            &entity.facts.content,
            &entity.localization.language,
            localization.language.clone(),
        )
        .map_err(err_update)?;
        entity.facts.content = content;
        entity.localization = localization.clone();
        Ok(())
    }
    fn update_scraped_at(
        &self,
        id: &ImportedRetailerOfferId,
        scraped_at: DateTime<Utc>,
    ) -> Result<(), RetailerOfferRepositoryError> {
        let id = Uuid::parse_str(id.value()).map_err(err_update)?;
        let mut entries = self.entries.lock().map_err(err_update)?;
        let entity = entries.get_mut(&id).ok_or_else(err_offer_missing)?;
        entity.scraped_at = entity.scraped_at.max(Some(scraped_at));
        Ok(())
    }

    fn images(
        &self,
        id: &ImportedRetailerOfferId,
    ) -> Result<Vec<String>, RetailerOfferRepositoryError> {
        let id = Uuid::parse_str(id.value()).map_err(err_update)?;
        Ok(self
            .images
            .lock()
            .map_err(err_update)?
            .get(&id)
            .map(|images| images.values().cloned().collect())
            .unwrap_or_default())
    }
    fn add_image(&self, event: &OfferImageAddedEvent) -> Result<(), RetailerOfferRepositoryError> {
        if !is_valid(&event.image_url) {
            return Err(err_update("invalid image URL"));
        }
        let id = Uuid::parse_str(event.id.value()).map_err(err_update)?;
        let mut entries = self.entries.lock().map_err(err_update)?;
        let entity = entries.get_mut(&id).ok_or_else(err_offer_missing)?;
        let mut all = self.images.lock().map_err(err_update)?;
        let images = all.entry(id).or_default();
        if images.get(&event.position) == Some(&event.image_url) {
            return Ok(());
        }
        if images.contains_key(&event.position)
            || images.values().any(|url| url == &event.image_url)
        {
            return Err(err_update("conflicting image addition"));
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
        let mut entity = entity.clone();
        sort(&mut entity.facts.content);
        entries.insert(id, entity);
        all_images.insert(id, images);
        Ok(())
    }
    fn update_title(
        &self,
        id: &ImportedRetailerOfferId,
        title: &str,
    ) -> Result<(), RetailerOfferRepositoryError> {
        if title.contains('\0') {
            return Err(err_update("title must contain no NUL"));
        }
        let id = Uuid::parse_str(id.value()).map_err(err_update)?;
        let mut entries = self.entries.lock().map_err(err_update)?;
        let entity = entries.get_mut(&id).ok_or_else(err_offer_missing)?;
        let content = legacy_title(
            &entity.facts.content,
            &entity.localization.language,
            title.to_owned(),
        )
        .map_err(err_update)?;
        entity.facts.content = content;
        entity.title = title.to_owned();
        Ok(())
    }

    fn update_price(
        &self,
        id: &ImportedRetailerOfferId,
        price: &MoneyMoney,
    ) -> Result<(), RetailerOfferRepositoryError> {
        let id = Uuid::parse_str(id.value()).map_err(err_update)?;
        let mut entries = self.entries.lock().map_err(err_update)?;
        let entity = entries.get_mut(&id).ok_or_else(err_offer_missing)?;
        entity.price = price.clone();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::offers::event::OfferFacts;
    use chrono::Utc as ChronoUtc;
    use offer_content::singleton;
    use shared::retailer::RetailerCode;

    use super::*;
    use crate::offers::retailer_offer_id::RetailerOfferId;
    use money::{Currency, Money};

    #[test]
    fn updates_preserve_other_fields_and_reject_missing_offers() {
        let repository = InMemoryRetailerOfferRepository::new();
        let mut expected = RetailerOffer {
            facts: OfferFacts {
                content: singleton("Charger".into(), None, None),
                ..Default::default()
            },
            identifiers: Default::default(),
            localization: Default::default(),
            scraped_at: None,
            images: Vec::new(),
            id: RetailerOfferId::new(Uuid::new_v4().to_string()),
            retailer_code: RetailerCode::from_str("ankerau").unwrap(),
            title: "Charger".to_owned(),
            price: Money::new(1999, Currency::AUD),
            url: "https://example.com/charger".to_owned(),
            created_at: ChronoUtc::now(),
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
        expected.facts.content[0].title = expected.title.clone();
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

#[cfg(test)]
mod localized_content_tests {
    use offer_content::singleton as offer_content_singleton;
    use offer_content::upsert;
    use retailer_sauce::prelude::Localization as RetailerSauceLocalization;
    use retailer_sauce::prelude::StructuralizedContent as RetailerSauceStructuralizedContent;
    use serde_json::from_value;

    use super::*;
    use crate::offers::event::OfferCreatedEvent;
    use serde_json::json;

    #[test]
    fn full_snapshots_are_atomic_and_legacy_changes_reconcile_selected_bucket() {
        let repository = InMemoryRetailerOfferRepository::new();
        let id = Uuid::now_v7();
        let created: OfferCreatedEvent = from_value(json!({
            "id":id,"retailer_code":"AnkerCom","title":" English ","price":{"amount":1999,"currency":"USD"},"url":"https://example.com/localized","created_at":"2020-01-02T03:04:05Z",
            "localization":{"target_market":null,"language":"en-US"},
            "content":{"description":" Exact legacy "}
        })).unwrap();
        let entity = RetailerOffer {
            facts: created.facts,
            identifiers: created.identifiers,
            localization: created.localization,
            scraped_at: created.scraped_at,
            id: created.id,
            retailer_code: created.retailer_code,
            title: created.title,
            price: created.price,
            url: created.url,
            images: created.images,
            created_at: created.created_at,
        };
        repository.insert(&entity).unwrap();
        let content = upsert(
            &entity.facts.content,
            Some("fr".into()),
            "Français".into(),
            Some(Some("".into())),
        )
        .unwrap();
        repository
            .update_contents(&entity.id, &content, &Some("fr".into()))
            .unwrap();
        let snapshot = || repository.entries.lock().unwrap().get(&id).unwrap().clone();
        assert_eq!(snapshot().title, "Français");
        let before = snapshot();
        for (content, language) in [
            (vec![], None),
            (content.clone(), None),
            (
                vec![content[0].clone(), content[0].clone()],
                Some("en-US".into()),
            ),
        ] {
            assert!(
                repository
                    .update_contents(&entity.id, &content, &language)
                    .is_err()
            );
            assert_eq!(snapshot(), before);
        }
        repository
            .update_title(&entity.id, " Legacy title ")
            .unwrap();
        let description = RetailerSauceStructuralizedContent {
            description: Some(" Legacy description ".into()),
        };
        repository.update_content(&entity.id, &description).unwrap();
        let localization = RetailerSauceLocalization {
            target_market: None,
            language: Some("en-US".into()),
        };
        repository
            .update_localization(&entity.id, &localization)
            .unwrap();
        let after = snapshot();
        assert_eq!(after.title, " Legacy title ");
        assert_eq!(
            after.facts.content,
            offer_content_singleton(
                " Legacy title ".into(),
                Some("en-US".into()),
                Some(" Legacy description ".into())
            )
        );
        assert_eq!(after.price, entity.price);
        assert_eq!(after.url, entity.url);
    }
}
