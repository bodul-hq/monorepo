//! Version 1 of the sitemap.

use bodul_shared::SitemapConfig;
use bodul_shared::link::LinkKind;
use retailer_sauce::shopify::shopify_from_location;
use std::time::Duration;

pub fn sitemap_config() -> SitemapConfig {
    SitemapConfig {
        sitemap_url: vec!["https://uk.ugreen.com/sitemap.xml".to_string()],
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
                "https://uk.ugreen.com/products/240w-usb-c-cable-pd3-1-e-marker-chip",
                LinkKind::Product,
            ),
            (
                "https://uk.ugreen.com/collections/adapter",
                LinkKind::Catalog,
            ),
            (
                "https://uk.ugreen.com/blogs/buying-guide",
                LinkKind::Content,
            ),
            ("https://uk.ugreen.com/", LinkKind::Unknown),
        ];

        for (url, expected) in cases {
            assert_eq!(from_location(url), expected, "for {url}");
        }
    }

    #[test]
    fn handles_grouped_sitemap_edge_cases() {
        let cases = [
            ("https://uk.ugreen.com/products/25286", LinkKind::Product),
            (
                "https://uk.ugreen.com/collections/%E5%8B%BF%E5%88%A0-%E7%94%A8%E4%BA%8E%E9%9A%90%E8%97%8F%E6%90%9C%E7%B4%A2%E7%BB%93%E6%9E%9C",
                LinkKind::Catalog,
            ),
            (
                "https://uk.ugreen.com/collections/3-5mm-audio",
                LinkKind::Catalog,
            ),
            (
                "https://uk.ugreen.com/blogs/buying-guide/how-to-build-a-minimal-travel-charging-kit",
                LinkKind::Content,
            ),
            ("https://uk.ugreen.com/agents.md", LinkKind::Unknown),
        ];

        for (url, expected) in cases {
            assert_eq!(from_location(url), expected, "for {url}");
        }
    }
}
