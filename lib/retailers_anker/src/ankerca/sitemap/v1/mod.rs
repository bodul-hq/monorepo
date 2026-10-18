//! Version 1 of the sitemap.

use crate::anker_from_location;
use bodul_shared::SitemapConfig;
use bodul_shared::link::LinkKind;
use std::time::Duration;

pub fn sitemap_config() -> SitemapConfig {
    SitemapConfig {
        sitemap_url: vec![
            "https://www.anker.com/ca/sitemap.xml".to_string(),
            "https://www.anker.com/ca-fr/sitemap.xml".to_string(),
        ],
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

/// Canada is served as two locales, each with its own sitemap set. `ca-fr` is not
/// a suffix of `ca` under the trailing slash, so the prefixes never collide.
const LOCALE_PREFIXES: [&str; 2] = ["https://www.anker.com/ca/", "https://www.anker.com/ca-fr/"];

pub fn from_location(url: &str, source: &str) -> LinkKind {
    let path = url.to_lowercase();

    for base in LOCALE_PREFIXES {
        let Some(file) = source.strip_prefix(base) else {
            continue;
        };

        if !path.starts_with(base) {
            return LinkKind::NotInterested;
        }

        return match file {
            "server-sitemap-index-pages.xml" => LinkKind::NotInterested,
            "server-sitemap-index-products.xml" => LinkKind::Product,
            "server-sitemap-index-collections.xml" => LinkKind::Catalog,
            "server-sitemap-index-blog.xml" => LinkKind::Content,
            _ => anker_from_location(&path),
        };
    }

    panic!(
        "ankerca: sitemap source {source:?} (for url {url:?}) does not start with any known locale prefix {LOCALE_PREFIXES:?}"
    );
}

// NOTE: URLs synthesized from anker_de shapes; the ankerca grouped-sitemap data is empty.
#[cfg(test)]
mod tests {
    use super::*;

    const PRODUCTS_INDEX: &str = "https://www.anker.com/ca/server-sitemap-index-products.xml";
    const COLLECTIONS_INDEX: &str = "https://www.anker.com/ca/server-sitemap-index-collections.xml";
    const BLOG_INDEX: &str = "https://www.anker.com/ca/server-sitemap-index-blog.xml";
    const PAGES_INDEX: &str = "https://www.anker.com/ca/server-sitemap-index-pages.xml";

    const FR_PRODUCTS_INDEX: &str = "https://www.anker.com/ca-fr/server-sitemap-index-products.xml";
    const FR_COLLECTIONS_INDEX: &str =
        "https://www.anker.com/ca-fr/server-sitemap-index-collections.xml";
    const FR_BLOG_INDEX: &str = "https://www.anker.com/ca-fr/server-sitemap-index-blog.xml";
    const FR_PAGES_INDEX: &str = "https://www.anker.com/ca-fr/server-sitemap-index-pages.xml";

    #[test]
    fn unmatched_source_still_rejects_out_of_region_urls() {
        for url in [
            "https://www.anker.com/agents.md",
            "https://www.anker.com/products/a110a",
        ] {
            assert_eq!(
                classify_link(url, "https://www.anker.com/ca/sitemap.xml", 0),
                LinkKind::NotInterested,
                "for {url}"
            );
        }
    }

    #[test]
    fn classifies_products() {
        let url = "https://www.anker.com/ca/products/a110a-anker-prime-26k-300w-power-bank";
        assert_eq!(
            classify_link(url, PRODUCTS_INDEX, 0),
            LinkKind::Product,
            "for {url}"
        );
    }

    #[test]
    fn classifies_catalog() {
        let url = "https://www.anker.com/ca/collections/1-2-phone-charges";
        assert_eq!(
            classify_link(url, COLLECTIONS_INDEX, 0),
            LinkKind::Catalog,
            "for {url}"
        );
    }

    #[test]
    fn classifies_content() {
        let url = "https://www.anker.com/ca/blogs/ac-power";
        assert_eq!(
            classify_link(url, BLOG_INDEX, 0),
            LinkKind::Content,
            "for {url}"
        );
    }

    #[test]
    fn classifies_not_interested() {
        let pages_url = "https://www.anker.com/ca/pages/about-us";
        assert_eq!(
            classify_link(pages_url, PAGES_INDEX, 0),
            LinkKind::NotInterested,
            "for {pages_url}"
        );
    }

    // Edge cases below are synthesized because the ankerca grouped-sitemap data folder is empty.

    #[test]
    fn region_guard_rejects_other_region() {
        // A source-matched product URL in a different region must not classify as Product.
        let url = "https://www.anker.com/eu-de/products/a110a";
        assert_eq!(
            classify_link(url, PRODUCTS_INDEX, 0),
            LinkKind::NotInterested,
            "for {url}"
        );
    }

    #[test]
    fn ca_fr_prefix_does_not_satisfy_ca_guard() {
        // `/ca-fr/` does not start with `/ca/`, so the region guard rejects it.
        let url = "https://www.anker.com/ca-fr/products/a110a";
        assert_eq!(
            classify_link(url, PRODUCTS_INDEX, 0),
            LinkKind::NotInterested,
            "for {url}"
        );
    }

    #[test]
    fn unmatched_source_falls_back_to_path() {
        // Unmatched source -> path-based fallback (anker_from_location).
        let unmatched_source = "https://www.anker.com/ca/sitemap.xml";

        let product_url = "https://www.anker.com/ca/products/a1109-f0";
        assert_eq!(
            classify_link(product_url, unmatched_source, 0),
            LinkKind::Product,
            "for {product_url}"
        );

        let unknown_url = "https://www.anker.com/ca/agents.md";
        assert_eq!(
            classify_link(unknown_url, unmatched_source, 0),
            LinkKind::Unknown,
            "for {unknown_url}"
        );
    }

    #[test]
    fn path_matching_is_case_insensitive() {
        let url = "HTTPS://WWW.ANKER.COM/CA/PRODUCTS/A110A";
        assert_eq!(
            classify_link(url, PRODUCTS_INDEX, 0),
            LinkKind::Product,
            "for {url}"
        );
    }

    // The `ca-fr` locale has its own sitemap set and is classified the same way.

    #[test]
    fn classifies_ca_fr_locale() {
        let cases = [
            (
                FR_PRODUCTS_INDEX,
                "https://www.anker.com/ca-fr/products/a110a",
                LinkKind::Product,
            ),
            (
                FR_COLLECTIONS_INDEX,
                "https://www.anker.com/ca-fr/collections/1-2-phone-charges",
                LinkKind::Catalog,
            ),
            (
                FR_BLOG_INDEX,
                "https://www.anker.com/ca-fr/blogs/ac-power",
                LinkKind::Content,
            ),
            (
                FR_PAGES_INDEX,
                "https://www.anker.com/ca-fr/pages/about-us",
                LinkKind::NotInterested,
            ),
        ];

        for (source, url, expected) in cases {
            assert_eq!(classify_link(url, source, 0), expected, "for {url}");
        }
    }

    #[test]
    fn ca_prefix_does_not_satisfy_ca_fr_guard() {
        // The mirror of `ca_fr_prefix_does_not_satisfy_ca_guard`: each locale's
        // source only accepts links under that same locale.
        let url = "https://www.anker.com/ca/products/a110a";
        assert_eq!(
            classify_link(url, FR_PRODUCTS_INDEX, 0),
            LinkKind::NotInterested,
            "for {url}"
        );
    }

    #[test]
    fn ca_fr_path_matching_is_case_insensitive() {
        let url = "HTTPS://WWW.ANKER.COM/CA-FR/PRODUCTS/A110A";
        assert_eq!(
            classify_link(url, FR_PRODUCTS_INDEX, 0),
            LinkKind::Product,
            "for {url}"
        );
    }
}
