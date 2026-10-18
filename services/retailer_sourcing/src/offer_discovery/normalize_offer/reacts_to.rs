use super::command::NormalizeOffer;
use crate::offer_discovery::extract_offer_page::prelude::OfferPageWasExtracted;

/// Normalizes every offer as soon as its page is extracted.
pub fn on(event: &OfferPageWasExtracted) -> Option<NormalizeOffer> {
    Some(NormalizeOffer {
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
    fn normalizes_the_offer_that_was_extracted() {
        let event = OfferPageWasExtracted {
            offer_id: Uuid::now_v7(),
            consolidated_sitemap_id: Uuid::now_v7(),
            retailer_code: RetailerCode::AnkerAu,
            url: "https://shop.test/products/a".to_string(),
        };

        assert_eq!(
            on(&event),
            Some(NormalizeOffer {
                offer_id: event.offer_id,
                consolidated_sitemap_id: event.consolidated_sitemap_id,
                retailer_code: RetailerCode::AnkerAu,
                url: event.url.clone(),
            })
        );
    }
}
