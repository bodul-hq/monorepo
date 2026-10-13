use super::command::DiscoverOffers;
use crate::sitemap_discovery::prelude::SitemapConsolidated;

/// Discovers offers in every consolidated sitemap that lists products.
pub fn on(event: &SitemapConsolidated) -> Option<DiscoverOffers> {
    if event.product_count == 0 {
        return None;
    }
    Some(DiscoverOffers {
        consolidated_sitemap_id: event.consolidated_sitemap_id,
        retailer_code: event.retailer_code,
    })
}

#[cfg(test)]
mod tests {
    use bodul_shared::retailer::RetailerCode;
    use uuid::Uuid;

    use super::*;

    fn consolidated(product_count: usize) -> SitemapConsolidated {
        SitemapConsolidated {
            consolidated_sitemap_id: Uuid::now_v7(),
            processed_sitemap_id: Uuid::now_v7(),
            retrieval_id: Uuid::now_v7(),
            retailer_code: RetailerCode::UgreenCa,
            product_count,
            catalog_count: 0,
            content_count: 0,
            not_interested_count: 0,
            unknown_count: 0,
        }
    }

    #[test]
    fn discovers_offers_in_a_sitemap_with_products() {
        let event = consolidated(2);

        assert_eq!(
            on(&event),
            Some(DiscoverOffers {
                consolidated_sitemap_id: event.consolidated_sitemap_id,
                retailer_code: RetailerCode::UgreenCa,
            })
        );
    }

    #[test]
    fn ignores_a_sitemap_without_products() {
        assert_eq!(on(&consolidated(0)), None);
    }
}
