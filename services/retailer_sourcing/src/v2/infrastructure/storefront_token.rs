use crate::platform::http::Client;
use retailer_sauce::prelude::{Http as Fetch, StorefrontToken};
use std::env::VarError;
use std::env::var;

impl Fetch for Client {
    fn configured_token(&self, name: &str) -> Result<Option<StorefrontToken>, String> {
        match var(name) {
            Ok(value) => StorefrontToken::new(value)
                .map(Some)
                .map_err(|_| format!("invalid {name}")),
            Err(VarError::NotPresent) => Ok(None),
            Err(_) => Err(format!("invalid {name}")),
        }
    }
    fn get(&self, url: &str) -> Result<String, String> {
        Client::get_storefront(self, url).map_err(|e| e.to_string())
    }
}
