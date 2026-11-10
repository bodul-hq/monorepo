use crate::platform::bus::Command;
use retailer_sauce::shopify::initial_fill::StorefrontToken;
use serde::{Deserialize, Serialize};
use shared::retailer::RetailerCode;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FetchProductCatalogIdentifiers {
    pub run_id: Uuid,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub context: Option<retailer_sauce::shopify::markets::MarketContext>,
    pub retailer_code: RetailerCode,
    pub storefront_token: Option<StorefrontToken>,
    pub after: Option<String>,
}
impl Command for FetchProductCatalogIdentifiers {
    const NAME: &'static str = "FetchProductCatalogIdentifiers";
}
