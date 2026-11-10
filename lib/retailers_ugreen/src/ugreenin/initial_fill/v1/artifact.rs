//! Complete product snapshot with request/response evidence; authentication is excluded.
use retailer_sauce::shopify::initial_fill::GraphqlRequest;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Page {
    pub request: GraphqlRequest,
    pub response: Value,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OfferDetails {
    pub schema_version: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub context: Option<retailer_sauce::shopify::markets::MarketContext>,
    pub product: Value,
    pub pages: Vec<Page>,
}
