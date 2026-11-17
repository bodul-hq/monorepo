//! Collect product identifiers for each configured retailer locale.
//!
//! `initial` starts one catalog traversal per locale. The handler walks the
//! whole catalog for that locale, batch by batch, and emits a single
//! `ProductCatalogIdsRetrieved` with every product ID; a reaction uses it to
//! queue product details. A failed batch retries the whole traversal; detail
//! command IDs are stable per product, so the queue drops repeated work.

pub(crate) mod prelude;

use crate::platform::bus::{Command, Event, Handle, KeyedCommand, stable_command_id};
use crate::v2::incoming::InitialFillContexts;
use crate::v2::incoming::storefront_engine;
use crate::v2::prelude::StorefrontTokenAcquired;
use chrono::DateTime;
use chrono::Utc;
use retailer_data_extractor::get_storefront_config;
use retailer_sauce::initial_fill::LocaleInitialFillIntegration;
use retailer_sauce::initial_fill::RequestedContext;
use retailer_sauce::prelude::{Http, StorefrontToken};
use serde::{Deserialize, Serialize};
use serde_json::to_vec;
use shared::retailer::RetailerCode;
use std::collections::HashSet;
use tracing::error;
use tracing::info;
use uuid::Uuid;

/// Collect every catalog identifier of one locale within an initial-fill run.
/// Commands queued by the former per-batch continuation also carried `after`;
/// serde ignores it, so they walk the whole catalog from the start.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FetchProductCatalogIdentifiers {
    pub run_id: Uuid,
    /// Locale and consistency checks carried through the traversal. Older queued
    /// commands may omit this field, so deserialization still accepts `None`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub context: Option<RequestedContext>,
    pub retailer_code: RetailerCode,
    pub storefront_token: Option<StorefrontToken>,
}
impl Command for FetchProductCatalogIdentifiers {
    const NAME: &'static str = "FetchProductCatalogIdentifiers";
}

pub struct Handler<F>(pub F);
impl<F: Http + Send + Sync + 'static> Handle for Handler<F> {
    type Command = FetchProductCatalogIdentifiers;
    type Event = ProductCatalogIdsRetrieved;
    type Error = String;
    fn handle(&self, command: Self::Command) -> Result<Vec<Self::Event>, String> {
        // The storefront engine owns API requests, pagination within a batch and
        // response validation. Propagate failures so the queue can retry them.
        let engine = storefront_engine(command.retailer_code)?;
        let mut context = command.context;
        let mut after = None;
        let mut seen_cursors = HashSet::new();
        let mut seen_ids = HashSet::new();
        let mut product_ids = Vec::new();
        let mut loop_count = 0;
        loop {
            loop_count += 1;
            let batch = engine.catalog_for_locale(
                &self.0,
                command.storefront_token.as_ref(),
                context.as_ref(),
                after.take(),
            )?;
            // Carry the integration's resolved context forward, including any
            // observed defaults needed to check later batches and details.
            context = batch.context;
            let count = batch.batch.product_ids.len();
            // Integrations deduplicate within a batch; keep first-seen order
            // across batches too.
            for id in batch.batch.product_ids {
                if seen_ids.insert(id.clone()) {
                    product_ids.push(id);
                }
            }
            info!(run_id = %command.run_id, context = ?context, loop_count, count, total = product_ids.len(), "catalog batch retrieved");
            match batch.batch.next_cursor {
                None => break,
                // Integrations only detect stalled cursors within a batch; a
                // cycle across batches would otherwise loop forever.
                Some(cursor) if !seen_cursors.insert(cursor.clone()) => {
                    return Err(format!("catalog cursor {cursor} repeated"));
                }
                Some(cursor) => after = Some(cursor),
            }
        }
        let retrieved = ProductCatalogIdsRetrieved {
            context,
            run_id: command.run_id,
            retailer_code: command.retailer_code,
            storefront_token: command.storefront_token,
            product_count: product_ids.len(),
            loop_count,
            product_ids,
            retrieved_at: Utc::now(),
        };

        Ok(vec![retrieved])
    }
}

/// Every product ID of one locale's catalog, consumed by the detail-fetch reaction.
/// Historical per-batch events also carried `after` and `next_cursor`; serde
/// ignores those fields when reading them.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProductCatalogIdsRetrieved {
    pub run_id: Uuid,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub context: Option<RequestedContext>,
    pub retailer_code: RetailerCode,
    pub storefront_token: Option<StorefrontToken>,
    /// Number of unique product IDs found. Historical events read as 0.
    #[serde(default)]
    pub product_count: usize,
    /// Number of catalog batches fetched to walk the whole catalog. Historical
    /// events read as 0.
    #[serde(default)]
    pub loop_count: usize,
    pub product_ids: Vec<String>,
    pub retrieved_at: DateTime<Utc>,
}
impl Event for ProductCatalogIdsRetrieved {
    const NAME: &'static str = "ProductCatalogIdsRetrieved";
}

