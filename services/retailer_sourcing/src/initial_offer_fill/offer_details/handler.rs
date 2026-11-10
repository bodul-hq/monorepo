use super::{SourceType, command::FetchOfferDetails, event::OfferDetailsFetched, fetch};
use crate::initial_offer_fill::transport::Http;
use crate::platform::{
    bus::Handle,
    object_storage::{ObjectStorage, OfferStorageMetadata},
};

pub const BUCKET: &str = "offer_details_raw";
pub trait Store: Send + Sync + 'static {
    fn store(
        &self,
        body: &str,
        fetched_at: chrono::DateTime<chrono::Utc>,
    ) -> Result<OfferStorageMetadata, String>;
}
impl Store for ObjectStorage {
    fn store(
        &self,
        body: &str,
        fetched_at: chrono::DateTime<chrono::Utc>,
    ) -> Result<OfferStorageMetadata, String> {
        let mut metadata = self.create_offer(BUCKET, body).map_err(|e| e.to_string())?;
        metadata.scraped_at = Some(fetched_at);
        Ok(metadata)
    }
}
pub struct Handler<F, S> {
    pub fetcher: F,
    pub store: S,
}
impl<F: Http + Send + Sync + 'static, S: Store> Handle for Handler<F, S> {
    type Command = FetchOfferDetails;
    type Event = OfferDetailsFetched;
    type Error = String;
    fn handle(&self, command: Self::Command) -> Result<Vec<Self::Event>, String> {
        let SourceType::GraphQl { product_id } = &command.source else {
            return Err("HTML initial-fill details are not implemented".into());
        };
        let body = fetch::fetch_in_context(
            &self.fetcher,
            command.retailer_code,
            command.storefront_token.as_ref(),
            product_id,
            command.context.as_ref(),
        )?;
        let fetched_at = chrono::Utc::now();
        let storage = self.store.store(&body, fetched_at)?;
        tracing::info!(run_id = %command.run_id, context = ?command.context, product_id, object_storage_id = %storage.object_storage_id, "complete offer details stored");
        Ok(vec![OfferDetailsFetched {
            context: command.context,
            run_id: command.run_id,
            retailer_code: command.retailer_code,
            source: command.source,
            fetched_at,
            storage,
        }])
    }
}
