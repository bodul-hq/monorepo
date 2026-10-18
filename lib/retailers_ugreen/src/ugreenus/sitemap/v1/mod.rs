//! Version 1 of the sitemap.

use bodul_shared::SitemapConfig;
use bodul_shared::link::LinkKind;
use retailer_sauce::shopify::shopify_from_location;
use std::time::Duration;

pub fn sitemap_config() -> SitemapConfig {
    SitemapConfig {
        sitemap_url: vec!["https://us.ugreen.com/sitemap.xml".to_string()],
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
                "https://us.ugreen.com/products/100w-3c1a-gan-fast-charger",
                LinkKind::Product,
            ),
            (
                "https://us.ugreen.com/collections/accessories",
                LinkKind::Catalog,
            ),
            ("https://us.ugreen.com/blogs/charger", LinkKind::Content),
            ("https://us.ugreen.com/", LinkKind::Unknown),
        ];

        for (url, expected) in cases {
            assert_eq!(from_location(url), expected, "for {url}");
        }
    }

    #[test]
    fn handles_grouped_sitemap_edge_cases() {
        let cases = [
            ("https://us.ugreen.com/products/15375", LinkKind::Product),
            (
                "https://us.ugreen.com/collections/%E6%B5%8B%E8%AF%95%E6%96%B0%E4%B8%93%E8%BE%91%E6%A8%A1%E6%9D%BF",
                LinkKind::Catalog,
            ),
            (
                "https://us.ugreen.com/collections/ugreen-all-products-chargers-power-banks-cables-accessories-%E5%89%AF%E6%9C%AC",
                LinkKind::Catalog,
            ),
            (
                "https://us.ugreen.com/blogs/charger/2025-usb-c-charger-guide",
                LinkKind::Content,
            ),
            ("https://us.ugreen.com/agents.md", LinkKind::Unknown),
        ];

        for (url, expected) in cases {
            assert_eq!(from_location(url), expected, "for {url}");
        }
    }
}
