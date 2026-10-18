//! Version 1 of the sitemap.

use crate::anker_from_location;
use bodul_shared::SitemapConfig;
use bodul_shared::link::LinkKind;
use std::time::Duration;

// NOTE: URLs synthesized from anker_de shapes; the ankerau grouped-sitemap data is empty.

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

    if !path.starts_with("https://www.anker.com/au/") {
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

    // Source sitemap URLs used for classification.
    const PRODUCTS_INDEX: &str = "https://www.anker.com/server-sitemap-index-products.xml";
    const COLLECTIONS_INDEX: &str = "https://www.anker.com/server-sitemap-index-collections.xml";
    const BLOG_INDEX: &str = "https://www.anker.com/server-sitemap-index-blog.xml";
    const PAGES_INDEX: &str = "https://www.anker.com/server-sitemap-index-pages.xml";
    const SITEMAP_0: &str = "https://www.anker.com/sitemap-0.xml";

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
        let url = "https://www.anker.com/au/products/a110a-anker-prime-26k-300w-power-bank";
        assert_eq!(
            classify_link(url, PRODUCTS_INDEX, 0),
            LinkKind::Product,
            "for {url}"
        );
    }

    #[test]
    fn classifies_catalogs() {
        let url = "https://www.anker.com/au/collections/1-2-phone-charges";
        assert_eq!(
            classify_link(url, COLLECTIONS_INDEX, 0),
            LinkKind::Catalog,
            "for {url}"
        );
    }

    #[test]
    fn classifies_content() {
        let url = "https://www.anker.com/au/blogs/ac-power";
        assert_eq!(
            classify_link(url, BLOG_INDEX, 0),
            LinkKind::Content,
            "for {url}"
        );
    }

    #[test]
    fn classifies_not_interested() {
        let pages_url = "https://www.anker.com/au/pages/about-us";
        assert_eq!(
            classify_link(pages_url, PAGES_INDEX, 0),
            LinkKind::NotInterested,
            "for {pages_url}"
        );

        let sitemap_url = "https://www.anker.com/au/products/a110a-anker-prime-26k-300w-power-bank";
        assert_eq!(
            classify_link(sitemap_url, SITEMAP_0, 0),
            LinkKind::NotInterested,
            "for {sitemap_url}"
        );
    }

    // Edge cases below are synthesized (the ankerau grouped-sitemap data folder is empty),
    // so URLs are borrowed from anker_de shapes with the region prefix swapped to /au/.

    #[test]
    fn region_guard_rejects_other_region() {
        // A matched source but a url outside /au/ is guarded down to NotInterested.
        let other_region = "https://www.anker.com/eu-de/products/a110a";
        assert_eq!(
            classify_link(other_region, PRODUCTS_INDEX, 0),
            LinkKind::NotInterested,
            "for {other_region}"
        );

        let no_region = "https://www.anker.com/2023-anker-prime";
        assert_eq!(
            classify_link(no_region, PRODUCTS_INDEX, 0),
            LinkKind::NotInterested,
            "for {no_region}"
        );
    }

    #[test]
    fn unmatched_source_falls_back_to_path() {
        // An unmatched source classifies in-region URLs by path.
        let unmatched_source = "https://www.anker.com/sitemap.xml";

        let product_url = "https://www.anker.com/au/products/a1109-f0";
        assert_eq!(
            classify_link(product_url, unmatched_source, 0),
            LinkKind::Product,
            "for {product_url}"
        );

        let unknown_url = "https://www.anker.com/au/agents.md";
        assert_eq!(
            classify_link(unknown_url, unmatched_source, 0),
            LinkKind::Unknown,
            "for {unknown_url}"
        );
    }

    #[test]
    fn path_matching_is_case_insensitive() {
        let url = "HTTPS://WWW.ANKER.COM/AU/PRODUCTS/A110A";
        assert_eq!(
            classify_link(url, PRODUCTS_INDEX, 0),
            LinkKind::Product,
            "for {url}"
        );
    }
}
