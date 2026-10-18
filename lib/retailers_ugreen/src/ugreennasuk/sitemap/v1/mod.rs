//! Version 1 of the sitemap.

use bodul_shared::SitemapConfig;
use bodul_shared::link::LinkKind;
use retailer_sauce::shopify::shopify_from_location;
use std::time::Duration;

pub fn sitemap_config() -> SitemapConfig {
    SitemapConfig {
        sitemap_url: vec!["https://nas-uk.ugreen.com/sitemap.xml".to_string()],
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
    fn classifies_grouped_sitemap_examples() {
        let cases = [
            (
                "https://nas-uk.ugreen.com/products/ugreen-nasync-dh2300-nas-storage",
                LinkKind::Product,
            ),
            (
                "https://nas-uk.ugreen.com/collections/nas-storage",
                LinkKind::Catalog,
            ),
            (
                "https://nas-uk.ugreen.com/blogs/buying-guide",
                LinkKind::Content,
            ),
            ("https://nas-uk.ugreen.com/", LinkKind::Unknown),
        ];

        for (url, expected) in cases {
            assert_eq!(from_location(url), expected, "for {url}");
        }
    }

    #[test]
    fn handles_grouped_sitemap_edge_cases() {
        let cases = [
            (
                "https://nas-uk.ugreen.com/products/ugreen-nasync-dxp8800-plus-8-bay-nas-256tb",
                LinkKind::Product,
            ),
            (
                "https://nas-uk.ugreen.com/products/worry-free-purchase",
                LinkKind::Product,
            ),
            (
                "https://nas-uk.ugreen.com/blogs/buying-guide/40-valentines-day-gifts-for-him-tech",
                LinkKind::Content,
            ),
            ("https://nas-uk.ugreen.com/agents.md", LinkKind::Unknown),
        ];

        for (url, expected) in cases {
            assert_eq!(from_location(url), expected, "for {url}");
        }
    }
}
