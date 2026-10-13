//! Version 1 of the sitemap.

use bodul_shared::SitemapConfig;
use bodul_shared::link::LinkKind;
use retailer_sauce::shopify::shopify_from_location;

pub fn sitemap_config() -> SitemapConfig {
    SitemapConfig {
        sitemap_url: vec!["https://nas-it.ugreen.com/sitemap.xml".to_string()],
    }
}

pub fn classify_link(url: &str, _source: &str, _image_count: usize) -> LinkKind {
    from_location(url)
}

pub fn from_location(url: &str) -> LinkKind {
    shopify_from_location(url)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifies_grouped_sitemap_examples() {
        let cases = [
            (
                "https://nas-it.ugreen.com/products/ugreen-nasync-dh2300-server-nas",
                LinkKind::Product,
            ),
            (
                "https://nas-it.ugreen.com/collections/nas-storage",
                LinkKind::Catalog,
            ),
            (
                "https://nas-it.ugreen.com/pages/about-us",
                LinkKind::Content,
            ),
            ("https://nas-it.ugreen.com/", LinkKind::Unknown),
        ];

        for (url, expected) in cases {
            assert_eq!(from_location(url), expected, "for {url}");
        }
    }

    #[test]
    fn handles_grouped_sitemap_edge_cases() {
        let cases = [
            (
                "https://nas-it.ugreen.com/products/ugreen-nas-backup-power-120w-12000mah",
                LinkKind::Product,
            ),
            (
                "https://nas-it.ugreen.com/products/worry-free-purchase",
                LinkKind::Product,
            ),
            (
                "https://nas-it.ugreen.com/blogs/how-to/install-jellyfin-setup-step-by-step",
                LinkKind::Content,
            ),
            ("https://nas-it.ugreen.com/agents.md", LinkKind::Unknown),
        ];

        for (url, expected) in cases {
            assert_eq!(from_location(url), expected, "for {url}");
        }
    }
}
