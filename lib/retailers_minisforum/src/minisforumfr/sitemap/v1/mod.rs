//! Version 1 of the sitemap.

use bodul_shared::SitemapConfig;
use bodul_shared::link::LinkKind;
use retailer_sauce::shopify::shopify_from_location;

pub fn sitemap_config() -> SitemapConfig {
    SitemapConfig {
        sitemap_url: vec!["https://minisforumpc.fr/sitemap.xml".to_string()],
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
            "https://minisforumpc.fr/en/products/gift-card",
            "https://minisforumpc.fr/en/products/minisforum-ai-x1",
        ];
        for url in urls {
            assert_eq!(from_location(url), LinkKind::Product, "for {url}");
        }
    }

    #[test]
    fn classifies_catalog() {
        let urls = [
            "https://minisforumpc.fr/collections/100-499",
            "https://minisforumpc.fr/collections/100-eur",
        ];
        for url in urls {
            assert_eq!(from_location(url), LinkKind::Catalog, "for {url}");
        }
    }

    #[test]
    fn classifies_content() {
        let urls = [
            "https://minisforumpc.fr/blogs/blog",
            "https://minisforumpc.fr/blogs/blog/avis-de-qualite-du-minisforum-nab9",
            "https://minisforumpc.fr/en/pages/affiliates-program",
        ];
        for url in urls {
            assert_eq!(from_location(url), LinkKind::Content, "for {url}");
        }
    }

    #[test]
    fn unknown_edge_cases() {
        let urls = [
            "https://minisforumpc.fr/",
            "https://minisforumpc.fr/agents.md",
            "https://minisforumpc.fr/en",
        ];
        for url in urls {
            assert_eq!(from_location(url), LinkKind::Unknown, "for {url}");
        }
    }

    #[test]
    fn locale_prefixed_product_still_product() {
        let urls = [
            "https://minisforumpc.fr/en/products/minisforum-ai-x1-pro-370",
            "https://minisforumpc.fr/en/products/minisforum-ai-x1-pro-470-ai-mini-pc",
        ];
        for url in urls {
            assert_eq!(from_location(url), LinkKind::Product, "for {url}");
        }
    }

    #[test]
    fn case_insensitive() {
        let url = "https://minisforumpc.fr/en/products/gift-card".to_uppercase();
        assert_eq!(from_location(&url), LinkKind::Product, "for {url}");
    }
}
