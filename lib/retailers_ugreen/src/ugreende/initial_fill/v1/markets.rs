//! Country-specific Shopify market discovery. Languages belong to each country.
use retailer_sauce::shopify::initial_fill::GraphqlRequest;
use serde::Deserialize;
use serde_json::{Value, json};
use std::collections::BTreeMap;

use retailer_sauce::shopify::markets::MarketContext;

pub fn request() -> GraphqlRequest {
    GraphqlRequest {
        operation_name: "StorefrontMarkets".into(),
        query: "query StorefrontMarkets { localization { availableCountries { isoCode currency { isoCode } availableLanguages { isoCode } } } }".into(),
        variables: json!({}),
    }
}

#[derive(Deserialize)]
struct Envelope {
    data: Data,
    #[serde(default)]
    errors: Vec<Value>,
}
#[derive(Deserialize)]
struct Data {
    localization: Localization,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Localization {
    available_countries: Vec<Country>,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Country {
    iso_code: String,
    currency: Code,
    available_languages: Vec<Code>,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Code {
    iso_code: String,
}

pub fn contexts(response: &Value) -> Result<Vec<MarketContext>, String> {
    let response: Envelope = serde_json::from_value(response.clone())
        .map_err(|e| format!("invalid market discovery response: {e}"))?;
    if !response.errors.is_empty() {
        return Err("market discovery returned GraphQL errors".into());
    }
    let mut contexts = BTreeMap::new();
    for country in response.data.localization.available_countries {
        if country.available_languages.is_empty() {
            return Err("advertised country has no languages".into());
        }
        for language in country.available_languages {
            for code in [
                &country.iso_code,
                &country.currency.iso_code,
                &language.iso_code,
            ] {
                if code.is_empty() || !code.bytes().all(|c| c.is_ascii_uppercase() || c == b'_') {
                    return Err("invalid market context code".into());
                }
            }
            let key = (country.iso_code.clone(), language.iso_code.clone());
            let context = MarketContext {
                country: country.iso_code.clone(),
                language: language.iso_code,
                currency: country.currency.iso_code.clone(),
            };
            if let Some(previous) = contexts.insert(key, context.clone())
                && previous != context
            {
                return Err("conflicting currencies for advertised context".into());
            }
        }
    }
    if contexts.is_empty() {
        return Err("storefront advertised no market contexts".into());
    }
    Ok(contexts.into_values().collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn country(code: &str, currency: &str, languages: &[&str]) -> Value {
        json!({"isoCode":code,"currency":{"isoCode":currency},"availableLanguages":languages.iter().map(|l|json!({"isoCode":l})).collect::<Vec<_>>()})
    }
    fn response(countries: Vec<Value>) -> Value {
        json!({"data":{"localization":{"availableCountries":countries}}})
    }
    #[test]
    fn languages_are_country_specific_sorted_and_deduplicated() {
        let raw = response(vec![
            country("US", "USD", &["EN"]),
            country("CA", "CAD", &["FR", "EN", "FR"]),
        ]);
        let found = contexts(&raw).unwrap();
        assert_eq!(
            found
                .iter()
                .map(|c| (c.country.as_str(), c.language.as_str()))
                .collect::<Vec<_>>(),
            [("CA", "EN"), ("CA", "FR"), ("US", "EN")]
        );
        assert_eq!(found[0].currency, "CAD");
    }
    #[test]
    fn rejects_empty_invalid_and_conflicting_contexts() {
        for countries in [
            vec![],
            vec![country("CA", "CAD", &[])],
            vec![country("../CA", "CAD", &["EN"])],
            vec![country("CA", "CAD", &["EN"]), country("CA", "USD", &["EN"])],
        ] {
            assert!(contexts(&response(countries)).is_err());
        }
    }
    #[test]
    fn rejects_partial_graphql_errors() {
        let mut raw = response(vec![country("US", "USD", &["EN"])]);
        raw["errors"] = json!([{"message":"partial failure"}]);
        assert!(contexts(&raw).is_err());
    }
}
