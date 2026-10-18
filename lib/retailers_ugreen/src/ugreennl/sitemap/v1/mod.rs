//! Version 1 of the sitemap.

use bodul_shared::SitemapConfig;
use bodul_shared::link::LinkKind;
use retailer_sauce::shopify::shopify_from_location;
use std::time::Duration;

pub fn sitemap_config() -> SitemapConfig {
    SitemapConfig {
        sitemap_url: vec!["https://nl.ugreen.com/sitemap.xml".to_string()],
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
                "https://nl.ugreen.com/products/200w-8poorts-gan-desktop-snellader",
                LinkKind::Product,
            ),
            (
                "https://nl.ugreen.com/collections/audiokabel",
                LinkKind::Catalog,
            ),
            ("https://nl.ugreen.com/blogs/adapter", LinkKind::Content),
            ("https://nl.ugreen.com/", LinkKind::Unknown),
        ];

        for (url, expected) in cases {
            assert_eq!(from_location(url), expected, "for {url}");
        }
    }

    #[test]
    fn handles_grouped_sitemap_edge_cases() {
        let cases = [
            ("https://nl.ugreen.com/products/10914", LinkKind::Product),
            (
                "https://nl.ugreen.com/collections/all-products-%E5%89%AF%E6%9C%AC",
                LinkKind::Catalog,
            ),
            (
                "https://nl.ugreen.com/pages/ugreen-revodok-maxidok-thunderbolt%E2%84%A2-5-dockingstation-serie-launch-aanbieding-tot-20-korting-1",
                LinkKind::Content,
            ),
            (
                "https://nl.ugreen.com/blogs/adapter/bluetooth-4-0-vs-5-0",
                LinkKind::Content,
            ),
            ("https://nl.ugreen.com/agents.md", LinkKind::Unknown),
        ];

        for (url, expected) in cases {
            assert_eq!(from_location(url), expected, "for {url}");
        }
    }
}
