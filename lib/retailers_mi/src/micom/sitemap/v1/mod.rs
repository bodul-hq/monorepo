//! Version 1 of the sitemap.

use bodul_shared::SitemapConfig;
use bodul_shared::link::LinkKind;
use std::time::Duration;

pub fn sitemap_config() -> SitemapConfig {
    todo!()
}

/// Whether the crawl downloads the child sitemap at `url`. Root sitemaps are
/// always downloaded; drop one from [`sitemap_config`] instead.
pub fn sitemap_greylisted(_url: &str) -> bool {
    true
}

/// How long the crawl pauses between groups of sitemap downloads.
pub fn sitemap_pause() -> Duration {
    Duration::from_secs(1)
}

pub fn classify_link(_url: &str, _source: &str, _image_count: usize) -> LinkKind {
    todo!()
}
