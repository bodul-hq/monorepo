//! Version 1 of the sitemap.

use bodul_shared::SitemapConfig;
use bodul_shared::link::LinkKind;
use retailer_sauce::shopify::shopify_from_location;

pub fn sitemap_config() -> SitemapConfig {
    SitemapConfig {
        sitemap_url: vec!["https://nas-au.ugreen.com/sitemap.xml".to_string()],
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
    fn classifies_fixture_urls() {
        let cases = [
            (
                "https://nas-au.ugreen.com/products/ugreen-nas-backup-power-120w-12000mah",
                LinkKind::Product,
            ),
            (
                "https://nas-au.ugreen.com/collections/compare",
                LinkKind::Catalog,
            ),
            ("https://nas-au.ugreen.com/blogs/news", LinkKind::Content),
            ("https://nas-au.ugreen.com/agents.md", LinkKind::Unknown),
        ];

        for (url, expected) in cases {
            assert_eq!(from_location(url), expected, "for {url}");
        }
    }

    #[test]
    fn classifies_numeric_and_root_fixture_urls() {
        let cases = [
            (
                "https://nas-au.ugreen.com/pages/2-bay-nas-for-home-ugreen-dxp2800",
                LinkKind::Content,
            ),
            ("https://nas-au.ugreen.com/", LinkKind::Unknown),
        ];

        for (url, expected) in cases {
            assert_eq!(from_location(url), expected, "for {url}");
        }
    }
}
