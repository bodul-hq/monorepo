//! Version 1 of the sitemap.

use bodul_shared::SitemapConfig;
use bodul_shared::link::LinkKind;
use retailer_sauce::shopify::shopify_from_location;
use std::time::Duration;

pub fn sitemap_config() -> SitemapConfig {
    SitemapConfig {
        sitemap_url: vec!["https://ca.ugreen.com/sitemap.xml".to_string()],
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
                "https://ca.ugreen.com/products/100w-3c1a-gan-fast-charger",
                LinkKind::Product,
            ),
            (
                "https://ca.ugreen.com/collections/2023-labor-day",
                LinkKind::Catalog,
            ),
            (
                "https://ca.ugreen.com/blogs/bluetooth-adapter",
                LinkKind::Content,
            ),
        ];

        for (url, expected) in cases {
            assert_eq!(from_location(url), expected, "for {url}");
        }
    }

    #[test]
    fn handles_fixture_edge_cases() {
        let cases = [
            (
                "https://ca.ugreen.com/fr/products/100w-3c1a-gan-fast-charger",
                LinkKind::Product,
            ),
            ("https://ca.ugreen.com/", LinkKind::Unknown),
            ("https://ca.ugreen.com/fr", LinkKind::Unknown),
        ];

        for (url, expected) in cases {
            assert_eq!(from_location(url), expected, "for {url}");
        }
    }
}
