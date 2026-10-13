//! Version 1 of the sitemap.

use bodul_shared::SitemapConfig;
use bodul_shared::link::LinkKind;
use retailer_sauce::shopify::shopify_from_location;

pub fn sitemap_config() -> SitemapConfig {
    SitemapConfig {
        sitemap_url: vec!["https://ugreen.jp/sitemap.xml".to_string()],
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
                "https://ugreen.jp/products/10000mah-2way-mobile-battery",
                LinkKind::Product,
            ),
            ("https://ugreen.jp/collections/all", LinkKind::Catalog),
            (
                "https://ugreen.jp/blogs/%E3%82%A4%E3%83%A4%E3%83%9B%E3%83%B3",
                LinkKind::Content,
            ),
        ];

        for (url, expected) in cases {
            assert_eq!(from_location(url), expected, "for {url}");
        }
    }

    #[test]
    fn handles_fixture_edge_cases() {
        let cases = [
            ("https://ugreen.jp/", LinkKind::Unknown),
            ("https://ugreen.jp/agents.md", LinkKind::Unknown),
        ];

        for (url, expected) in cases {
            assert_eq!(from_location(url), expected, "for {url}");
        }
    }
}
