//! Version 1 of the sitemap.

use bodul_shared::SitemapConfig;
use bodul_shared::link::LinkKind;
use retailer_sauce::shopify::shopify_from_location;

pub fn sitemap_config() -> SitemapConfig {
    SitemapConfig {
        sitemap_url: vec!["https://ca.ugreen.com/sitemap.xml".to_string()],
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
                "https://ca.ugreen.com/products/100w-3c1a-gan-fast-charger",
                LinkKind::Product,
            ),
            (
                "https://ca.ugreen.com/collections/2023-labor-day",
                LinkKind::Catalog,
            ),
            (
                "https://ca.ugreen.com/blogs/bluetooth-adapter",
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
            (
                "https://ca.ugreen.com/fr/products/100w-3c1a-gan-fast-charger",
                LinkKind::Product,
            ),
            ("https://ca.ugreen.com/", LinkKind::Unknown),
            ("https://ca.ugreen.com/fr", LinkKind::Unknown),
        ];

        for (url, expected) in cases {
            assert_eq!(from_location(url), expected, "for {url}");
        }
    }
}
