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

mod document;
mod parse;
mod schema;

mod consolidate_sitemap;
mod process_sitemap;
mod retrieve_sitemap;

use bodul_shared::SitemapConfig;
use bodul_shared::link::LinkKind;
use bodul_shared::retailer::RetailerCode;

/// The `source` given to URLs that don't come from a known sitemap file.
const ROOT_SITEMAP_SOURCE: &str = "main";

/// How deep sitemap indexes may nest before a branch is cut off.
const MAX_SITEMAP_DEPTH: usize = 32;

/// The retailer registry: each retailer's root sitemaps and link rules.
#[derive(Debug, Clone, Copy, Default)]
struct Retailers;

impl Retailers {
    fn sitemap_config(self, retailer_code: &RetailerCode) -> Option<SitemapConfig> {
        retailer_data_extractor::sitemap_config_for(retailer_code)
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
