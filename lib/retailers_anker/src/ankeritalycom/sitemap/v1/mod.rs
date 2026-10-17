//! Version 1 of the sitemap.

use crate::anker_from_location;
use bodul_shared::SitemapConfig;
use bodul_shared::link::LinkKind;
use std::time::Duration;

pub fn sitemap_config() -> SitemapConfig {
    SitemapConfig {
        sitemap_url: vec!["https://www.ankeritaly.com/sitemap.xml".to_string()],
    }
}

/// Whether the crawl downloads the child sitemap at `url`. Root sitemaps are
/// always downloaded; drop one from [`sitemap_config`] instead.
pub fn sitemap_greylisted(_url: &str) -> bool {
    true
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
        "https://www.ankeritaly.com/sitemap-0.xml" => Some(LinkKind::NotInterested),
        "https://www.ankeritaly.com/server-sitemap-index-pages.xml" => {
            Some(LinkKind::NotInterested)
        }
        "https://www.ankeritaly.com/server-sitemap-index-products.xml" => Some(LinkKind::Product),
        "https://www.ankeritaly.com/server-sitemap-index-collections.xml" => {
            Some(LinkKind::Catalog)
        }
        "https://www.ankeritaly.com/server-sitemap-index-blog.xml" => Some(LinkKind::Content),
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
    fn classifies_fixture_urls_by_sitemap_source() {
        let cases = [
            (
                "https://www.ankeritaly.com/products/a121d-anker-nano-caricatore-45w-3-pezzi",
                "https://www.ankeritaly.com/server-sitemap-index-products.xml",
                LinkKind::Product,
            ),
            (
                "https://www.ankeritaly.com/collections/3-4-phone-charges",
                "https://www.ankeritaly.com/server-sitemap-index-collections.xml",
                LinkKind::Catalog,
            ),
            (
                "https://www.ankeritaly.com/blogs/caricabatterie/my-phone-charger-is-plugged-in-but-not-charging",
                "https://www.ankeritaly.com/server-sitemap-index-blog.xml",
                LinkKind::Content,
            ),
            (
                "https://www.ankeritaly.com/world-first-45w-smart-display-iphone-charger",
                "https://www.ankeritaly.com/server-sitemap-index-pages.xml",
                LinkKind::NotInterested,
            ),
        ];

        for (url, source, expected) in cases {
            assert_eq!(classify_link(url, source, 0), expected, "for {url}");
        }
    }

    #[test]
    fn source_precedence_handles_overlapping_fixture_urls() {
        let cases = [
            (
                "https://www.ankeritaly.com/blogs",
                "https://www.ankeritaly.com/server-sitemap-index-blog.xml",
                LinkKind::Content,
            ),
            (
                "https://www.ankeritaly.com/blogs",
                "https://www.ankeritaly.com/sitemap-0.xml",
                LinkKind::NotInterested,
            ),
            (
                "https://www.ankeritaly.com/collections/all",
                "https://www.ankeritaly.com/server-sitemap-index-collections.xml",
                LinkKind::Catalog,
            ),
            (
                "https://www.ankeritaly.com/collections/all",
                "https://www.ankeritaly.com/server-sitemap-index-pages.xml",
                LinkKind::NotInterested,
            ),
        ];

        for (url, source, expected) in cases {
            assert_eq!(
                classify_link(url, source, 0),
                expected,
                "for {url} from {source}"
            );
        }
    }

    #[test]
    fn falls_back_to_url_classification_for_unknown_sources() {
        let cases = [
            (
                "https://www.ankeritaly.com/products/powerline-iii-usb-c-to-usb-c-cable-a9afa18c-734a-478b-bbb2-ccd18b2e5a59",
                LinkKind::Product,
            ),
            (
                "https://www.ankeritaly.com/collections/soundcore-care-liberty-5-pro-max",
                LinkKind::Catalog,
            ),
            (
                "https://www.ankeritaly.com/blogs/power-bank/charge-and-discharge-power-bank-at-the-same-time",
                LinkKind::Content,
            ),
            ("https://www.ankeritaly.com", LinkKind::Unknown),
        ];

        for (url, expected) in cases {
            assert_eq!(
                classify_link(url, "unknown-source", 0),
                expected,
                "for {url}"
            );
        }
    }
}
