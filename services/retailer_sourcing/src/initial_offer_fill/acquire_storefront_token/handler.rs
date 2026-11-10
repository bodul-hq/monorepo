use super::{command::AcquireStorefrontToken, event::StorefrontTokenAcquired};
use crate::platform::{bus::Handle, http::Client};
use retailer_sauce::shopify::initial_fill::StorefrontToken;

pub use retailer_sauce::initial_fill::Http as Fetch;
impl Fetch for Client {
    fn configured_token(&self, name: &str) -> Result<Option<StorefrontToken>, String> {
        match std::env::var(name) {
            Ok(value) => StorefrontToken::new(value)
                .map(Some)
                .map_err(|_| format!("invalid {name}")),
            Err(std::env::VarError::NotPresent) => Ok(None),
            Err(_) => Err(format!("invalid {name}")),
        }
    }
    fn get(&self, url: &str) -> Result<String, String> {
        Client::get_storefront(self, url).map_err(|e| e.to_string())
    }
}
pub struct Handler<F>(pub F);
impl<F: Fetch + Send + Sync + 'static> Handle for Handler<F> {
    type Command = AcquireStorefrontToken;
    type Event = StorefrontTokenAcquired;
    type Error = String;
    fn handle(&self, command: AcquireStorefrontToken) -> Result<Vec<Self::Event>, String> {
        let integration =
            retailer_data_extractor::initial_fill_integration_for(command.retailer_code)
                .ok_or("retailer has no initial fill configuration")?;
        let storefront_token = integration.resolve_access(&self.0, &command.url)?;
        tracing::info!(run_id = %command.run_id, retailer = ?command.retailer_code, has_token = storefront_token.is_some(), "Storefront access resolved");
        Ok(vec![StorefrontTokenAcquired {
            run_id: command.run_id,
            retailer_code: command.retailer_code,
            url: command.url,
            storefront_token,
            acquired_at: chrono::Utc::now(),
        }])
    }
}
