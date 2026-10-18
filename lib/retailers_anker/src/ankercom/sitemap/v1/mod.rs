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

pub fn classify_link(url: &str, _source: &str, _image_count: usize) -> LinkKind {
    from_location(url)
}

pub fn from_location(url: &str) -> LinkKind {
    let path = url.to_lowercase();

    if path.starts_with("https://www.anker.com/products/") {
        LinkKind::Product
    } else if path.contains("https://www.anker.com/collections/") {
        LinkKind::Catalog
    } else if path.contains("https://www.anker.com/pages/")
        || path.contains("https://www.anker.com/blogs/")
        || path.contains("https://www.anker.com/blog/")
    {
        LinkKind::Content
    } else {
        let v = vec![
            "/ca-fr/", "/eu-de/", "/eu-en/", "/eu-pl/", "/ae/", "/au/", "/ca/", "/fr/", "/my/",
            "/nz/", "/uk/", "/vn/", "/de/", "/mx/",
        ];

        if v.iter().any(|p| path.contains(p)) {
            return LinkKind::NotInterested;
        }

        anker_from_location(url)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifies_products() {
        let urls = [
            "https://www.anker.com/products/a1109-f0",
            "https://www.anker.com/products/a110a-anker-prime-26k-300w-power-bank",
        ];

        for url in urls {
            assert_eq!(from_location(url), LinkKind::Product, "for {url}");
        }
    }

    #[test]
    fn classifies_catalogs() {
        let urls = [
            "https://www.anker.com/collections/1-2-phone-charges",
            "https://www.anker.com/collections/10000-mah-power-bank",
        ];

        for url in urls {
            assert_eq!(from_location(url), LinkKind::Catalog, "for {url}");
        }
    }

    #[test]
    fn classifies_content() {
        let urls = [
            "https://www.anker.com/blogs/ac-power",
            "https://www.anker.com/blogs/ac-power/a-guide-to-designing-an-ideal-family-charging-station",
        ];

        for url in urls {
            assert_eq!(from_location(url), LinkKind::Content, "for {url}");
        }
    }

    #[test]
    fn classifies_not_interested() {
        let urls = [
            "https://www.anker.com/ae/404",
            "https://www.anker.com/ae/500",
        ];

        for url in urls {
            assert_eq!(from_location(url), LinkKind::NotInterested, "for {url}");
        }
    }

    #[test]
    fn region_prefixed_product_is_not_interested() {
        // The products check uses starts_with the bare
        // "https://www.anker.com/products/" path, so region-prefixed product URLs
        // fall through to the region check and are classified NotInterested.
        let urls = [
            "https://www.anker.com/ae/products/a110d",
            "https://www.anker.com/ae/products/a1229",
        ];

        for url in urls {
            assert_eq!(from_location(url), LinkKind::NotInterested, "for {url}");
        }
    }

    #[test]
    fn root_and_bare_slug_are_unknown() {
        // Root and bare-slug URLs match no product/catalog/content/region rule,
        // so they fall through to anker_from_location which yields Unknown.
        let urls = [
            "https://www.anker.com",
            "https://www.anker.com/735charger65w",
            "https://www.anker.com/2023-anker-prime",
        ];

        for url in urls {
            assert_eq!(from_location(url), LinkKind::Unknown, "for {url}");
        }
    }

    #[test]
    fn path_matching_is_case_insensitive() {
        // The path is lowercased before matching, so an uppercase product URL
        // still classifies as Product.
        let url = "HTTPS://WWW.ANKER.COM/PRODUCTS/A1109-F0";
        assert_eq!(from_location(url), LinkKind::Product, "for {url}");
    }
}
