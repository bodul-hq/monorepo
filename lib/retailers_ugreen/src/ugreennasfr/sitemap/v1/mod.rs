//! Version 1 of the sitemap.

use bodul_shared::SitemapConfig;
use bodul_shared::link::LinkKind;
use retailer_sauce::shopify::shopify_from_location;

pub fn sitemap_config() -> SitemapConfig {
    SitemapConfig {
        sitemap_url: vec!["https://nas-fr.ugreen.com/sitemap.xml".to_string()],
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
                "https://nas-fr.ugreen.com/products/35340",
                LinkKind::Product,
            ),
            (
                "https://nas-fr.ugreen.com/collections/all-product",
                LinkKind::Catalog,
            ),
            (
                "https://nas-fr.ugreen.com/blogs/connaissances-nas",
                LinkKind::Content,
            ),
            ("https://nas-fr.ugreen.com/agents.md", LinkKind::Unknown),
        ];

        for (url, expected) in cases {
            assert_eq!(from_location(url), expected, "for {url}");
        }
    }

    #[test]
    fn classifies_localized_nested_and_root_fixture_urls() {
        let cases = [
            (
                "https://nas-fr.ugreen.com/blogs/connaissances-nas/qu-est-ce-qu-un-nas-utilisation",
                LinkKind::Content,
            ),
            ("https://nas-fr.ugreen.com/", LinkKind::Unknown),
        ];

        for (url, expected) in cases {
            assert_eq!(from_location(url), expected, "for {url}");
        }
    }
}
