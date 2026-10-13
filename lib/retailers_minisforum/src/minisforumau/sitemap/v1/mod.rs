//! Version 1 of the sitemap.

use bodul_shared::SitemapConfig;
use bodul_shared::link::LinkKind;
use retailer_sauce::shopify::shopify_from_location;

pub fn sitemap_config() -> SitemapConfig {
    SitemapConfig {
        sitemap_url: vec!["https://au.minisforum.com/sitemap.xml".to_string()],
    }
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
            "https://au.minisforum.com/products/adjustable-cellphone-stand",
            "https://au.minisforum.com/products/minisforum-ai-x1",
        ];
        for url in urls {
            assert_eq!(from_location(url), LinkKind::Product, "for {url}");
        }
    }

    #[test]
    fn classifies_catalog() {
        let urls = [
            "https://au.minisforum.com/collections/2026-valentines-day",
            "https://au.minisforum.com/collections/accessory",
        ];
        for url in urls {
            assert_eq!(from_location(url), LinkKind::Catalog, "for {url}");
        }
    }

    #[test]
    fn classifies_content() {
        let urls = [
            "https://au.minisforum.com/blogs/news",
            "https://au.minisforum.com/blogs/minisforum-ai-x1-pro-and-ai-x1-which-one-should-you-buy",
            "https://au.minisforum.com/pages/6th-anniversary-sale",
        ];
        for url in urls {
            assert_eq!(from_location(url), LinkKind::Content, "for {url}");
        }
    }

    #[test]
    fn unknown_edge_cases() {
        let urls = [
            "https://au.minisforum.com/",
            "https://au.minisforum.com/agents.md",
        ];
        for url in urls {
            assert_eq!(from_location(url), LinkKind::Unknown, "for {url}");
        }
    }

    #[test]
    fn url_encoded_path_classifies() {
        let urls = [
            "https://au.minisforum.com/blogs/news/intel%C2%AE-core%E2%84%A2-ultra-9-vs-core%E2%84%A2-i9-a-comprehensive-dialogue-between-two-generations-of-flagship-architectures",
            "https://au.minisforum.com/blogs/news/intel%C2%AE-core%E2%84%A2-ultra-9-vs-core%E2%84%A2-i9-a-comprehensive-dialogue-between-two-generations-of-flagship-architectures-1",
        ];
        for url in urls {
            assert_eq!(from_location(url), LinkKind::Content, "for {url}");
        }
    }

    #[test]
    fn case_insensitive() {
        let url = "https://au.minisforum.com/products/minisforum-ai-x1".to_uppercase();
        assert_eq!(from_location(&url), LinkKind::Product, "for {url}");
    }
}
