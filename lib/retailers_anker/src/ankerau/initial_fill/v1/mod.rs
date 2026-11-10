//! Independent GraphQL integration for ankerau.
mod artifact;
mod auth;
mod catalog;
mod fetch;
pub mod markets;
mod operations;
mod scripts;
mod token;
use retailer_sauce::initial_fill::{CatalogBatch, Http, InitialFillIntegration};
use retailer_sauce::shopify::{
    initial_fill::{GraphqlRequest, StorefrontConfig, StorefrontToken},
    markets::MarketContext,
};
use serde_json::Value;
const CODE: &str = "ankerau";
const ALL_MARKETS: bool = false;
pub const CONFIG: StorefrontConfig = StorefrontConfig {
    acquire_token: true,
    token_required: true,
    storefront_url: "https://www.anker.com/au",
    graphql_url: "https://oneanker-au.myshopify.com/api/2026-10/graphql.json",
    token_market: Some("au"),
    token_environment: None,
    country: "AU",
    language: "EN",
    currency: "AUD",
};
/// Continuations identify the owning product/variant, never a translated label.
#[derive(Debug, Clone)]
pub enum Operation {
    Markets,
    InContext {
        context: MarketContext,
        operation: Box<Operation>,
    },
    Catalog {
        first: u16,
        after: Option<String>,
    },
    Details {
        id: String,
    },
    ProductConnection {
        id: String,
        field: String,
        after: Option<String>,
    },
    VariantConnection {
        id: String,
        field: String,
        after: Option<String>,
    },
    SellingPlans {
        id: String,
        group_after: Option<String>,
        group_cursor: String,
        after: Option<String>,
    },
}

pub struct Integration;
impl InitialFillIntegration for Integration {
    fn config(&self) -> StorefrontConfig {
        CONFIG
    }
    fn all_market_contexts(&self) -> bool {
        ALL_MARKETS
    }
    fn resolve_access(
        &self,
        http: &dyn Http,
        url: &str,
    ) -> Result<Option<StorefrontToken>, String> {
        auth::resolve(http, url)
    }
    fn discover_markets(
        &self,
        http: &dyn Http,
        token: Option<&StorefrontToken>,
    ) -> Result<Vec<MarketContext>, String> {
        markets::contexts(&execute(http, token, &Operation::Markets)?)
    }
    fn catalog(
        &self,
        http: &dyn Http,
        token: Option<&StorefrontToken>,
        context: Option<&MarketContext>,
        after: Option<String>,
    ) -> Result<CatalogBatch, String> {
        catalog::fetch(http, token, context, after)
    }
    fn details(
        &self,
        http: &dyn Http,
        token: Option<&StorefrontToken>,
        context: Option<&MarketContext>,
        id: &str,
    ) -> Result<String, String> {
        serde_json::to_string(&fetch::fetch_in_context(http, token, id, context)?)
            .map_err(|e| e.to_string())
    }
}
pub fn request(operation: &Operation) -> Result<GraphqlRequest, String> {
    operations::request(operation, &CONFIG)
}
pub fn validate_response(operation: &Operation, response: &Value) -> Result<(), String> {
    operations::validate_response(operation, response, &CONFIG)
}
fn execute(
    http: &dyn Http,
    token: Option<&StorefrontToken>,
    operation: &Operation,
) -> Result<Value, String> {
    let config = CONFIG;
    if config.requires_token() && token.is_none() {
        return Err("Storefront authentication does not match retailer configuration".into());
    }
    let request = request(operation)?;
    let response = http.post(config.graphql_url, &request, token)?;
    validate_response(operation, &response)?;
    let expected = match operation {
        Operation::InContext { context, .. } => {
            Some((context.country.as_str(), context.language.as_str()))
        }
        Operation::Markets => None,
        _ => Some((config.country, config.language)),
    };
    if matches!(operation, Operation::InContext { .. })
        && response.pointer("/extensions/context").is_none()
    {
        return Err("Storefront omitted requested context evidence".into());
    }
    if let Operation::InContext { context, .. } = operation
        && (response
            .pointer("/extensions/context/country")
            .and_then(Value::as_str)
            != Some(context.country.as_str())
            || response
                .pointer("/extensions/context/language")
                .and_then(Value::as_str)
                != Some(context.language.as_str()))
    {
        return Err("Storefront response omitted or changed the requested context".into());
    }
    if let Some((country, language)) = expected
        && let Some(context) = response.pointer("/extensions/context")
        && (context.get("country").is_some_and(|c| c != country)
            || context.get("language").is_some_and(|l| l != language))
    {
        return Err("Storefront response context does not match retailer market".into());
    }
    Ok(response)
}
pub fn product_gid(id: &str) -> Result<(), String> {
    let digits = id
        .strip_prefix("gid://shopify/Product/")
        .ok_or("invalid Shopify product GID")?;
    if digits.is_empty() || !digits.bytes().all(|b| b.is_ascii_digit()) {
        return Err("invalid Shopify product GID".into());
    }
    Ok(())
}

