//! Version 1 of the sitemap.

use bodul_shared::SitemapConfig;
use bodul_shared::link::LinkKind;
use retailer_sauce::shopify::shopify_from_location;

pub fn sitemap_config() -> SitemapConfig {
    SitemapConfig {
        sitemap_url: vec!["https://kr.ugreen.com/sitemap.xml".to_string()],
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
                "https://kr.ugreen.com/products/11-in-1-usb-c-hub-10gbps-dual-hdmi-4k60hz-100w",
                LinkKind::Product,
            ),
            (
                "https://kr.ugreen.com/collections/cables",
                LinkKind::Catalog,
            ),
            ("https://kr.ugreen.com/blogs/news", LinkKind::Content),
        ];

        for (url, expected) in cases {
            assert_eq!(from_location(url), expected, "for {url}");
        }
    }

    #[test]
    fn handles_fixture_edge_cases() {
        let cases = [
            ("https://kr.ugreen.com/", LinkKind::Unknown),
            ("https://kr.ugreen.com/agents.md", LinkKind::Unknown),
        ];

        for (url, expected) in cases {
            assert_eq!(from_location(url), expected, "for {url}");
        }
    }
}
