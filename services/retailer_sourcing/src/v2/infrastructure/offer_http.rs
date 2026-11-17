//! Paced HTTP only. Each retailer builds and validates its own requests.
use crate::platform::http::OfferClient;
use retailer_sauce::prelude::GraphqlRequest;
use retailer_sauce::prelude::Http;
use retailer_sauce::prelude::StorefrontToken;
use serde_json::Value;
use serde_json::from_str;
use serde_json::to_string;
use std::thread::sleep;
use std::time::Duration;
impl Http for OfferClient {
    fn post(
        &self,
        url: &str,
        request: &GraphqlRequest,
        token: Option<&StorefrontToken>,
    ) -> Result<Value, String> {
        let body = to_string(request).map_err(|e| e.to_string())?;
        sleep(Duration::from_secs(6));
        let response = match token {
            Some(token) => self.post_storefront_json(url, &body, token.as_str()),
            None => self.post_storefront_json_with_access(url, &body, None),
        }
        .map_err(|e| e.to_string())?;
        from_str(&response).map_err(|_| "Storefront returned invalid JSON".into())
    }
}
