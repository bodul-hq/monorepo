use super::{command::FetchProductCatalogIdentifiers, event::ProductCatalogIdsRetrieved};
use crate::initial_offer_fill::transport::Http;
use crate::platform::bus::Handle;

pub struct Handler<F>(pub F);
impl<F: Http + Send + Sync + 'static> Handle for Handler<F> {
    type Command = FetchProductCatalogIdentifiers;
    type Event = ProductCatalogIdsRetrieved;
    type Error = String;
    fn handle(&self, command: Self::Command) -> Result<Vec<Self::Event>, String> {
        let integration =
            retailer_data_extractor::initial_fill_integration_for(command.retailer_code)
                .ok_or("initial fill is not configured")?;
        let batch = integration.catalog(
            &self.0,
            command.storefront_token.as_ref(),
            command.context.as_ref(),
            command.after.clone(),
        )?;
        let product_ids = batch.product_ids;
        let after = batch.next_cursor;
        tracing::info!(run_id = %command.run_id, context = ?command.context, count = product_ids.len(), has_more = after.is_some(), "catalog identifiers retrieved");
        Ok(vec![ProductCatalogIdsRetrieved {
            context: command.context,
            run_id: command.run_id,
            retailer_code: command.retailer_code,
            storefront_token: command.storefront_token,
            after: command.after,
            product_ids,
            next_cursor: after,
            retrieved_at: chrono::Utc::now(),
        }])
    }
}
