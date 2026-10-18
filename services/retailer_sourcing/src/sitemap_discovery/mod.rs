//! Sitemap discovery.
//!
//! Crawls a retailer's sitemaps and sorts every page URL they list into
//! products, catalogs, content and the rest. Three feature slices run in
//! sequence, each started by the previous one's event:
//!
//! 1. [`retrieve_sitemap`] fetches the root sitemaps and every child sitemap.
//! 2. [`process_sitemap`] parses the stored files into one sitemap tree.
//! 3. [`consolidate_sitemap`] sorts each URL in the tree by kind of page, using
//!    the retailer's rules.
//!
//! This module holds the domain shared by the slices. Other contexts may only
//! use [`prelude`].

pub mod prelude;

mod schema;

mod consolidate_sitemap;
mod process_sitemap;
mod retrieve_sitemap;

use std::time::Duration;

use bodul_shared::SitemapConfig;
use bodul_shared::link::LinkKind;
use bodul_shared::retailer::RetailerCode;

/// The pause between groups of sitemap downloads for a retailer that doesn't
/// set its own.
const DEFAULT_SITEMAP_PAUSE: Duration = Duration::from_secs(1);

/// The retailer registry: each retailer's root sitemaps and link rules.
#[derive(Debug, Clone, Copy, Default)]
struct Retailers;

impl Retailers {
    fn sitemap_config(self, retailer_code: &RetailerCode) -> Option<SitemapConfig> {
        retailer_data_extractor::sitemap_config_for(retailer_code)
    }

    /// Whether the retailer's crawl downloads the child sitemap at `url`. A
    /// retailer this crate doesn't know downloads every one.
    fn sitemap_greylisted(self, retailer_code: RetailerCode, url: &str) -> bool {
        retailer_data_extractor::sitemap_greylisted_for(retailer_code, url).unwrap_or(true)
    }

    /// How long the retailer's crawl pauses between groups of downloads.
    fn sitemap_pause(self, retailer_code: RetailerCode) -> Duration {
        retailer_data_extractor::sitemap_pause_for(retailer_code).unwrap_or(DEFAULT_SITEMAP_PAUSE)
    }

    fn classify_link(
        self,
        retailer_code: RetailerCode,
        url: &str,
        source: &str,
        image_count: usize,
    ) -> LinkKind {
        retailer_data_extractor::classify_link_for(retailer_code, url, source, image_count)
            .unwrap_or(LinkKind::Unknown)
    }
}
