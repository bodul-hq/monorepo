//! Version 1 of the sitemap.

use crate::anker_from_location;
use bodul_shared::SitemapConfig;
use bodul_shared::link::LinkKind;
use std::time::Duration;

pub fn sitemap_config() -> SitemapConfig {
    SitemapConfig {
        sitemap_url: vec!["https://www.ankernordics.com/sitemap.xml".to_string()],
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
    let matced_by_source = match source {
        "https://www.ankernordics.com/sitemap-0.xml" => Some(LinkKind::NotInterested),
        "https://www.ankernordics.com/server-sitemap-index-pages.xml" => {
            Some(LinkKind::NotInterested)
        }
        "https://www.ankernordics.com/server-sitemap-index-products.xml" => Some(LinkKind::Product),
        "https://www.ankernordics.com/server-sitemap-index-collections.xml" => {
            Some(LinkKind::Catalog)
        }
        "https://www.ankernordics.com/server-sitemap-index-blog.xml" => Some(LinkKind::Content),
        _ => None,
    };

    if let Some(y) = matced_by_source {
        return y;
    }

    anker_from_location(url)
}

#[cfg(test)]
mod tests {
    use super::*;

    const SITEMAP_0: &str = "https://www.ankernordics.com/sitemap-0.xml";
    const PAGES_INDEX: &str = "https://www.ankernordics.com/server-sitemap-index-pages.xml";
    const PRODUCTS_INDEX: &str = "https://www.ankernordics.com/server-sitemap-index-products.xml";
    const COLLECTIONS_INDEX: &str =
        "https://www.ankernordics.com/server-sitemap-index-collections.xml";
    const BLOG_INDEX: &str = "https://www.ankernordics.com/server-sitemap-index-blog.xml";

    const UNMATCHED_SOURCE: &str = "https://www.ankernordics.com/sitemap.xml";

    const PRODUCT_URL: &str = "https://www.ankernordics.com/products/3874x-soundcore-aerofit-2-ai-assistant-translate-earbuds";
    const CATALOG_URL: &str = "https://www.ankernordics.com/collections/3-4-phone-charges";
    const CONTENT_URL: &str = "https://www.ankernordics.com/blogs/se/all";
    const ROOT_URL: &str = "https://www.ankernordics.com";

    #[test]
    fn classifies_products() {
        let url = PRODUCT_URL;
        assert_eq!(
            classify_link(url, PRODUCTS_INDEX, 0),
            LinkKind::Product,
            "for {url}"
        );
    }

    #[test]
    fn classifies_catalog() {
        let url = CATALOG_URL;
        assert_eq!(
            classify_link(url, COLLECTIONS_INDEX, 0),
            LinkKind::Catalog,
            "for {url}"
        );
    }

    #[test]
    fn classifies_content() {
        let url = CONTENT_URL;
        assert_eq!(
            classify_link(url, BLOG_INDEX, 0),
            LinkKind::Content,
            "for {url}"
        );
    }

    #[test]
    fn classifies_not_interested() {
        let url = ROOT_URL;
        assert_eq!(
            classify_link(url, PAGES_INDEX, 0),
            LinkKind::NotInterested,
            "for {url}"
        );
        assert_eq!(
            classify_link(url, SITEMAP_0, 0),
            LinkKind::NotInterested,
            "for {url}"
        );
    }

    #[test]
    fn unmatched_source_falls_back_to_path() {
        assert_eq!(
            classify_link(PRODUCT_URL, UNMATCHED_SOURCE, 0),
            LinkKind::Product,
            "for {PRODUCT_URL}"
        );
        assert_eq!(
            classify_link(CATALOG_URL, UNMATCHED_SOURCE, 0),
            LinkKind::Catalog,
            "for {CATALOG_URL}"
        );
        assert_eq!(
            classify_link(ROOT_URL, UNMATCHED_SOURCE, 0),
            LinkKind::Unknown,
            "for {ROOT_URL}"
        );
    }

    #[test]
    fn path_matching_is_case_insensitive() {
        let url = PRODUCT_URL.to_uppercase();
        assert_eq!(
            classify_link(&url, UNMATCHED_SOURCE, 0),
            LinkKind::Product,
            "for {url}"
        );
    }
}
