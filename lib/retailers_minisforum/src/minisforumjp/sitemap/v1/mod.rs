//! Version 1 of the sitemap.

use bodul_shared::SitemapConfig;
use bodul_shared::link::LinkKind;
use retailer_sauce::shopify::shopify_from_location;
use std::time::Duration;

pub fn sitemap_config() -> SitemapConfig {
    SitemapConfig {
        sitemap_url: vec!["https://www.minisforum.jp/sitemap.xml".to_string()],
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
            "https://www.minisforum.jp/products/ai-x1",
            "https://www.minisforum.jp/products/ms-a2",
        ];

        for url in urls {
            assert_eq!(from_location(url), LinkKind::Product, "for {url}");
        }
    }

    #[test]
    fn classifies_catalog() {
        let urls = [
            "https://www.minisforum.jp/collections/mini-pc",
            "https://www.minisforum.jp/collections/workstation",
        ];

        for url in urls {
            assert_eq!(from_location(url), LinkKind::Catalog, "for {url}");
        }
    }

    #[test]
    fn classifies_content() {
        let urls = [
            "https://www.minisforum.jp/blogs/blog/ces-2026",
            "https://www.minisforum.jp/pages/about-minisforum",
        ];

        for url in urls {
            assert_eq!(from_location(url), LinkKind::Content, "for {url}");
        }
    }

    #[test]
    fn unknown_edge_cases() {
        let urls = [
            "https://www.minisforum.jp/",
            "https://www.minisforum.jp/agents.md",
        ];

        for url in urls {
            assert_eq!(from_location(url), LinkKind::Unknown, "for {url}");
        }
    }

    #[test]
    fn url_encoded_path_classifies() {
        // Real percent-encoded (CJK) catalog and blog paths.
        assert_eq!(
            from_location(
                "https://www.minisforum.jp/collections/%E3%82%AD%E3%83%BC%E3%83%9C%E3%83%BC%E3%83%89"
            ),
            LinkKind::Catalog,
            "for encoded collections path"
        );
        assert_eq!(
            from_location(
                "https://www.minisforum.jp/blogs/%E3%83%91%E3%82%BD%E3%82%B3%E3%83%B3%E5%91%A8%E3%82%8A%E3%81%AE%E8%A8%98%E4%BA%8B"
            ),
            LinkKind::Content,
            "for encoded blogs path"
        );
    }

    #[test]
    fn case_insensitive() {
        assert_eq!(
            from_location("HTTPS://WWW.MINISFORUM.JP/PRODUCTS/AI-X1"),
            LinkKind::Product,
            "for uppercased product url"
        );
    }
}
