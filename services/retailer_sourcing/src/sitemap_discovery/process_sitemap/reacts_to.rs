use super::command::ProcessSitemap;
use crate::sitemap_discovery::retrieve_sitemap::prelude::SitemapRetrieved;

/// Processes every retrieval as soon as its sitemap files are stored.
pub fn on(event: &SitemapRetrieved) -> Option<ProcessSitemap> {
    Some(ProcessSitemap {
        retrieval_id: event.retrieval_id,
        retailer_code: event.retailer_code,
    })
}

#[cfg(test)]
mod tests {
    use bodul_shared::retailer::RetailerCode;
    use uuid::Uuid;

    use super::*;

    #[test]
    fn processes_the_retrieval_that_was_retrieved() {
        let retrieval_id = Uuid::now_v7();
        let event = SitemapRetrieved {
            retrieval_id,
            retailer_code: RetailerCode::UgreenCa,
            document_count: 3,
        };

        assert_eq!(
            on(&event),
            Some(ProcessSitemap {
                retrieval_id,
                retailer_code: RetailerCode::UgreenCa,
            })
        );
    }
}
