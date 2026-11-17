//! Step that fetches offer details.

pub(crate) mod prelude;

use crate::platform::bus::{Command, Event, Handle, KeyedCommand, stable_command_id};
use crate::v2::incoming::storefront_engine;
use crate::v2::prelude::ProductCatalogIdsRetrieved;
use chrono::DateTime;
use chrono::Utc;
use retailer_sauce::initial_fill::FetchedDetails;
use retailer_sauce::initial_fill::LocaleInitialFillIntegration;
use retailer_sauce::initial_fill::RequestedContext;
use retailer_sauce::prelude::MarketContext;
use retailer_sauce::prelude::{Http, StorefrontToken};
use serde::{Deserialize, Serialize};
use serde_json::to_vec;
use shared::retailer::RetailerCode;
use tracing::info;
use uuid::Uuid;

use crate::platform::object_storage::{ObjectStorage, OfferStorageMetadata};

/// The source and input needed to fetch offer details.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SourceType {
    Html { url: String },
    GraphQl { product_id: String },
}

/// Fetches all selected offer details from the given source.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FetchOfferDetails {
    pub run_id: Uuid,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub context: Option<RequestedContext>,
    pub storefront_token: Option<StorefrontToken>,
    pub retailer_code: RetailerCode,
    pub source: SourceType,
}

impl Command for FetchOfferDetails {
    const NAME: &'static str = "FetchOfferDetails";
}

pub const BUCKET: &str = "offer_details_raw";
pub trait Store: Send + Sync + 'static {
    fn store(&self, body: &str, fetched_at: DateTime<Utc>) -> Result<OfferStorageMetadata, String>;
}
impl Store for ObjectStorage {
    fn store(&self, body: &str, fetched_at: DateTime<Utc>) -> Result<OfferStorageMetadata, String> {
        let mut metadata = self.create_offer(BUCKET, body).map_err(|e| e.to_string())?;
        metadata.scraped_at = Some(fetched_at);
        Ok(metadata)
    }
}
pub struct Handler<F, S> {
    pub fetcher: F,
    pub store: S,
}
impl<F: Http + Send + Sync + 'static, S: Store> Handle for Handler<F, S> {
    type Command = FetchOfferDetails;
    type Event = OfferDetailsFetched;
    type Error = String;
    fn handle(&self, command: Self::Command) -> Result<Vec<Self::Event>, String> {
        let SourceType::GraphQl { product_id } = &command.source else {
            return Err("HTML initial-fill details are not implemented".into());
        };
        let details = fetch_for_locale(
            &self.fetcher,
            command.retailer_code,
            command.storefront_token.as_ref(),
            product_id,
            command.context.as_ref(),
        )?;
        let fetched_at = Utc::now();
        let storage = self.store.store(&details.body, fetched_at)?;
        info!(run_id = %command.run_id, context = ?command.context, product_id, object_storage_id = %storage.object_storage_id, "complete offer details stored");
        let fetched = OfferDetailsFetched {
            context: details.context,
            run_id: command.run_id,
            retailer_code: command.retailer_code,
            source: command.source,
            fetched_at,
            storage,
        };

        Ok(vec![fetched])
    }
}

/// Records the retailer and source used to fetch offer details.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OfferDetailsFetched {
    pub run_id: Uuid,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub context: Option<MarketContext>,
    pub retailer_code: RetailerCode,
    pub source: SourceType,
    pub fetched_at: DateTime<Utc>,
    pub storage: OfferStorageMetadata,
}

impl Event for OfferDetailsFetched {
    const NAME: &'static str = "OfferDetailsFetched";
}

pub fn fetch_for_locale<F: Http>(
    fetcher: &F,
    retailer: RetailerCode,
    token: Option<&StorefrontToken>,
    id: &str,
    context: Option<&RequestedContext>,
) -> Result<FetchedDetails, String> {
    storefront_engine(retailer)?.details_for_locale(fetcher, token, context, id)
}

pub fn fetch_details_for_catalog_products(
    event: &ProductCatalogIdsRetrieved,
) -> Vec<KeyedCommand<FetchOfferDetails>> {
    event
        .product_ids
        .iter()
        .map(|id| {
            // Preserve the original identity bytes when replaying queued events;
            // payload serialization now uses lowercase retailer slugs (ADR 49).
            let key = to_vec(&(
                "FetchOfferDetails",
                format!("{:?}", event.retailer_code),
                id,
            ))
            .expect("IDs serialize");

            let key = if let Some(context) = &event.context {
                to_vec(&(
                    "FetchOfferDetails",
                    format!("{:?}", event.retailer_code),
                    &context.country,
                    &context.language,
                    id,
                ))
                .expect("context serializes")
            } else {
                key
            };

            KeyedCommand {
                id: stable_command_id(&event.run_id, &key),
                run_id: event.run_id,
                command: FetchOfferDetails {
                    context: event.context.clone(),
                    run_id: event.run_id,
                    retailer_code: event.retailer_code,
                    storefront_token: event.storefront_token.clone(),
                    source: SourceType::GraphQl {
                        product_id: id.clone(),
                    },
                },
            }
        })
        .collect()
}

#[cfg(test)]
pub fn fetch_in_context<F: Http>(
    fetcher: &F,
    retailer: RetailerCode,
    token: Option<&StorefrontToken>,
    id: &str,
    context: Option<&MarketContext>,
) -> Result<String, String> {
    use retailer_sauce::prelude::InitialFillIntegration;

    storefront_engine(retailer)?.details(fetcher, token, context, id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::v2::application::prelude::product_catalog_ids::prelude::initial;
    use std::collections::HashSet;

    #[test]
    fn catalog_products_keep_context_and_historical_command_ids() {
        let catalogs = initial(Uuid::nil(), RetailerCode::AnkerCa, None).unwrap();
        let mut ids = Vec::new();
        for catalog in catalogs {
            let event = ProductCatalogIdsRetrieved {
                run_id: catalog.run_id,
                context: catalog.command.context,
                retailer_code: RetailerCode::AnkerCa,
                storefront_token: None,
                product_count: 2,
                loop_count: 1,
                product_ids: vec![
                    "gid://shopify/Product/1".into(),
                    "gid://shopify/Product/2".into(),
                ],
                retrieved_at: Utc::now(),
            };
            let commands = fetch_details_for_catalog_products(&event);
            assert_eq!(commands.len(), 2);
            for (command, product_id) in commands.iter().zip(&event.product_ids) {
                let context = event.context.as_ref().unwrap();
                let key = to_vec(&(
                    "FetchOfferDetails",
                    "AnkerCa",
                    &context.country,
                    &context.language,
                    product_id,
                ))
                .unwrap();
                assert_eq!(command.id, stable_command_id(&event.run_id, &key));
                assert_eq!(command.command.context, event.context);
                assert_eq!(
                    command.command.source,
                    SourceType::GraphQl {
                        product_id: product_id.clone()
                    }
                );
                ids.push(command.id);
            }
            let empty = ProductCatalogIdsRetrieved {
                product_ids: Vec::new(),
                ..event
            };
            assert!(fetch_details_for_catalog_products(&empty).is_empty());
        }
        assert_eq!(ids.iter().collect::<HashSet<_>>().len(), 4);
    }
}
