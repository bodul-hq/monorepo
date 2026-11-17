//! Sourcing's policy over types received from the retailer libraries.
//!
//! Which locale contexts an initial fill crawls is decided by sourcing, not by
//! `retailer_sauce`; this moved here from `retailer_sauce::initial_fill` in 0.3.0.
//! Every initial-fill storefront uses `retailer_sauce`'s shared Shopify engine,
//! built from the retailer's storefront configuration.

use retailer_data_extractor::get_storefront_config;
use retailer_sauce::initial_fill::RequestedContext;
use retailer_sauce::prelude::{Market, StorefrontConfig};
use retailer_sauce::shopify::graphql::Engine;
use shared::retailer::RetailerCode;

/// The Shopify engine for a retailer's storefront, or an error when the
/// retailer has no storefront configuration.
pub(crate) fn storefront_engine(retailer: RetailerCode) -> Result<Engine, String> {
    get_storefront_config(retailer)
        .map(Engine)
        .ok_or_else(|| "initial fill is not configured".into())
}

/// Initial-fill request contexts derived from a storefront's supported locales.
pub(crate) trait InitialFillContexts {
    /// Snapshot the main-table locales in declaration order, collapsing request aliases.
    fn initial_fill_contexts(&self) -> Result<Vec<RequestedContext>, String>;
}
impl InitialFillContexts for StorefrontConfig {
    fn initial_fill_contexts(&self) -> Result<Vec<RequestedContext>, String> {
        let mut contexts = Vec::new();
        for locale in self.supported_locales() {
            let country = match locale.country_region {
                Some(Market::Country(country)) => Some(country.as_str().to_owned()),
                Some(Market::Region(_)) => return Err("GraphQL country cannot be a region".into()),
                None => None,
            };
            let context = RequestedContext {
                country,
                language: locale.language.as_str().to_owned(),
                currency: None,
                observed_country: None,
            };
            if !contexts.contains(&context) {
                contexts.push(context);
            }
        }
        if contexts.is_empty() {
            return Err("retailer has no supported locales".into());
        }
        Ok(contexts)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use retailer_data_extractor::get_storefront_config;
    use retailer_sauce::prelude::{
        Language, Locale, Region, RetailerCode, RetailerProfile, StorefrontAccess, SupportedLocales,
    };

    #[test]
    fn inventory_contexts_exclude_advertised_entries_and_preserve_countryless_languages() {
        let contexts = |code| {
            get_storefront_config(code)
                .unwrap()
                .initial_fill_contexts()
                .unwrap()
        };
        let ca = contexts(RetailerCode::AnkerCa);
        assert_eq!(
            ca.iter()
                .map(|c| (c.country.as_deref(), c.language.as_str()))
                .collect::<Vec<_>>(),
            [(Some("CA"), "EN"), (Some("CA"), "FR")]
        );
        assert_eq!(
            contexts(RetailerCode::AnkerEu)[0].country.as_deref(),
            Some("AT")
        );
        assert_eq!(contexts(RetailerCode::AnkerDe).len(), 1);
        let eu = contexts(RetailerCode::MinisForumEu);
        assert_eq!(
            eu.iter()
                .map(|c| (c.country.as_deref(), c.language.as_str()))
                .collect::<Vec<_>>(),
            [(None, "EN")]
        );
        let main = contexts(RetailerCode::UgreenCom);
        assert_eq!(main.len(), 2);
        assert!(main.iter().all(|c| c.currency.is_none()));
    }

    #[test]
    fn empty_locales_and_regional_countries_are_rejected() {
        const REGION_LOCALE: [SupportedLocales; 1] = [SupportedLocales::new(
            Locale::en_EU,
            "https://example.com",
            Some(Market::Region(Region::EuropeanUnion)),
            Language::EN,
        )];
        let config = |supported_locales| {
            StorefrontConfig::new(
                RetailerProfile {
                    retailer_code: RetailerCode::AnkerAu,
                    storefront_website: "https://example.com",
                    target_markets: &[],
                    shipping_countries: &[],
                    supported_locales,
                },
                StorefrontAccess {
                    shopify_shop: None,
                    storefront_graphql_api: None,
                    graphql_api_status: None,
                    acquire_token: false,
                    token_required: false,
                    storefront_url: "https://example.com",
                    graphql_url: "https://example.com/graphql",
                    token_market: None,
                    token_environment: None,
                    default_language: "EN",
                },
            )
        };
        assert!(config(&[]).initial_fill_contexts().is_err());
        assert!(config(&REGION_LOCALE).initial_fill_contexts().is_err());
    }
}
