//! Version 1 of the sitemap.

use bodul_shared::SitemapConfig;
use bodul_shared::link::LinkKind;
use retailer_sauce::shopify::shopify_from_location;

pub fn sitemap_config() -> SitemapConfig {
    SitemapConfig {
        sitemap_url: vec!["https://nas-de.ugreen.com/sitemap.xml".to_string()],
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
                "https://nas-de.ugreen.com/products/sd40180",
                LinkKind::Product,
            ),
            (
                "https://nas-de.ugreen.com/collections/all-product",
                LinkKind::Catalog,
            ),
            (
                "https://nas-de.ugreen.com/blogs/buying-guide",
                LinkKind::Content,
            ),
            ("https://nas-de.ugreen.com/agents.md", LinkKind::Unknown),
        ];

        for (url, expected) in cases {
            assert_eq!(from_location(url), expected, "for {url}");
        }
    }

    #[test]
    fn classifies_localized_nested_and_root_fixture_urls() {
        let cases = [
            (
                "https://nas-de.ugreen.com/blogs/buying-guide/wie-du-ein-richtiges-nas-laufwerk-auswahlst",
                LinkKind::Content,
            ),
            ("https://nas-de.ugreen.com/", LinkKind::Unknown),
        ];

        for (url, expected) in cases {
            assert_eq!(from_location(url), expected, "for {url}");
        }
    }
}
