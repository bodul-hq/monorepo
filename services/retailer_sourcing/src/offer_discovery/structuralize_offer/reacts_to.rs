use super::command::StructuralizeOffer;
use crate::offer_discovery::normalize_offer::prelude::OfferWasNormalized;

/// Structuralizes every offer as soon as it is normalized.
pub fn on(event: &OfferWasNormalized) -> Option<StructuralizeOffer> {
    Some(StructuralizeOffer {
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
    fn structuralizes_the_offer_that_was_normalized() {
        let event = OfferWasNormalized {
            offer_id: Uuid::now_v7(),
            consolidated_sitemap_id: Uuid::now_v7(),
            retailer_code: RetailerCode::AnkerAu,
            url: "https://shop.test/products/a".to_string(),
        };

        assert_eq!(
            on(&event),
            Some(StructuralizeOffer {
                offer_id: event.offer_id,
                consolidated_sitemap_id: event.consolidated_sitemap_id,
                retailer_code: RetailerCode::AnkerAu,
                url: event.url.clone(),
            })
        );
    }
}
