use super::command::ProcessOfferForExport;
use crate::offer_discovery::structuralize_offer::prelude::OfferWasStructuralized;

pub fn on(event: &OfferWasStructuralized) -> Option<ProcessOfferForExport> {
    Some(ProcessOfferForExport {
        offer_id: event.offer_id,
        consolidated_sitemap_id: event.consolidated_sitemap_id,
        retailer_code: event.retailer_code,
        url: event.url.clone(),
        storage: event.storage,
    })
}
