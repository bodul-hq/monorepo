//! Independent GraphQL integration for ugreennaseu.
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
const CODE: &str = "ugreennaseu";
const ALL_MARKETS: bool = true;
pub const CONFIG: StorefrontConfig = StorefrontConfig {
    acquire_token: true,
    token_required: false,
    storefront_url: "https://ai-eu.ugreen.com",
    graphql_url: "https://ugreen-nas-eu.myshopify.com/api/2026-10/graphql.json",
    token_market: None,
    token_environment: None,
    country: "AD",
    language: "EN",
    currency: "EUR",
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
mod integration_tests;
