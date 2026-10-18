//! Version 1 of the sitemap.

use bodul_shared::SitemapConfig;
use bodul_shared::link::LinkKind;
use std::time::Duration;

pub fn sitemap_config() -> SitemapConfig {
    SitemapConfig {
        sitemap_url: vec!["https://www.ugreen.com/sitemap.xml".to_string()],
    }
}

/// Whether the crawl downloads the child sitemap at `url`. Root sitemaps are
/// always downloaded; drop one from [`sitemap_config`] instead.
pub fn sitemap_greylisted(url: &str) -> bool {
    // we are skipping any sitemap file having `metaobject` in filename
    !url.contains("metaobject")
}

/// How long the crawl pauses between groups of sitemap downloads. The store rate-limits
/// quick crawls, so it gets a longer pause.
pub fn sitemap_pause() -> Duration {
    Duration::from_secs(5)
}

pub fn classify_link(url: &str, _source: &str, _image_count: usize) -> LinkKind {
    from_location(url)
}

pub fn from_location(url: &str) -> LinkKind {
    let path = url.to_lowercase();

    if path.starts_with("https://www.ugreen.com/products/") {
        LinkKind::Product
    } else if path.contains("https://www.ugreen.com/collections/") {
        LinkKind::Catalog
    } else if path.contains("https://www.ugreen.com/pages/")
        || path.contains("https://www.ugreen.com/blogs/")
        || path.contains("https://www.ugreen.com/blog/")
    {
        LinkKind::Content
    } else {
        let v = vec![
            "/ar-ae/", "/ar-sa/", "/de-de/", "/en-ae/", "/en-au/", "/en-ca/", "/en-de/", "/en-es/",
            "/en-eu/", "/en-fr/", "/en-gb/", "/en-id/", "/en-it/", "/en-my/", "/en-nl/", "/en-ph/",
            "/en-sa/", "/en-sg/", "/en-th/", "/en-tr/", "/en-vn/", "/es-es/", "/fr-fr/", "/id-id/",
            "/it-it/", "/nl-nl/", "/th-th/", "/tr-tr/", "/vi-vn/", "/nl/", "/tr/", "/vi/",
        ];

        if v.iter().any(|p| path.contains(p)) {
            return LinkKind::NotInterested;
        }

        LinkKind::Unknown
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifies_fixture_urls() {
        let cases = [
            (
                "https://www.ugreen.com/products/ugreen-nasync-dxp2800-gt",
                LinkKind::Product,
            ),
            (
                "https://www.ugreen.com/collections/ae-audio",
                LinkKind::Catalog,
            ),
            ("https://www.ugreen.com/blogs/news", LinkKind::Content),
        ];

        for (url, expected) in cases {
            assert_eq!(from_location(url), expected, "for {url}");
        }
    }

    #[test]
    fn handles_localized_fixture_boundaries() {
        let cases = [
            (
                "https://www.ugreen.com/ar-ae/products/ae-10232",
                LinkKind::NotInterested,
            ),
            (
                "https://www.ugreen.com/ar-ae/collections/ae-audio",
                LinkKind::NotInterested,
            ),
            (
                "https://www.ugreen.com/ar-ae/pages/about-ugreen",
                LinkKind::NotInterested,
            ),
            ("https://www.ugreen.com/ar-ae", LinkKind::Unknown),
        ];

        for (url, expected) in cases {
            assert_eq!(from_location(url), expected, "for {url}");
        }
    }
}
