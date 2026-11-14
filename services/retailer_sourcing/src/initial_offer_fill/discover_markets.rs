//! Snapshot all advertised contexts and fan out replay-safe catalog commands.
use super::{product_catalog_ids::command::FetchProductCatalogIdentifiers, transport::Http};
use crate::platform::{
    Platform,
    bus::{Bus, Command, Event, Handle, KeyedCommand, stable_command_id},
};
use retailer_sauce::shopify::markets::MarketContext;
use serde::{Deserialize, Serialize};
use shared::retailer::RetailerCode;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoverStorefrontMarkets {
    pub run_id: Uuid,
    #[serde(default)]
    pub storefront_token: Option<retailer_sauce::shopify::initial_fill::StorefrontToken>,
    pub retailer_code: RetailerCode,
}
impl Command for DiscoverStorefrontMarkets {
    const NAME: &'static str = "DiscoverStorefrontMarkets";
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorefrontMarketsDiscovered {
    pub run_id: Uuid,
    #[serde(default)]
    pub storefront_token: Option<retailer_sauce::shopify::initial_fill::StorefrontToken>,
    pub retailer_code: RetailerCode,
    pub contexts: Vec<MarketContext>,
}
impl Event for StorefrontMarketsDiscovered {
    const NAME: &'static str = "StorefrontMarketsDiscovered";
}
struct Handler<F>(F);
impl<F: Http + Send + Sync + 'static> Handle for Handler<F> {
    type Command = DiscoverStorefrontMarkets;
    type Event = StorefrontMarketsDiscovered;
    type Error = String;
    fn handle(&self, command: Self::Command) -> Result<Vec<Self::Event>, String> {
        let integration =
            retailer_data_extractor::initial_fill_integration_for(command.retailer_code)
                .ok_or("initial fill is not configured")?;
        let contexts = integration.discover_markets(&self.0, command.storefront_token.as_ref())?;
        Ok(vec![StorefrontMarketsDiscovered {
            storefront_token: command.storefront_token,
            run_id: command.run_id,
            retailer_code: command.retailer_code,
            contexts,
        }])
    }
}
fn on(event: &StorefrontMarketsDiscovered) -> Vec<KeyedCommand<FetchProductCatalogIdentifiers>> {
    event
        .contexts
        .iter()
        .map(|context| {
            let key = serde_json::to_vec(&(
                "FetchProductCatalogIdentifiers",
                &context.country,
                &context.language,
                Option::<String>::None,
            ))
            .expect("context serializes");
            KeyedCommand {
                id: stable_command_id(&event.run_id, &key),
                run_id: event.run_id,
                command: FetchProductCatalogIdentifiers {
                    run_id: event.run_id,
                    retailer_code: event.retailer_code,
                    storefront_token: event.storefront_token.clone(),
                    after: None,
                    context: Some(context.clone()),
                },
            }
        })
        .collect()
}
pub fn register(bus: &mut Bus, platform: &Platform) {
    bus.handle(Handler(platform.offer_http.clone()));
    bus.on_many_keyed(on);
    bus.on_many_keyed(acquired);
}

fn acquired(
    event: &super::acquire_storefront_token::prelude::StorefrontTokenAcquired,
) -> Vec<KeyedCommand<DiscoverStorefrontMarkets>> {
    if !retailer_data_extractor::all_market_contexts(event.retailer_code) {
        return Vec::new();
    }
    vec![KeyedCommand {
        id: stable_command_id(&event.run_id, b"DiscoverStorefrontMarkets"),
        run_id: event.run_id,
        command: DiscoverStorefrontMarkets {
            run_id: event.run_id,
            retailer_code: event.retailer_code,
            storefront_token: event.storefront_token.clone(),
        },
    }]
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn contexts_have_distinct_replay_safe_catalog_and_detail_commands() {
        let event = StorefrontMarketsDiscovered {
            storefront_token: None,
            run_id: Uuid::now_v7(),
            retailer_code: RetailerCode::UgreenCa,
            contexts: vec![
                MarketContext {
                    country: "CA".into(),
                    language: "EN".into(),
                    currency: "CAD".into(),
                },
                MarketContext {
                    country: "CA".into(),
                    language: "FR".into(),
                    currency: "CAD".into(),
                },
            ],
        };
        let commands = on(&event);
        assert_eq!(commands[0].id, on(&event)[0].id);
        assert_ne!(commands[0].id, commands[1].id);
        let mut ids = Vec::new();
        for command in commands {
            let event = super::super::product_catalog_ids::event::ProductCatalogIdsRetrieved {
                context: command.command.context,
                run_id: event.run_id,
                retailer_code: event.retailer_code,
                storefront_token: event.storefront_token.clone(),
                after: None,
                product_ids: vec!["gid://shopify/Product/1".into()],
                next_cursor: Some("next".into()),
                retrieved_at: chrono::Utc::now(),
            };
            let next = super::super::product_catalog_ids::reacts_to::next_batch(&event);
            assert_eq!(next[0].command.context, event.context);
            let details = super::super::offer_details::reacts_to::on(&event);
            assert_eq!(details[0].command.context, event.context);
            ids.push(details[0].id);
        }
        assert_ne!(ids[0], ids[1]);
    }
}

#[cfg(test)]
mod token_tests {
    use super::super::tests::{FakeHttp, token_value};
    use super::*;
    #[test]
    fn acquired_token_survives_discovery_and_catalog_fanout() {
        let event = super::super::acquire_storefront_token::prelude::StorefrontTokenAcquired {
            run_id: Uuid::now_v7(),
            retailer_code: RetailerCode::UgreenEu,
            storefront_token: Some(token_value()),
            url: "https://eu.ugreen.com".into(),
            acquired_at: chrono::Utc::now(),
        };
        let command = acquired(&event).remove(0).command;
        assert_eq!(command.storefront_token, event.storefront_token);
        let handler = Handler(FakeHttp::from(vec![
            serde_json::json!({"data":{"localization":{"availableCountries":[
                {"isoCode":"DE","currency":{"isoCode":"EUR"},"availableLanguages":[{"isoCode":"DE"},{"isoCode":"EN"}]}
            ]}}}),
        ]));
        let result = handler.handle(command).unwrap().remove(0);
        for command in on(&result) {
            assert_eq!(command.command.storefront_token, event.storefront_token);
        }
    }
}
