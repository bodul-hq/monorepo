//! Version 1 of the sitemap.

use bodul_shared::SitemapConfig;
use bodul_shared::link::LinkKind;
use retailer_sauce::shopify::shopify_from_location;

pub fn sitemap_config() -> SitemapConfig {
    SitemapConfig {
        sitemap_url: vec!["https://nas.ugreen.jp/sitemap.xml".to_string()],
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
                "https://nas.ugreen.jp/products/ugreen-nasync-dxp2800",
                LinkKind::Product,
            ),
            (
                "https://nas.ugreen.jp/collections/nas-storage",
                LinkKind::Catalog,
            ),
            ("https://nas.ugreen.jp/blogs/inform", LinkKind::Content),
            ("https://nas.ugreen.jp/", LinkKind::Unknown),
        ];

        for (url, expected) in cases {
            assert_eq!(from_location(url), expected, "for {url}");
        }
    }

    #[test]
    fn handles_grouped_sitemap_edge_cases() {
        let cases = [
            (
                "https://nas.ugreen.jp/products/ugreen-lan%E3%82%B1%E3%83%BC%E3%83%96%E3%83%AB-2m",
                LinkKind::Product,
            ),
            (
                "https://nas.ugreen.jp/blogs/%E3%81%8A%E7%9F%A5%E3%82%89%E3%81%9B",
                LinkKind::Content,
            ),
            (
                "https://nas.ugreen.jp/blogs/inform/april-lucky-draw-event",
                LinkKind::Content,
            ),
            ("https://nas.ugreen.jp/agents.md", LinkKind::Unknown),
        ];

        for (url, expected) in cases {
            assert_eq!(from_location(url), expected, "for {url}");
        }
    }
}
