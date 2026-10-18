//! Version 1 of the sitemap.

use crate::anker_from_location;
use bodul_shared::SitemapConfig;
use bodul_shared::link::LinkKind;
use std::time::Duration;

pub fn sitemap_config() -> SitemapConfig {
    SitemapConfig {
        sitemap_url: vec!["https://www.ankerjapan.com/sitemap.xml".to_string()],
    }
}

/// Whether the crawl downloads the child sitemap at `url`. Root sitemaps are
/// always downloaded; drop one from [`sitemap_config`] instead.
pub fn sitemap_greylisted(url: &str) -> bool {
    // we are skipping any sitemap file having `metaobject` in filename
    !url.contains("metaobject")
}

/// How long the crawl pauses between groups of sitemap downloads.
pub fn sitemap_pause() -> Duration {
    Duration::from_secs(1)
}

pub fn classify_link(url: &str, source: &str, _image_count: usize) -> LinkKind {
    from_location(url, source)
}

pub fn from_location(url: &str, source: &str) -> LinkKind {
    let matced_by_source = match source {
        "https://www.ankerjapan.com/sitemap-0.xml" => Some(LinkKind::NotInterested),
        "https://www.ankerjapan.com/server-sitemap-index-pages.xml" => {
            Some(LinkKind::NotInterested)
        }
        "https://www.ankerjapan.com/server-sitemap-index-products.xml" => Some(LinkKind::Product),
        "https://www.ankerjapan.com/server-sitemap-index-collections.xml" => {
            Some(LinkKind::Catalog)
        }
        "https://www.ankerjapan.com/server-sitemap-index-blog.xml" => Some(LinkKind::Content),
        _ => None,
    };

    if let Some(y) = matced_by_source {
        return y;
    }

    anker_from_location(url)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifies_normal_sitemap_urls() {
        let cases = [
            (
                "https://www.ankerjapan.com/products/68anhub",
                "https://www.ankerjapan.com/server-sitemap-index-products.xml",
                LinkKind::Product,
            ),
            (
                "https://www.ankerjapan.com/collections/wirelesscharger",
                "https://www.ankerjapan.com/server-sitemap-index-collections.xml",
                LinkKind::Catalog,
            ),
            (
                "https://www.ankerjapan.com/blogs/charging-banner",
                "https://www.ankerjapan.com/server-sitemap-index-blog.xml",
                LinkKind::Content,
            ),
            ("https://www.ankerjapan.com/", "", LinkKind::Unknown),
        ];

        for (url, source, expected) in cases {
            assert_eq!(classify_link(url, source, 0), expected, "for {url}");
        }
    }

    #[test]
    fn classifies_edge_case_sitemap_urls() {
        let cases = [
            (
                "https://www.ankerjapan.com/collections/%E3%82%AF%E3%83%BC%E3%83%9D%E3%83%B3%E9%81%A9%E7%94%A8%E8%A3%BD%E5%93%81",
                "",
                LinkKind::Catalog,
            ),
            (
                "https://www.ankerjapan.com/blogs/faq/%E3%82%A4%E3%83%A4%E3%83%9B%E3%83%B3%E3%82%92%E3%81%94%E4%BD%BF%E7%94%A8%E4%B8%AD-%E3%81%94%E6%A4%9C%E8%A8%8E%E4%B8%AD%E3%81%AE%E3%81%8A%E5%AE%A2%E6%A7%98%E3%81%B8",
                "",
                LinkKind::Content,
            ),
            (
                "https://www.ankerjapan.com/agents.md",
                "",
                LinkKind::Unknown,
            ),
            (
                "https://www.ankerjapan.com/products/68anhub",
                "https://www.ankerjapan.com/sitemap-0.xml",
                LinkKind::NotInterested,
            ),
        ];

        for (url, source, expected) in cases {
            assert_eq!(classify_link(url, source, 0), expected, "for {url}");
        }
    }
}
