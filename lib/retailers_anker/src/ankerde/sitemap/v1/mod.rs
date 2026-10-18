//! Version 1 of the sitemap.

use crate::anker_from_location;
use bodul_shared::SitemapConfig;
use bodul_shared::link::LinkKind;
use std::time::Duration;

pub fn sitemap_config() -> SitemapConfig {
    SitemapConfig {
        sitemap_url: vec!["https://www.anker.com/de/sitemap.xml".to_string()],
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

    if !path.starts_with("https://www.anker.com/de/") {
        return LinkKind::NotInterested;
    }

    match source {
        "https://www.anker.com/de/server-sitemap-index-pages.xml" => LinkKind::NotInterested,
        "https://www.anker.com/de/server-sitemap-index-products.xml" => LinkKind::Product,
        "https://www.anker.com/de/server-sitemap-index-collections.xml" => LinkKind::Catalog,
        "https://www.anker.com/de/server-sitemap-index-blog.xml" => LinkKind::Content,
        _ => anker_from_location(&path),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const PRODUCTS_INDEX: &str = "https://www.anker.com/de/server-sitemap-index-products.xml";
    const COLLECTIONS_INDEX: &str = "https://www.anker.com/de/server-sitemap-index-collections.xml";
    const BLOG_INDEX: &str = "https://www.anker.com/de/server-sitemap-index-blog.xml";
    const PAGES_INDEX: &str = "https://www.anker.com/de/server-sitemap-index-pages.xml";

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
    fn classifies_products() {
        let urls = ["https://www.anker.com/de/products/a110a-anker-prime-powerbank-26250mah-300w"];

        for url in urls {
            assert_eq!(
                classify_link(url, PRODUCTS_INDEX, 0),
                LinkKind::Product,
                "for {url}"
            );
        }
    }

    #[test]
    fn classifies_catalogs() {
        let urls = ["https://www.anker.com/de/collections/1-2-phone-charges"];

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
        let urls = ["https://www.anker.com/de/blogs/balkonkraftwerk"];

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
        // The pages sitemap is never interesting, even under /de/.
        let url = "https://www.anker.com/de/pages/impressum";
        assert_eq!(
            classify_link(url, PAGES_INDEX, 0),
            LinkKind::NotInterested,
            "for {url}"
        );
    }

    #[test]
    fn region_guard_rejects_other_region() {
        // A source matches, but the url is not under /de/, so it is rejected.
        let urls = [
            "https://www.anker.com/2023-anker-prime",
            "https://www.anker.com/au/products/a110a",
        ];

        for url in urls {
            assert_eq!(
                classify_link(url, PRODUCTS_INDEX, 0),
                LinkKind::NotInterested,
                "for {url}"
            );
        }
    }

    #[test]
    fn unmatched_source_falls_back_to_path() {
        // An unmatched source bypasses the region guard and classifies purely by path.
        assert_eq!(
            classify_link(
                "https://www.anker.com/de/products/a121c-anker-zolo-ladegerat-70w-4-ports",
                "https://www.anker.com/de/sitemap.xml",
                0,
            ),
            LinkKind::Product,
            "unmatched source with /products/ path is a Product",
        );
        assert_eq!(
            classify_link(
                "https://www.anker.com/de/agents.md",
                "https://www.anker.com/de/sitemap.xml",
                0
            ),
            LinkKind::Unknown,
            "unmatched source with unknown path is Unknown",
        );
    }

    #[test]
    fn path_matching_is_case_insensitive() {
        // The region guard lowercases the url, so uppercase urls still match /de/.
        let urls = ["HTTPS://WWW.ANKER.COM/DE/PRODUCTS/A110A"];

        for url in urls {
            assert_eq!(
                classify_link(url, PRODUCTS_INDEX, 0),
                LinkKind::Product,
                "for {url}"
            );
        }
    }
}
