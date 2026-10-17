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

    if !path.starts_with("https://www.anker.com/uk/") {
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
    // NOTE: URLs synthesized from sibling anker stores; the ankeruk grouped-sitemap data is empty.
    use super::*;

    const PRODUCTS_INDEX: &str = "https://www.anker.com/server-sitemap-index-products.xml";
    const COLLECTIONS_INDEX: &str = "https://www.anker.com/server-sitemap-index-collections.xml";
    const BLOG_INDEX: &str = "https://www.anker.com/server-sitemap-index-blog.xml";
    const PAGES_INDEX: &str = "https://www.anker.com/server-sitemap-index-pages.xml";
    const SITEMAP_0: &str = "https://www.anker.com/sitemap-0.xml";

    #[test]
    fn classifies_products() {
        let urls = [
            "https://www.anker.com/uk/products/a1259-built-in-cable-power-bank-10000mah",
            "https://www.anker.com/uk/products/a1215",
            "https://www.anker.com/uk/products/a1288",
        ];

        for url in urls {
            assert_eq!(
                classify_link(url, PRODUCTS_INDEX, 0),
                LinkKind::Product,
                "for {url}"
            );
        }
    }

    #[test]
    fn classifies_catalog() {
        let urls = [
            "https://www.anker.com/uk/collections/1-2-phone-charges",
            "https://www.anker.com/uk/collections/10000-mah-power-bank",
            "https://www.anker.com/uk/collections/100w-gan-charger",
        ];

        for url in urls {
            assert_eq!(
                classify_link(url, COLLECTIONS_INDEX, 0),
                LinkKind::Catalog,
                "for {url}"
            );
        }
    }

    #[test]
    fn classifies_content() {
        let urls = ["https://www.anker.com/uk/blogs/news"];

        for url in urls {
            assert_eq!(
                classify_link(url, BLOG_INDEX, 0),
                LinkKind::Content,
                "for {url}"
            );
        }
    }

    #[test]
    fn classifies_not_interested() {
        assert_eq!(
            classify_link("https://www.anker.com/uk/pages/warranty", PAGES_INDEX, 0),
            LinkKind::NotInterested,
            "for pages index"
        );
        assert_eq!(
            classify_link("https://www.anker.com/uk/products/a1215", SITEMAP_0, 0),
            LinkKind::NotInterested,
            "for sitemap-0"
        );
    }

    #[test]
    fn region_guard_rejects_other_region() {
        assert_eq!(
            classify_link("https://www.anker.com/2023-anker-prime", PRODUCTS_INDEX, 0),
            LinkKind::NotInterested,
            "for root-level url"
        );
        assert_eq!(
            classify_link(
                "https://www.anker.com/eu-de/products/a110a",
                PRODUCTS_INDEX,
                0
            ),
            LinkKind::NotInterested,
            "for other-region url"
        );
    }

    #[test]
    fn unmatched_source_falls_back_to_path() {
        assert_eq!(
            classify_link(
                "https://www.anker.com/uk/products/a1215",
                "https://www.anker.com/sitemap.xml",
                0
            ),
            LinkKind::Product,
            "for product path"
        );
        assert_eq!(
            classify_link(
                "https://www.anker.com/uk/agents.md",
                "https://www.anker.com/sitemap.xml",
                0
            ),
            LinkKind::Unknown,
            "for unknown path"
        );
    }

    #[test]
    fn path_matching_is_case_insensitive() {
        assert_eq!(
            classify_link("HTTPS://WWW.ANKER.COM/UK/PRODUCTS/A1215", PRODUCTS_INDEX, 0),
            LinkKind::Product,
            "for uppercased url"
        );
    }
}
