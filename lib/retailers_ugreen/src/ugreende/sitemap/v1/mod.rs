//! Version 1 of the sitemap.

use bodul_shared::SitemapConfig;
use bodul_shared::link::LinkKind;
use retailer_sauce::shopify::shopify_from_location;

pub fn sitemap_config() -> SitemapConfig {
    SitemapConfig {
        sitemap_url: vec!["https://de.ugreen.com/sitemap.xml".to_string()],
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
                "https://de.ugreen.com/products/10-port-mac-mini-m4-dockingstation-8tb",
                LinkKind::Product,
            ),
            (
                "https://de.ugreen.com/collections/%E5%8B%BF%E5%88%A0-%E7%94%A8%E4%BA%8E%E9%9A%90%E8%97%8F%E6%90%9C%E7%B4%A2%E7%BB%93%E6%9E%9C",
                LinkKind::Catalog,
            ),
            ("https://de.ugreen.com/blogs/adapter", LinkKind::Content),
        ];

        for (url, expected) in cases {
            assert_eq!(from_location(url), expected, "for {url}");
        }
    }

    #[test]
    fn handles_fixture_edge_cases() {
        let cases = [
            ("https://de.ugreen.com/", LinkKind::Unknown),
            ("https://de.ugreen.com/agents.md", LinkKind::Unknown),
        ];

        for (url, expected) in cases {
            assert_eq!(from_location(url), expected, "for {url}");
        }
    }
}
