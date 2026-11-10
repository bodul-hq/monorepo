//! Neutral initial-fill boundary. Retailers own all source interpretation.
use crate::shopify::{
    initial_fill::{GraphqlRequest, StorefrontConfig, StorefrontToken},
    markets::MarketContext,
};
use serde_json::Value;

pub trait Http {
    fn get(&self, _url: &str) -> Result<String, String> {
        Err("GET is unavailable".into())
    }
    fn configured_token(&self, _name: &str) -> Result<Option<StorefrontToken>, String> {
        Ok(None)
    }
    fn post(
        &self,
        _url: &str,
        _request: &GraphqlRequest,
        _token: Option<&StorefrontToken>,
    ) -> Result<Value, String> {
        Err("POST is unavailable".into())
    }
}
pub struct CatalogBatch {
    pub product_ids: Vec<String>,
    pub next_cursor: Option<String>,
}
pub trait InitialFillIntegration: Sync {
    fn config(&self) -> StorefrontConfig;
    fn all_market_contexts(&self) -> bool;
    fn resolve_access(&self, http: &dyn Http, url: &str)
    -> Result<Option<StorefrontToken>, String>;
    fn discover_markets(
        &self,
        http: &dyn Http,
        token: Option<&StorefrontToken>,
    ) -> Result<Vec<MarketContext>, String>;
    fn catalog(
        &self,
        http: &dyn Http,
        token: Option<&StorefrontToken>,
        context: Option<&MarketContext>,
        after: Option<String>,
    ) -> Result<CatalogBatch, String>;
    fn details(
        &self,
        http: &dyn Http,
        token: Option<&StorefrontToken>,
        context: Option<&MarketContext>,
        id: &str,
    ) -> Result<String, String>;
}
