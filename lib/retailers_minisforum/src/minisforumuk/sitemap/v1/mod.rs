//! Version 1 of the sitemap.

use bodul_shared::SitemapConfig;
use bodul_shared::link::LinkKind;
use retailer_sauce::shopify::shopify_from_location;
use std::time::Duration;

pub fn sitemap_config() -> SitemapConfig {
    SitemapConfig {
        sitemap_url: vec!["https://www.minisforum.uk/sitemap.xml".to_string()],
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
            "https://www.minisforum.uk/products/ai-x1-pro",
            "https://www.minisforum.uk/products/minisforum-ms-01",
        ];

        for url in urls {
            assert_eq!(from_location(url), LinkKind::Product, "for {url}");
        }
    }

    #[test]
    fn classifies_catalog() {
        let urls = [
            "https://www.minisforum.uk/collections/intel",
            "https://www.minisforum.uk/collections/workstations",
        ];

        for url in urls {
            assert_eq!(from_location(url), LinkKind::Catalog, "for {url}");
        }
    }

    #[test]
    fn classifies_content() {
        let urls = [
            "https://www.minisforum.uk/blogs/blog/ces-2025",
            "https://www.minisforum.uk/pages/about-our-story",
        ];

        for url in urls {
            assert_eq!(from_location(url), LinkKind::Content, "for {url}");
        }
    }

    #[test]
    fn unknown_edge_cases() {
        let urls = [
            "https://www.minisforum.uk/",
            "https://www.minisforum.uk/agents.md",
        ];

        for url in urls {
            assert_eq!(from_location(url), LinkKind::Unknown, "for {url}");
        }
    }

    #[test]
    fn url_encoded_path_classifies() {
        // Real percent-encoded catalog path (encoded registered-trademark symbols).
        assert_eq!(
            from_location("https://www.minisforum.uk/collections/amd-%C2%AE-ryzen-%C2%AE"),
            LinkKind::Catalog,
            "for encoded collections path"
        );
    }

    #[test]
    fn case_insensitive() {
        assert_eq!(
            from_location("HTTPS://WWW.MINISFORUM.UK/PRODUCTS/AI-X1-PRO"),
            LinkKind::Product,
            "for uppercased product url"
        );
    }
}
