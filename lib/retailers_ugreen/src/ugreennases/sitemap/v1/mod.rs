//! Version 1 of the sitemap.

use bodul_shared::SitemapConfig;
use bodul_shared::link::LinkKind;
use retailer_sauce::shopify::shopify_from_location;

pub fn sitemap_config() -> SitemapConfig {
    SitemapConfig {
        sitemap_url: vec!["https://nas-es.ugreen.com/sitemap.xml".to_string()],
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
                "https://nas-es.ugreen.com/products/ugreen-nasync-dh2300-servidor-nas",
                LinkKind::Product,
            ),
            (
                "https://nas-es.ugreen.com/collections/compare",
                LinkKind::Catalog,
            ),
            (
                "https://nas-es.ugreen.com/blogs/buying-guide",
                LinkKind::Content,
            ),
            ("https://nas-es.ugreen.com/agents.md", LinkKind::Unknown),
        ];

        for (url, expected) in cases {
            assert_eq!(from_location(url), expected, "for {url}");
        }
    }

    #[test]
    fn classifies_localized_nested_and_root_fixture_urls() {
        let cases = [
            (
                "https://nas-es.ugreen.com/blogs/buying-guide/guia-elegir-mejor-nas-hogar",
                LinkKind::Content,
            ),
            ("https://nas-es.ugreen.com/", LinkKind::Unknown),
        ];

        for (url, expected) in cases {
            assert_eq!(from_location(url), expected, "for {url}");
        }
    }
}
