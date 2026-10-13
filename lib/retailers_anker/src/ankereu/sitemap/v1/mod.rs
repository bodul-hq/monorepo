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
        if !path.starts_with("https://www.anker.com/eu-en/") {
            return LinkKind::NotInterested;
        }

        return y;
    }

    anker_from_location(&path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifies_normal_sitemap_urls() {
        let cases = [
            (
                "https://www.anker.com/eu-en/products/a110a-anker-prime-26k-300w-power-bank",
                "https://www.anker.com/server-sitemap-index-products.xml",
                LinkKind::Product,
            ),
            (
                "https://www.anker.com/eu-en/collections/100w-usb-c-charger",
                "https://www.anker.com/server-sitemap-index-collections.xml",
                LinkKind::Catalog,
            ),
            (
                "https://www.anker.com/eu-en/blogs/power-banks/what-iphones-have-magsafe",
                "https://www.anker.com/server-sitemap-index-blog.xml",
                LinkKind::Content,
            ),
            (
                "https://www.anker.com",
                "https://www.anker.com/sitemap-0.xml",
                LinkKind::NotInterested,
            ),
            ("https://www.anker.com/agents.md", "", LinkKind::Unknown),
        ];

        for (url, source, expected) in cases {
            assert_eq!(classify_link(url, source, 0), expected, "for {url}");
        }
    }

    #[test]
    fn classifies_edge_case_sitemap_urls() {
        let cases = [
            (
                "https://www.anker.com/eu-en/collections/中小充-n-c800-f3800",
                "https://www.anker.com/server-sitemap-index-collections.xml",
                LinkKind::Catalog,
            ),
            (
                "https://www.anker.com/eu-en/products/soundcore-c30i-open-ear-clip-earbuds-mit-sicherem-halt",
                "https://www.anker.com/server-sitemap-index-products.xml",
                LinkKind::Product,
            ),
            (
                "https://www.anker.com/eu-en/blogs/power-banks/what-to-do-with-swollen-power-bank",
                "https://www.anker.com/server-sitemap-index-blog.xml",
                LinkKind::Content,
            ),
            (
                "https://www.anker.com/ae/collections/100w-power-bank",
                "https://www.anker.com/server-sitemap-index-collections.xml",
                LinkKind::NotInterested,
            ),
            (
                "https://www.anker.com/eu-en/products/a1215",
                "https://www.anker.com/server-sitemap-index-pages.xml",
                LinkKind::NotInterested,
            ),
        ];

        for (url, source, expected) in cases {
            assert_eq!(classify_link(url, source, 0), expected, "for {url}");
        }
    }
}
