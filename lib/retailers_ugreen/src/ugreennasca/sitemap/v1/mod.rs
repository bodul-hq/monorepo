//! Version 1 of the sitemap.

use bodul_shared::SitemapConfig;
use bodul_shared::link::LinkKind;
use retailer_sauce::shopify::shopify_from_location;
use std::time::Duration;

pub fn sitemap_config() -> SitemapConfig {
    SitemapConfig {
        sitemap_url: vec!["https://nas-ca.ugreen.com/sitemap.xml".to_string()],
    }
}

/// Whether the crawl downloads the child sitemap at `url`. Root sitemaps are
/// always downloaded; drop one from [`sitemap_config`] instead.
pub fn sitemap_greylisted(url: &str) -> bool {
    // we are skipping any sitemap file having `metaobject` in filename
    !url.contains("metaobject")
}

/// How long the crawl pauses between groups of sitemap downloads.
pub fn sitemap_pause() -> Duration {
    Duration::from_secs(1)
}

pub fn classify_link(url: &str, _source: &str, _image_count: usize) -> LinkKind {
    from_location(url)
}

pub fn from_location(url: &str) -> LinkKind {
    shopify_from_location(url)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifies_fixture_urls() {
        let cases = [
            (
                "https://nas-ca.ugreen.com/products/ugreen-nasync-dxp4800-pro-136tb-4-bay-nas",
                LinkKind::Product,
            ),
            (
                "https://nas-ca.ugreen.com/collections/compare",
                LinkKind::Catalog,
            ),
            (
                "https://nas-ca.ugreen.com/blogs/knowledge",
                LinkKind::Content,
            ),
            ("https://nas-ca.ugreen.com/agents.md", LinkKind::Unknown),
        ];

        for (url, expected) in cases {
            assert_eq!(from_location(url), expected, "for {url}");
        }
    }

    #[test]
    fn classifies_numeric_and_root_fixture_urls() {
        let cases = [
            (
                "https://nas-ca.ugreen.com/pages/4-bay-nas-home-storage-ugreen-dxp4800-plus",
                LinkKind::Content,
            ),
            ("https://nas-ca.ugreen.com/", LinkKind::Unknown),
        ];

        for (url, expected) in cases {
            assert_eq!(from_location(url), expected, "for {url}");
        }
    }
}