/// Replaying an event must produce the same command ID within its run, so queue
/// deduplication can suppress duplicate work. A new run gets new IDs. The key
/// holds the requested country/language, not observed facts that may change
/// after a fetch. The trailing `null` is the start cursor of the former
/// per-batch commands, kept so command IDs stay unchanged (ADR 74).
fn keyed(
    run_id: Uuid,
    context: &RequestedContext,
    command: FetchProductCatalogIdentifiers,
) -> KeyedCommand<FetchProductCatalogIdentifiers> {
    let key = to_vec(&(
        "FetchProductCatalogIdentifiers",
        &context.country,
        &context.language,
        None::<String>,
    ))
    .expect("context serializes");
    KeyedCommand {
        id: stable_command_id(&run_id, &key),
        run_id,
        command,
    }
}
/// Start one catalog traversal per supported locale, in inventory order.
/// Used both by the start API and after token acquisition, so they share the
/// same locale validation and command identities.
pub fn initial(
    run_id: Uuid,
    retailer_code: RetailerCode,
    storefront_token: Option<StorefrontToken>,
) -> Result<Vec<KeyedCommand<FetchProductCatalogIdentifiers>>, String> {
    let retailer = get_storefront_config(retailer_code).ok_or("retailer inventory is missing")?;
    // Inventory decides which locales to crawl. Invalid inventory is an error;
    // silently choosing a default could collect the wrong market or language.
    Ok(retailer
        .initial_fill_contexts()?
        .into_iter()
        .map(|context| {
            let command = FetchProductCatalogIdentifiers {
                context: Some(context.clone()),
                run_id,
                retailer_code,
                storefront_token: storefront_token.clone(),
            };

            keyed(run_id, &context, command)
        })
        .collect())
}
/// Continue a token-acquisition event with the first catalog commands.
pub fn start_catalog_for_locales(
    event: &StorefrontTokenAcquired,
) -> Vec<KeyedCommand<FetchProductCatalogIdentifiers>> {
    // The API validated inventory before queueing the run, but this reaction
    // reads it again: configuration may have changed while the event waited.
    initial(
        event.run_id,
        event.retailer_code,
        event.storefront_token.clone(),
    )
    .unwrap_or_else(|error| {
        // This reaction returns commands, not a Result. Log invalid inventory
        // and emit no commands; this branch does not request a queue retry.
        error!(run_id=%event.run_id, retailer=%event.retailer_code.as_str(), %error, "initial fill has no catalog contexts");
        Vec::new()
    })
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::v2::application::steps::offer_details::fetch_details_for_catalog_products;
    fn token_value() -> StorefrontToken {
        StorefrontToken::new("synthetic-au-token".into()).unwrap()
    }

    fn token_event(retailer_code: RetailerCode) -> StorefrontTokenAcquired {
        StorefrontTokenAcquired {
            run_id: Uuid::now_v7(),
            retailer_code,
            storefront_token: Some(token_value()),
            url: "https://example.test".into(),
            acquired_at: Utc::now(),
        }
    }

    fn requested(
        commands: &[KeyedCommand<FetchProductCatalogIdentifiers>],
    ) -> Vec<(String, String)> {
        commands
            .iter()
            .map(|c| {
                let context = c.command.context.as_ref().unwrap();
                (
                    context.country.clone().unwrap_or_default(),
                    context.language.clone(),
                )
            })
            .collect()
    }

    #[test]
    fn token_event_fans_out_one_catalog_command_per_supported_locale() {
        let mut count = 0;
        for code in RetailerCode::ALL {
            let Some(config) = get_storefront_config(code) else {
                continue;
            };
            let event = token_event(code);
            let commands = start_catalog_for_locales(&event);
            let expected = config.initial_fill_contexts().unwrap();
            assert_eq!(
                commands
                    .iter()
                    .map(|c| c.command.context.clone().unwrap())
                    .collect::<Vec<_>>(),
                expected
            );
            assert_eq!(
                commands.iter().map(|c| c.id).collect::<Vec<_>>(),
                start_catalog_for_locales(&event)
                    .iter()
                    .map(|c| c.id)
                    .collect::<Vec<_>>()
            );
            for command in &commands {
                assert_eq!(command.command.retailer_code, code);
                assert_eq!(command.command.storefront_token, event.storefront_token);
            }
            count += 1;
        }
        assert_eq!(count, 48);
    }

    #[test]
    fn ankerca_crawls_both_locales_and_ankerau_one() {
        let pairs = |code| requested(&start_catalog_for_locales(&token_event(code)));
        assert_eq!(
            pairs(RetailerCode::AnkerCa),
            [("CA".into(), "EN".into()), ("CA".into(), "FR".into())]
        );
        assert_eq!(pairs(RetailerCode::AnkerAu), [("AU".into(), "EN".into())]);
    }

    #[test]
    fn contexts_keep_replay_safe_catalog_and_detail_identities() {
        let event = token_event(RetailerCode::AnkerCa);
        let commands = start_catalog_for_locales(&event);
        assert_eq!(
            commands[0].id,
            stable_command_id(
                &event.run_id,
                br#"["FetchProductCatalogIdentifiers","CA","EN",null]"#
            )
        );
        assert_ne!(commands[0].id, commands[1].id);
        let mut ids = Vec::new();
        for command in commands {
            let event = ProductCatalogIdsRetrieved {
                context: command.command.context,
                run_id: event.run_id,
                retailer_code: event.retailer_code,
                storefront_token: event.storefront_token.clone(),
                product_count: 1,
                loop_count: 1,
                product_ids: vec!["gid://shopify/Product/1".into()],
                retrieved_at: Utc::now(),
            };
            let details = fetch_details_for_catalog_products(&event);
            assert_eq!(details[0].command.context, event.context);
            let language = &event.context.as_ref().unwrap().language;
            let historical_detail_key = to_vec(&(
                "FetchOfferDetails",
                "AnkerCa",
                "CA",
                language,
                "gid://shopify/Product/1",
            ))
            .unwrap();
            assert_eq!(
                details[0].id,
                stable_command_id(&event.run_id, &historical_detail_key)
            );
            ids.push(details[0].id);
        }
        assert_ne!(ids[0], ids[1]);
    }
}