pub fn next_cursor(connection: &Value) -> Result<Option<String>, String> {
    let more = connection
        .pointer("/pageInfo/hasNextPage")
        .and_then(Value::as_bool)
        .ok_or("missing connection pageInfo")?;
    if !more {
        return Ok(None);
    }
    connection
        .pointer("/pageInfo/endCursor")
        .and_then(Value::as_str)
        .filter(|s| !s.is_empty())
        .map(|s| Some(s.to_owned()))
        .ok_or_else(|| "missing continuation cursor".into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    #[test]
    fn au_configuration_and_full_wip_response_match_typed_operations() {
        assert_eq!(CONFIG.storefront_url, "https://www.anker.com/au");
        assert_eq!(
            CONFIG.graphql_url,
            "https://oneanker-au.myshopify.com/api/2026-10/graphql.json"
        );
        let mut body: Value =
            serde_json::from_str(include_str!("fixtures/offer_details.json")).unwrap();
        // The WIP captured only __typename for presentations. Select their ID
        // too: graphql_client generates an empty enum for typename-only objects.
        // Keep the recorded fixture intact and supply only this newly selected
        // field for the schema compatibility check.
        fn presentation_ids(value: &mut Value) {
            match value {
                Value::Object(fields) => {
                    if fields.get("__typename").and_then(Value::as_str) == Some("MediaPresentation")
                    {
                        fields.insert("id".into(), json!("gid://shopify/MediaPresentation/1"));
                    }
                    for child in fields.values_mut() {
                        presentation_ids(child);
                    }
                }
                Value::Array(items) => items.iter_mut().for_each(presentation_ids),
                _ => {}
            }
        }
        presentation_ids(&mut body);
        let op = Operation::Details {
            id: body["data"]["product"]["id"].as_str().unwrap().into(),
        };
        validate_response(&op, &body).unwrap();
        assert_eq!(
            body["data"]["product"]["collections"]["pageInfo"]["hasNextPage"],
            true
        );
        let request = request(&op).unwrap();
        assert!(
            request
                .query
                .contains("country: $country, language: $language")
        );
        assert!(request.query.contains("sellingPlanAllocations"));
        assert!(request.query.contains("storeAvailability"));
        assert_eq!(request.variables["id"], body["data"]["product"]["id"]);
    }
    #[test]
    fn catalog_selects_only_identifiers_and_preserves_cursor_variables() {
        let op = Operation::Catalog {
            first: 250,
            after: Some("cursor".into()),
        };
        let request = request(&op).unwrap();
        assert_eq!(
            request.variables,
            json!({"country":"AU","language":"EN","first":250,"after":"cursor"})
        );
        assert!(!request.query.contains("variants"));
        let body = json!({"data":{"products":{"edges":[{"cursor":"c","node":{"id":"gid://shopify/Product/1"}}],"pageInfo":{"hasNextPage":false,"endCursor":"c"}}}});
        validate_response(&op, &body).unwrap();
        assert!(
            validate_response(
                &op,
                &json!({"data":body["data"],"errors":[{"message":"denied"}]})
            )
            .is_err()
        );
        assert!(validate_response(&op, &json!({"data":null})).is_err());
        assert!(validate_response(&op, &json!({"data":{"products":{}}})).is_err());
    }

    #[test]
    fn variant_continuations_decode_fractional_coordinates() {
        let operation = Operation::VariantConnection {
            id: "gid://shopify/ProductVariant/1".into(),
            field: "storeAvailability".into(),
            after: Some("previous".into()),
        };
        let body = json!({"data":{"node":{
            "__typename":"ProductVariant", "id":"gid://shopify/ProductVariant/1",
            "storeAvailability":{
                "pageInfo":{"hasNextPage":false,"endCursor":"last"},
                "edges":[{"cursor":"last","node":{
                    "available":true,"pickUpTime":"Tomorrow","quantityAvailable":3,
                    "location":{"id":"gid://shopify/Location/1","name":"Sydney",
                        "address":{"formatted":["Sydney"],"latitude":-33.8688,"longitude":151.2093}}
                }}]
            }
        }}});
        validate_response(&operation, &body).unwrap();
    }
}

#[cfg(test)]
mod integration_tests;
