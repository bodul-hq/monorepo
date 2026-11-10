//! Paced HTTP only. Each retailer builds and validates its own requests.
use crate::platform::http::OfferClient;
pub use retailer_sauce::initial_fill::Http;
use retailer_sauce::shopify::initial_fill::{GraphqlRequest, StorefrontToken};
use serde_json::Value;
impl Http for OfferClient {
    fn post(
        &self,
        url: &str,
        request: &GraphqlRequest,
        token: Option<&StorefrontToken>,
    ) -> Result<Value, String> {
        let body = serde_json::to_string(request).map_err(|e| e.to_string())?;
        std::thread::sleep(std::time::Duration::from_secs(6));
        let response = match token {
            Some(token) => self.post_storefront_json(url, &body, token.as_str()),
            None => self.post_storefront_json_with_access(url, &body, None),
        }
        .map_err(|e| e.to_string())?;
        serde_json::from_str(&response).map_err(|_| "Storefront returned invalid JSON".into())
    }
}
