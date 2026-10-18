//! Version 1 of the sitemap.

use bodul_shared::SitemapConfig;
use bodul_shared::link::LinkKind;
use retailer_sauce::shopify::shopify_from_location;
use std::time::Duration;

const BASE: &str = "https://au.minisforum.com/";

pub fn sitemap_config() -> SitemapConfig {
    SitemapConfig {
        sitemap_url: vec!["https://au.minisforum.com/sitemap.xml".to_string()],
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

/// MinisForum runs Shopify; after the AU-specific exclusions (other hosts, the homepage,
/// `agents.md`, `/pages/` and service products) the shared Shopify rule applies.
pub fn from_location(url: &str) -> LinkKind {
    let path = url.to_lowercase();

    let Some(rest) = path.strip_prefix(BASE) else {
        return LinkKind::NotInterested;
    };

    if rest.is_empty() || rest == "agents.md" || rest.starts_with("pages/") {
        return LinkKind::NotInterested;
    }

    if let Some(slug) = rest.strip_prefix("products/")
        && (slug == "shipping-protection"
            || slug == "product-protection"
            || slug.starts_with("price-difference"))
    {
        return LinkKind::NotInterested;
    }

    shopify_from_location(&path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifies_products() {
        let urls = [
            "https://au.minisforum.com/products/adjustable-cellphone-stand",
            "https://au.minisforum.com/products/minisforum-ai-x1",
            "https://au.minisforum.com/products/minisforum-ms-01",
            "https://au.minisforum.com/products/minisforum-esp4b-pcie-to-4-nvme-m-2-expansion-card",
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
            "https://au.minisforum.com/collections/hot-sales",
        ];
        for url in urls {
            assert_eq!(from_location(url), LinkKind::Catalog, "for {url}");
        }
    }

    #[test]
    fn classifies_content() {
        let urls = [
            "https://au.minisforum.com/blogs/news",
            "https://au.minisforum.com/blogs/blog/m1-pro-285h-review-1",
            "https://au.minisforum.com/blogs/minisforum-ai-x1-pro-and-ai-x1-which-one-should-you-buy",
        ];
        for url in urls {
            assert_eq!(from_location(url), LinkKind::Content, "for {url}");
        }
    }

    #[test]
    fn classifies_not_interested() {
        let urls = [
            "https://au.minisforum.com/",
            "https://au.minisforum.com/agents.md",
            "https://au.minisforum.com/products/shipping-protection",
            "https://au.minisforum.com/products/product-protection",
            "https://au.minisforum.com/products/price-difference-2",
            "https://au.minisforum.com/pages/contact",
            "https://au.minisforum.com/pages/privacy-policy",
            "https://au.minisforum.com/pages/minisforum-back-to-school-sale-2026",
            "https://au.minisforum.com/pages/%F0%9F%93%A2-china-national-day-shipping-notice-2026",
            "https://store.minisforum.com/products/minisforum-ai-x1",
        ];
        for url in urls {
            assert_eq!(from_location(url), LinkKind::NotInterested, "for {url}");
        }
    }

    #[test]
    fn unknown_edge_cases() {
        let urls = ["https://au.minisforum.com/cart"];
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

    #[test]
    fn classify_link_ignores_source_and_images() {
        // The products sitemap also lists the homepage, so the source is not used.
        let source =
            "https://au.minisforum.com/sitemap_products_1.xml?from=8090954236019&to=10338714058867";
        assert_eq!(
            classify_link("https://au.minisforum.com/", source, 0),
            LinkKind::NotInterested
        );
        assert_eq!(
            classify_link(
                "https://au.minisforum.com/products/minisforum-ms-01",
                source,
                1
            ),
            LinkKind::Product
        );
    }
}
