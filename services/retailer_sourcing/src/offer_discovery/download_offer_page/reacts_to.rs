use super::command::DownloadOfferPage;
use crate::offer_discovery::discover_offers::prelude::OfferDiscovered;

/// Downloads every offer's page as soon as the offer is discovered.
pub fn on(event: &OfferDiscovered) -> Option<DownloadOfferPage> {
    Some(DownloadOfferPage {
        offer_id: event.offer_id,
        consolidated_sitemap_id: event.consolidated_sitemap_id,
        retailer_code: event.retailer_code,
        url: event.url.clone(),
    })
}

#[cfg(test)]
mod tests {
    use bodul_shared::retailer::RetailerCode;
    use uuid::Uuid;

    use super::*;

    #[test]
    fn downloads_the_offer_that_was_discovered() {
        let event = OfferDiscovered {
            offer_id: Uuid::now_v7(),
            consolidated_sitemap_id: Uuid::now_v7(),
            retailer_code: RetailerCode::UgreenCa,
            url: "https://shop.test/products/a".to_string(),
        };

        assert_eq!(
            on(&event),
            Some(DownloadOfferPage {
                offer_id: event.offer_id,
                consolidated_sitemap_id: event.consolidated_sitemap_id,
                retailer_code: RetailerCode::UgreenCa,
                url: event.url.clone(),
            })
        );
    }
}
