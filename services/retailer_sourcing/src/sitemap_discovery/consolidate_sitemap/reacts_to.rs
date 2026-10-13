use super::command::ConsolidateSitemap;
use crate::sitemap_discovery::process_sitemap::prelude::SitemapProcessed;

/// Consolidates every sitemap as soon as it has been processed.
pub fn on(event: &SitemapProcessed) -> Option<ConsolidateSitemap> {
    Some(ConsolidateSitemap {
        processed_sitemap_id: event.processed_sitemap_id,
    })
}

#[cfg(test)]
mod tests {
    use bodul_shared::retailer::RetailerCode;
    use uuid::Uuid;

    use super::*;

    #[test]
    fn consolidates_the_sitemap_that_was_processed() {
        let processed_sitemap_id = Uuid::now_v7();
        let event = SitemapProcessed {
            retrieval_id: Uuid::now_v7(),
            processed_sitemap_id,
            retailer_code: RetailerCode::UgreenCa,
            url_count: 3,
        };

        assert_eq!(
            on(&event),
            Some(ConsolidateSitemap {
                processed_sitemap_id
            })
        );
    }
}
