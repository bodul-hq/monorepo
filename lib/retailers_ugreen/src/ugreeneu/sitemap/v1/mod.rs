//! Version 1 of the sitemap.

use bodul_shared::SitemapConfig;
use bodul_shared::link::LinkKind;
use retailer_sauce::shopify::shopify_from_location;
use std::time::Duration;

pub fn sitemap_config() -> SitemapConfig {
    SitemapConfig {
        sitemap_url: vec!["https://eu.ugreen.com/sitemap.xml".to_string()],
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
                "https://eu.ugreen.com/products/10000mah-power-bank-integrated-usb-c-cable",
                LinkKind::Product,
            ),
            (
                "https://eu.ugreen.com/collections/3c-accessories",
                LinkKind::Catalog,
            ),
            (
                "https://eu.ugreen.com/blogs/bateria-externa",
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
                "https://eu.ugreen.com/da-dk/products/10000mah-powerbank-med-integreret-usb-c-kabel",
                LinkKind::Product,
            ),
            ("https://eu.ugreen.com/", LinkKind::Unknown),
            ("https://eu.ugreen.com/sv-se", LinkKind::Unknown),
        ];

        for (url, expected) in cases {
            assert_eq!(from_location(url), expected, "for {url}");
        }
    }
}
