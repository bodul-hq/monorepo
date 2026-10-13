//! Version 1 of the sitemap.

use bodul_shared::SitemapConfig;
use bodul_shared::link::LinkKind;
use retailer_sauce::shopify::shopify_from_location;

pub fn sitemap_config() -> SitemapConfig {
    SitemapConfig {
        sitemap_url: vec!["https://www.mi.com/sitemap.xml".to_string()],
    }
}

pub fn classify_link(url: &str, _source: &str, _image_count: usize) -> LinkKind {
    from_location(url)
}

pub fn from_location(url: &str) -> LinkKind {
    shopify_from_location(url)
}

// NOTE: URLs synthesized; the micom grouped-sitemap data folder is empty.
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifies_products() {
        let urls = [
            "https://www.mi.com/products/redmi-note-14",
            "https://www.mi.com/products/xiaomi-14-ultra",
        ];

        for url in urls {
            assert_eq!(from_location(url), LinkKind::Product, "for {url}");
        }
    }

    #[test]
    fn classifies_catalog() {
        let urls = [
            "https://www.mi.com/collections/smartphones",
            "https://www.mi.com/collections/smart-home",
        ];

        for url in urls {
            assert_eq!(from_location(url), LinkKind::Catalog, "for {url}");
        }
    }

    #[test]
    fn classifies_content() {
        let urls = [
            "https://www.mi.com/pages/about-us",
            "https://www.mi.com/blogs/news/launch-event",
        ];

        for url in urls {
            assert_eq!(from_location(url), LinkKind::Content, "for {url}");
        }
    }

    #[test]
    fn unknown_edge_cases() {
        let urls = ["https://www.mi.com/"];

        for url in urls {
            assert_eq!(from_location(url), LinkKind::Unknown, "for {url}");
        }
    }

    #[test]
    fn case_insensitive() {
        assert_eq!(
            from_location("HTTPS://WWW.MI.COM/PRODUCTS/REDMI-NOTE-14"),
            LinkKind::Product,
            "for uppercased product url"
        );
    }
}
