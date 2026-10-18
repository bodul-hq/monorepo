//! Version 1 of the sitemap.

use bodul_shared::SitemapConfig;
use bodul_shared::link::LinkKind;
use retailer_sauce::shopify::shopify_from_location;
use std::time::Duration;

pub fn sitemap_config() -> SitemapConfig {
    SitemapConfig {
        sitemap_url: vec!["https://ca.minisforum.com/sitemap.xml".to_string()],
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
            "https://ca.minisforum.com/products/adjustable-cellphone-stand",
            "https://ca.minisforum.com/products/minisforum-ai-x1",
        ];
        for url in urls {
            assert_eq!(from_location(url), LinkKind::Product, "for {url}");
        }
    }

    #[test]
    fn classifies_catalog() {
        let urls = [
            "https://ca.minisforum.com/collections/accessory",
            "https://ca.minisforum.com/collections/ai-x1-pro-series",
        ];
        for url in urls {
            assert_eq!(from_location(url), LinkKind::Catalog, "for {url}");
        }
    }

    #[test]
    fn classifies_content() {
        let urls = [
            "https://ca.minisforum.com/blogs/news",
            "https://ca.minisforum.com/blogs/news/exploring-the-practicality-and-usage-conditions-of-egpu-docks",
            "https://ca.minisforum.com/pages/6th-anniversary-sale",
        ];
        for url in urls {
            assert_eq!(from_location(url), LinkKind::Content, "for {url}");
        }
    }

    #[test]
    fn unknown_edge_cases() {
        let urls = [
            "https://ca.minisforum.com/",
            "https://ca.minisforum.com/agents.md",
        ];
        for url in urls {
            assert_eq!(from_location(url), LinkKind::Unknown, "for {url}");
        }
    }

    #[test]
    fn url_encoded_path_classifies() {
        let urls = [
            "https://ca.minisforum.com/blogs/news/intel%C2%AE-core%E2%84%A2-ultra-9-vs-core%E2%84%A2-i9-a-comprehensive-dialogue-between-two-generations-of-flagship-architectures",
            "https://ca.minisforum.com/blogs/news/intel%C2%AE-core%E2%84%A2-ultra-9-vs-core%E2%84%A2-i9-a-comprehensive-dialogue-between-two-generations-of-flagship-architectures-1",
        ];
        for url in urls {
            assert_eq!(from_location(url), LinkKind::Content, "for {url}");
        }
    }

    #[test]
    fn case_insensitive() {
        let url = "https://ca.minisforum.com/products/minisforum-ai-x1".to_uppercase();
        assert_eq!(from_location(&url), LinkKind::Product, "for {url}");
    }
}
