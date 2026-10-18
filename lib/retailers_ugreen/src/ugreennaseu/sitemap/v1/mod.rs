//! Version 1 of the sitemap.

use bodul_shared::SitemapConfig;
use bodul_shared::link::LinkKind;
use retailer_sauce::shopify::shopify_from_location;
use std::time::Duration;

pub fn sitemap_config() -> SitemapConfig {
    SitemapConfig {
        sitemap_url: vec!["https://nas-eu.ugreen.com/sitemap.xml".to_string()],
    }
}

/// Whether the crawl downloads the child sitemap at `url`. Root sitemaps are
/// always downloaded; drop one from [`sitemap_config`] instead.
pub fn sitemap_greylisted(url: &str) -> bool {
    // we are skipping any sitemap file having `metaobject` in filename
    !url.contains("metaobject")
}

/// How long the crawl pauses between groups of sitemap downloads. The store rate-limits
/// quick crawls, so it gets a longer pause.
pub fn sitemap_pause() -> Duration {
    Duration::from_secs(5)
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
                "https://nas-eu.ugreen.com/products/worry-free-purchase",
                LinkKind::Product,
            ),
            (
                "https://nas-eu.ugreen.com/collections/compare",
                LinkKind::Catalog,
            ),
            (
                "https://nas-eu.ugreen.com/blogs/buying-guide",
                LinkKind::Content,
            ),
            ("https://nas-eu.ugreen.com/agents.md", LinkKind::Unknown),
        ];

        for (url, expected) in cases {
            assert_eq!(from_location(url), expected, "for {url}");
        }
    }

    #[test]
    fn classifies_locale_prefixed_nested_fixture_urls() {
        let cases = [
            (
                "https://nas-eu.ugreen.com/de/products/ugreen-nas-10gbps-m2-nvme-ssd-enclosure",
                LinkKind::Product,
            ),
            (
                "https://nas-eu.ugreen.com/nl-nl/collections/raid-calculator-recommended",
                LinkKind::Catalog,
            ),
            (
                "https://nas-eu.ugreen.com/de/blogs/buying-guide/40-regalos-tecnologicos-san-valentin-para-el",
                LinkKind::Content,
            ),
            ("https://nas-eu.ugreen.com/en-es", LinkKind::Unknown),
        ];

        for (url, expected) in cases {
            assert_eq!(from_location(url), expected, "for {url}");
        }
    }
}
