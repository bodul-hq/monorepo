//! Version 1 of the sitemap.

use crate::anker_from_location;
use bodul_shared::SitemapConfig;
use bodul_shared::link::LinkKind;

pub fn sitemap_config() -> SitemapConfig {
    SitemapConfig {
        sitemap_url: vec!["https://www.anker.com/sitemap.xml".to_string()],
    }
}

pub fn classify_link(url: &str, source: &str, _image_count: usize) -> LinkKind {
    from_location(url, source)
}

pub fn from_location(url: &str, source: &str) -> LinkKind {
    let matced_by_source = match source {
        "https://www.anker.com/sitemap-0.xml" => Some(LinkKind::NotInterested),
        "https://www.anker.com/server-sitemap-index-pages.xml" => Some(LinkKind::NotInterested),
        "https://www.anker.com/server-sitemap-index-products.xml" => Some(LinkKind::Product),
        "https://www.anker.com/server-sitemap-index-collections.xml" => Some(LinkKind::Catalog),
        "https://www.anker.com/server-sitemap-index-blog.xml" => Some(LinkKind::Content),
        _ => None,
    };

    let path = url.to_lowercase();

    if let Some(y) = matced_by_source {
        if !path.starts_with("https://www.anker.com/fr/") {
            return LinkKind::NotInterested;
        }

        return y;
    }

    anker_from_location(&path)
}

#[cfg(test)]
mod tests {
    use super::*;

    const BLOG_SOURCE: &str = "https://www.anker.com/server-sitemap-index-blog.xml";
    const COLLECTION_SOURCE: &str = "https://www.anker.com/server-sitemap-index-collections.xml";
    const PRODUCT_SOURCE: &str = "https://www.anker.com/server-sitemap-index-products.xml";
    const ROOT_SOURCE: &str = "https://www.anker.com/sitemap-0.xml";

    #[test]
    fn classifies_grouped_sitemap_examples() {
        let cases = [
            (
                "https://www.anker.com/fr/products/a110a-anker-prime-26k-300w-power-bank",
                PRODUCT_SOURCE,
                LinkKind::Product,
            ),
            (
                "https://www.anker.com/fr/collections/100w-gan-charger",
                COLLECTION_SOURCE,
                LinkKind::Catalog,
            ),
            (
                "https://www.anker.com/fr/blogs/anker-guides/30w-usb-c-charger",
                BLOG_SOURCE,
                LinkKind::Content,
            ),
            (
                "https://www.anker.com",
                ROOT_SOURCE,
                LinkKind::NotInterested,
            ),
            ("https://www.anker.com/agents.md", "", LinkKind::Unknown),
        ];

        for (url, source, expected) in cases {
            assert_eq!(classify_link(url, source, 0), expected, "url: {url}");
        }
    }

    #[test]
    fn handles_grouped_sitemap_edge_cases() {
        let cases = [
            (
                "https://www.anker.com/fr/collections/2026年pd抽奖20-折扣",
                COLLECTION_SOURCE,
                LinkKind::Catalog,
            ),
            (
                "https://www.anker.com/fr/blogs/anker-guides",
                BLOG_SOURCE,
                LinkKind::Content,
            ),
            (
                "https://www.anker.com/fr/anker-solix/kit-solaire-hes",
                ROOT_SOURCE,
                LinkKind::NotInterested,
            ),
            (
                "https://www.anker.com/eu-en/products/a110a-anker-prime-26k-300w-power-bank",
                PRODUCT_SOURCE,
                LinkKind::NotInterested,
            ),
        ];

        for (url, source, expected) in cases {
            assert_eq!(classify_link(url, source, 0), expected, "url: {url}");
        }
    }
}
