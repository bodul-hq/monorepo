use super::{command::FetchProductCatalogIdentifiers, event::ProductCatalogIdsRetrieved};
use crate::initial_offer_fill::acquire_storefront_token::prelude::StorefrontTokenAcquired;
use crate::platform::bus::KeyedCommand;
use crate::platform::bus::stable_command_id;

fn keyed(command: FetchProductCatalogIdentifiers) -> KeyedCommand<FetchProductCatalogIdentifiers> {
    // JSON distinguishes the initial null cursor from any literal cursor string.
    let key = serde_json::to_vec(&("FetchProductCatalogIdentifiers", &command.after))
        .expect("strings serialize");
    let key = if let Some(context) = &command.context {
        serde_json::to_vec(&(
            "FetchProductCatalogIdentifiers",
            &context.country,
            &context.language,
            &command.after,
        ))
        .expect("context serializes")
    } else {
        key
    };
    KeyedCommand {
        id: stable_command_id(&command.run_id, &key),
        run_id: command.run_id,
        command,
    }
}
pub fn acquired(
    event: &StorefrontTokenAcquired,
) -> Vec<KeyedCommand<FetchProductCatalogIdentifiers>> {
    if retailer_data_extractor::all_market_contexts(event.retailer_code) {
        return Vec::new();
    }
    vec![keyed(FetchProductCatalogIdentifiers {
        context: None,
        run_id: event.run_id,
        retailer_code: event.retailer_code,
        storefront_token: event.storefront_token.clone(),
        after: None,
    })]
}
pub fn next_batch(
    event: &ProductCatalogIdsRetrieved,
) -> Vec<KeyedCommand<FetchProductCatalogIdentifiers>> {
    event
        .next_cursor
        .as_ref()
        .map(|cursor| {
            keyed(FetchProductCatalogIdentifiers {
                context: event.context.clone(),
                run_id: event.run_id,
                retailer_code: event.retailer_code,
                storefront_token: event.storefront_token.clone(),
                after: Some(cursor.clone()),
            })
        })
        .into_iter()
        .collect()
}
