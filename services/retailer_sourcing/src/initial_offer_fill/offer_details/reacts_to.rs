use super::{SourceType, command::FetchOfferDetails};
use crate::initial_offer_fill::product_catalog_ids::prelude::ProductCatalogIdsRetrieved;
use crate::platform::bus::KeyedCommand;
use crate::platform::bus::stable_command_id;

pub fn on(event: &ProductCatalogIdsRetrieved) -> Vec<KeyedCommand<FetchOfferDetails>> {
    event
        .product_ids
        .iter()
        .map(|id| {
            // Preserve the original identity bytes when replaying queued events;
            // payload serialization now uses lowercase retailer slugs (ADR 49).
            let key = serde_json::to_vec(&(
                "FetchOfferDetails",
                format!("{:?}", event.retailer_code),
                id,
            ))
            .expect("IDs serialize");
            let key = if let Some(context) = &event.context {
                serde_json::to_vec(&(
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
