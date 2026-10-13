//! Version 1 of the sitemap.

use bodul_shared::SitemapConfig;
use bodul_shared::link::LinkKind;
use retailer_sauce::shopify::shopify_from_location;

pub fn sitemap_config() -> SitemapConfig {
    SitemapConfig {
        sitemap_url: vec!["https://www.ugreenindia.com/sitemap.xml".to_string()],
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
                "https://www.ugreenindia.com/products/100w-plus-100w-cable",
                LinkKind::Product,
            ),
            (
                "https://www.ugreenindia.com/collections/apple",
                LinkKind::Catalog,
            ),
            (
                "https://www.ugreenindia.com/blogs/best-chargers-for-iphone-16-in-india",
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
                "https://www.ugreenindia.com/bn/products/100w-plus-100w-cable",
                LinkKind::Product,
            ),
            ("https://www.ugreenindia.com/", LinkKind::Unknown),
            ("https://www.ugreenindia.com/hi", LinkKind::Unknown),
        ];

        for (url, expected) in cases {
            assert_eq!(from_location(url), expected, "for {url}");
        }
    }
}
