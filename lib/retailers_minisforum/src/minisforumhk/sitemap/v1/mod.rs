//! Version 1 of the sitemap.

use bodul_shared::SitemapConfig;
use bodul_shared::link::LinkKind;
use retailer_sauce::shopify::shopify_from_location;
use std::time::Duration;

pub fn sitemap_config() -> SitemapConfig {
    SitemapConfig {
        sitemap_url: vec!["https://minisforum.hk/sitemap.xml".to_string()],
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
            "https://minisforum.hk/products/ai-x1-260",
            "https://minisforum.hk/products/ms-a2",
        ];

        for url in urls {
            assert_eq!(from_location(url), LinkKind::Product, "for {url}");
        }
    }

    #[test]
    fn classifies_catalog() {
        let urls = [
            "https://minisforum.hk/collections/intel",
            "https://minisforum.hk/collections/atomman",
        ];

        for url in urls {
            assert_eq!(from_location(url), LinkKind::Catalog, "for {url}");
        }
    }

    #[test]
    fn classifies_content() {
        let urls = [
            "https://minisforum.hk/blogs/%E6%96%B0%E9%97%BB/coming-soon",
            "https://minisforum.hk/pages/about-us",
        ];

        for url in urls {
            assert_eq!(from_location(url), LinkKind::Content, "for {url}");
        }
    }

    #[test]
    fn unknown_edge_cases() {
        let urls = ["https://minisforum.hk/", "https://minisforum.hk/agents.md"];

        for url in urls {
            assert_eq!(from_location(url), LinkKind::Unknown, "for {url}");
        }
    }

    #[test]
    fn url_encoded_path_classifies() {
        // Real percent-encoded (CJK) catalog and blog paths.
        assert_eq!(
            from_location("https://minisforum.hk/collections/%E4%B8%BB%E6%A9%9F%E6%9D%BF"),
            LinkKind::Catalog,
            "for encoded collections path"
        );
        assert_eq!(
            from_location("https://minisforum.hk/blogs/%E6%96%B0%E9%97%BB"),
            LinkKind::Content,
            "for encoded blogs path"
        );
    }

    #[test]
    fn case_insensitive() {
        assert_eq!(
            from_location("HTTPS://MINISFORUM.HK/PRODUCTS/AI-X1-260"),
            LinkKind::Product,
            "for uppercased product url"
        );
    }
}
