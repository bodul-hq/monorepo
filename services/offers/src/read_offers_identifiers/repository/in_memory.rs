use super::{
    RetailerOfferIdentifiers, RetailerOfferIdentifiersRepository,
    RetailerOfferIdentifiersRepositoryError, err_insert, validate,
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
}
