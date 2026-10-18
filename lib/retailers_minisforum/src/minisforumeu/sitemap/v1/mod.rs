//! Version 1 of the sitemap.

use bodul_shared::SitemapConfig;
use bodul_shared::link::LinkKind;
use retailer_sauce::shopify::shopify_from_location;
use std::time::Duration;

pub fn sitemap_config() -> SitemapConfig {
    SitemapConfig {
        sitemap_url: vec!["https://minisforumpc.eu/sitemap.xml".to_string()],
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

/// MinisForum runs Shopify; classification uses the shared Shopify rule.
pub fn from_location(url: &str) -> LinkKind {
    shopify_from_location(url)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifies_products() {
        let urls = [
            "https://minisforumpc.eu/de/products/accessories",
            "https://minisforumpc.eu/de/products/ai-x1-pro-mini-pc",
        ];
        for url in urls {
            assert_eq!(from_location(url), LinkKind::Product, "for {url}");
        }
    }

    #[test]
    fn classifies_catalog() {
        let urls = [
            "https://minisforumpc.eu/collections/all-products",
            "https://minisforumpc.eu/collections/alle-produkte-1",
        ];
        for url in urls {
            assert_eq!(from_location(url), LinkKind::Catalog, "for {url}");
        }
    }

    #[test]
    fn classifies_content() {
        let urls = [
            "https://minisforumpc.eu/blogs/blogartikel",
            "https://minisforumpc.eu/blogs/blogartikels/a-guide-to-choosing-your-perfect-minisforum-mini-pc",
            "https://minisforumpc.eu/de/pages/anniversary-sale",
        ];
        for url in urls {
            assert_eq!(from_location(url), LinkKind::Content, "for {url}");
        }
    }

    #[test]
    fn unknown_edge_cases() {
        let urls = [
            "https://minisforumpc.eu/",
            "https://minisforumpc.eu/agents.md",
            "https://minisforumpc.eu/de",
        ];
        for url in urls {
            assert_eq!(from_location(url), LinkKind::Unknown, "for {url}");
        }
    }

    #[test]
    fn locale_prefixed_product_still_product() {
        let urls = [
            "https://minisforumpc.eu/de/products/aiberzy-xg1-370-refurbished-mini-pc",
            "https://minisforumpc.eu/de/products/bluetooth-lautsprecher-mond-led-licht",
        ];
        for url in urls {
            assert_eq!(from_location(url), LinkKind::Product, "for {url}");
        }
    }

    #[test]
    fn url_encoded_path_classifies() {
        let urls = [
            "https://minisforumpc.eu/blogs/blogartikels/ai-power-unleashed-intel%C2%AE-core%E2%84%A2-ultra-9-m1-pro-285h",
            "https://minisforumpc.eu/blogs/elitemini-hm90-mit-amd-ryzen%E2%84%A2-9-4900h-kommt-in-kurze-in-deutschland-auf-den-markt",
        ];
        for url in urls {
            assert_eq!(from_location(url), LinkKind::Content, "for {url}");
        }
    }

    #[test]
    fn case_insensitive() {
        let url = "https://minisforumpc.eu/de/products/accessories".to_uppercase();
        assert_eq!(from_location(&url), LinkKind::Product, "for {url}");
    }
}
