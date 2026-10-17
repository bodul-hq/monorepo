//! Version 1 of the sitemap.

use crate::anker_from_location;
use bodul_shared::SitemapConfig;
use bodul_shared::link::LinkKind;
use std::time::Duration;

pub fn sitemap_config() -> SitemapConfig {
    SitemapConfig {
        sitemap_url: vec!["https://www.anker.com/sitemap.xml".to_string()],
    }
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

pub fn classify_link(url: &str, source: &str, _image_count: usize) -> LinkKind {
    from_location(url, source)
}

pub fn from_location(url: &str, source: &str) -> LinkKind {
    let path = url.to_lowercase();

    if !path.starts_with("https://www.anker.com/fr/") {
        return LinkKind::NotInterested;
    }

    match source {
        "https://www.anker.com/sitemap-0.xml" => LinkKind::NotInterested,
        "https://www.anker.com/server-sitemap-index-pages.xml" => LinkKind::NotInterested,
        "https://www.anker.com/server-sitemap-index-products.xml" => LinkKind::Product,
        "https://www.anker.com/server-sitemap-index-collections.xml" => LinkKind::Catalog,
        "https://www.anker.com/server-sitemap-index-blog.xml" => LinkKind::Content,
        _ => anker_from_location(&path),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const BLOG_SOURCE: &str = "https://www.anker.com/server-sitemap-index-blog.xml";
    const COLLECTION_SOURCE: &str = "https://www.anker.com/server-sitemap-index-collections.xml";
    const PRODUCT_SOURCE: &str = "https://www.anker.com/server-sitemap-index-products.xml";
    const ROOT_SOURCE: &str = "https://www.anker.com/sitemap-0.xml";

    #[test]
    fn unmatched_source_still_rejects_out_of_region_urls() {
        for url in [
            "https://www.anker.com/agents.md",
            "https://www.anker.com/products/a110a",
        ] {
            assert_eq!(
                classify_link(url, "https://www.anker.com/sitemap.xml", 0),
                LinkKind::NotInterested,
                "for {url}"
            );
        }
    }

    #[test]
    fn classifies_grouped_sitemap_examples() {
        let cases = [
            (
                "https://www.anker.com/fr/products/a110a-anker-prime-26k-300w-power-bank",
                PRODUCT_SOURCE,
                LinkKind::Product,
            ),
            (
                "https://www.anker.com/fr/collections/100w-gan-charger",
                COLLECTION_SOURCE,
                LinkKind::Catalog,
            ),
            (
                "https://www.anker.com/fr/blogs/anker-guides/30w-usb-c-charger",
                BLOG_SOURCE,
                LinkKind::Content,
            ),
            (
                "https://www.anker.com",
                ROOT_SOURCE,
                LinkKind::NotInterested,
            ),
            ("https://www.anker.com/fr/agents.md", "", LinkKind::Unknown),
        ];

        for (url, source, expected) in cases {
            assert_eq!(classify_link(url, source, 0), expected, "url: {url}");
        }
    }

    #[test]
    fn handles_grouped_sitemap_edge_cases() {
        let cases = [
            (
                "https://www.anker.com/fr/collections/2026年pd抽奖20-折扣",
                COLLECTION_SOURCE,
                LinkKind::Catalog,
            ),
            (
                "https://www.anker.com/fr/blogs/anker-guides",
                BLOG_SOURCE,
                LinkKind::Content,
            ),
            (
                "https://www.anker.com/fr/anker-solix/kit-solaire-hes",
                ROOT_SOURCE,
                LinkKind::NotInterested,
            ),
            (
                "https://www.anker.com/eu-en/products/a110a-anker-prime-26k-300w-power-bank",
                PRODUCT_SOURCE,
                LinkKind::NotInterested,
            ),
        ];

        for (url, source, expected) in cases {
            assert_eq!(classify_link(url, source, 0), expected, "url: {url}");
        }
    }
}
