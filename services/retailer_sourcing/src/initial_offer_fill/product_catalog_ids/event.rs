use crate::platform::bus::Event;
use retailer_sauce::shopify::initial_fill::StorefrontToken;
use serde::{Deserialize, Serialize};
use shared::retailer::RetailerCode;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProductCatalogIdsRetrieved {
    pub run_id: Uuid,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub context: Option<retailer_sauce::shopify::markets::MarketContext>,
    pub retailer_code: RetailerCode,
    pub storefront_token: Option<StorefrontToken>,
    pub after: Option<String>,
    pub product_ids: Vec<String>,
    pub next_cursor: Option<String>,
    pub retrieved_at: chrono::DateTime<chrono::Utc>,
}
impl Event for ProductCatalogIdsRetrieved {
    const NAME: &'static str = "ProductCatalogIdsRetrieved";
}
