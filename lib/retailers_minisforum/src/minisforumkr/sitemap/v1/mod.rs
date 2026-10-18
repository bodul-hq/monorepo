//! Version 1 of the sitemap.

use bodul_shared::SitemapConfig;
use bodul_shared::link::LinkKind;
use retailer_sauce::shopify::shopify_from_location;
use std::time::Duration;

pub fn sitemap_config() -> SitemapConfig {
    SitemapConfig {
        sitemap_url: vec!["https://minisforum.kr/sitemap.xml".to_string()],
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
            "https://www.minisforum.kr/products/ms-s1-max",
            "https://www.minisforum.kr/products/minisforum-um890-pro",
        ];

        for url in urls {
            assert_eq!(from_location(url), LinkKind::Product, "for {url}");
        }
    }

    #[test]
    fn classifies_catalog() {
        let urls = [
            "https://www.minisforum.kr/collections/amd",
            "https://www.minisforum.kr/collections/minipc",
        ];

        for url in urls {
            assert_eq!(from_location(url), LinkKind::Catalog, "for {url}");
        }
    }

    #[test]
    fn classifies_content() {
        let urls = [
            "https://www.minisforum.kr/blogs/blog/ces-2026",
            "https://www.minisforum.kr/pages/about-minisforum",
        ];

        for url in urls {
            assert_eq!(from_location(url), LinkKind::Content, "for {url}");
        }
    }

    #[test]
    fn unknown_edge_cases() {
        let urls = [
            "https://www.minisforum.kr/",
            "https://www.minisforum.kr/agents.md",
        ];

        for url in urls {
            assert_eq!(from_location(url), LinkKind::Unknown, "for {url}");
        }
    }

    #[test]
    fn url_encoded_path_classifies() {
        // Real percent-encoded (Hangul) product and blog paths.
        assert_eq!(
            from_location(
                "https://www.minisforum.kr/products/minisforum-%EC%88%98%EB%82%A9-%EA%B0%80%EB%B0%A9"
            ),
            LinkKind::Product,
            "for encoded products path"
        );
        assert_eq!(
            from_location(
                "https://www.minisforum.kr/blogs/blog/%EB%AF%B8%EB%8B%88-%EC%9B%8C%ED%81%AC%EC%8A%A4%ED%85%8C%EC%9D%B4%EC%85%98%EC%9D%B4%EB%9E%80-%EC%84%A0%ED%83%9D-%EA%B0%80%EC%9D%B4%EB%93%9C"
            ),
            LinkKind::Content,
            "for encoded blogs path"
        );
    }

    #[test]
    fn case_insensitive() {
        assert_eq!(
            from_location("HTTPS://WWW.MINISFORUM.KR/PRODUCTS/MS-S1-MAX"),
            LinkKind::Product,
            "for uppercased product url"
        );
    }
}
