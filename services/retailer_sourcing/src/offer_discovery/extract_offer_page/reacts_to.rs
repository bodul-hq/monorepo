use super::command::ExtractOfferPage;
use crate::offer_discovery::download_offer_page::prelude::OfferPageWasDownloaded;

/// Extracts every offer page as soon as it is downloaded.
pub fn on(event: &OfferPageWasDownloaded) -> Option<ExtractOfferPage> {
    Some(ExtractOfferPage {
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
    fn extracts_the_page_that_was_downloaded() {
        let event = OfferPageWasDownloaded {
            offer_id: Uuid::now_v7(),
            consolidated_sitemap_id: Uuid::now_v7(),
            retailer_code: RetailerCode::AnkerAu,
            url: "https://shop.test/products/a".to_string(),
        };

        assert_eq!(
            on(&event),
            Some(ExtractOfferPage {
                offer_id: event.offer_id,
                consolidated_sitemap_id: event.consolidated_sitemap_id,
                retailer_code: RetailerCode::AnkerAu,
                url: event.url.clone(),
            })
        );
    }
}
