//! Version 1 of the sitemap.

use bodul_shared::SitemapConfig;
use bodul_shared::link::LinkKind;
use retailer_sauce::shopify::shopify_from_location;
use std::time::Duration;

pub fn sitemap_config() -> SitemapConfig {
    SitemapConfig {
        sitemap_url: vec!["https://minisforum.ru/sitemap.xml".to_string()],
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
            "https://minisforum.ru/products/minisforum-ai-x1",
            "https://minisforum.ru/products/minisforum-ms-01",
        ];

        for url in urls {
            assert_eq!(from_location(url), LinkKind::Product, "for {url}");
        }
    }

    #[test]
    fn classifies_catalog() {
        let urls = [
            "https://minisforum.ru/collections/mini-pc",
            "https://minisforum.ru/collections/amd-ryzen",
        ];

        for url in urls {
            assert_eq!(from_location(url), LinkKind::Catalog, "for {url}");
        }
    }

    #[test]
    fn classifies_content() {
        let urls = [
            "https://minisforum.ru/blogs/%E6%96%B0%E9%97%BB/um700",
            "https://minisforum.ru/pages/o-nas",
        ];

        for url in urls {
            assert_eq!(from_location(url), LinkKind::Content, "for {url}");
        }
    }

    #[test]
    fn unknown_edge_cases() {
        let urls = ["https://minisforum.ru/", "https://minisforum.ru/agents.md"];

        for url in urls {
            assert_eq!(from_location(url), LinkKind::Unknown, "for {url}");
        }
    }

    #[test]
    fn url_encoded_path_classifies() {
        // Real percent-encoded (Cyrillic/CJK) catalog and blog paths.
        assert_eq!(
            from_location("https://minisforum.ru/collections/%D0%B0%D0%BA%D1%86%D0%B8%D0%B8"),
            LinkKind::Catalog,
            "for encoded collections path"
        );
        assert_eq!(
            from_location("https://minisforum.ru/blogs/%E6%96%B0%E9%97%BB"),
            LinkKind::Content,
            "for encoded blogs path"
        );
    }

    #[test]
    fn case_insensitive() {
        assert_eq!(
            from_location("HTTPS://MINISFORUM.RU/PRODUCTS/MINISFORUM-AI-X1"),
            LinkKind::Product,
            "for uppercased product url"
        );
    }
}
