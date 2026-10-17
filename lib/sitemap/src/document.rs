//! Sitemap documents: the raw XML as fetched, and the parsed tree.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// A sitemap file as fetched, before parsing.
#[derive(Debug, Clone, PartialEq)]
pub struct RawSitemapDocument {
    pub url: String,
    pub last_modified: Option<DateTime<Utc>>,
    pub body: String,
    pub body_size: usize,
}

/// A parsed sitemap as a composite tree.
///
/// A document holds its own page entries (`urls`) and any child documents
/// (`children`), so a sitemap index and its sub-sitemaps form one tree that
/// behaves like a single node: the accessors below recurse through `children`.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct SitemapDocument {
    /// This document's own URL, when known (the resolved root or an index entry).
    pub location: Option<String>,
    /// `<lastmod>` for this document, when listed in a parent index.
    pub last_modified: Option<DateTime<Utc>>,
    /// `<url>` entries declared directly in this document.
    pub urls: Vec<SitemapUrl>,
    /// Child documents (from a `<sitemapindex>`), already fetched and parsed.
    pub children: Vec<SitemapDocument>,
}

impl SitemapDocument {
    pub fn build(
        location: Option<String>,
        last_modified: Option<DateTime<Utc>>,
        urls: Vec<SitemapUrl>,
        children: Vec<SitemapDocument>,
    ) -> Self {
        SitemapDocument {
            location,
            last_modified,
            urls,
            children,
        }
    }

    /// Every `<url>` entry in this document and all of its descendants.
    pub fn all_urls(&self, source: &str) -> impl Iterator<Item = SitemapUrl> {
        let mut urls = Vec::new();
        self.collect_urls(&mut urls, source);
        urls.into_iter()
    }

    fn collect_urls(&self, urls: &mut Vec<SitemapUrl>, source: &str) {
        let source = self.location.as_deref().unwrap_or(source);
        let sourced_urls = self
            .urls
            .iter()
            .cloned()
            .map(|mut url| {
                url.source = source.to_string();
                url
            })
            .collect::<Vec<_>>();

        urls.extend(sourced_urls);

        for child in &self.children {
            child.collect_urls(urls, source);
        }
    }
}

/// One URL entry in a standard sitemap URL set.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SitemapUrl {
    pub location: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub source: String,
    pub last_modified: Option<DateTime<Utc>>,
    pub change_frequency: Option<ChangeFrequency>,
    /// Priority in `[0.0, 1.0]`.
    pub priority: Option<f32>,
    pub images: Vec<SitemapImage>,
}

/// Image metadata nested under a sitemap URL entry.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SitemapImage {
    pub location: String,
    pub title: Option<String>,
    pub caption: Option<String>,
}

impl SitemapImage {
    pub fn new(location: impl Into<String>) -> Self {
        let location = location.into();
        debug_assert!(
            !location.is_empty(),
            "sitemap image location must not be empty"
        );
        Self {
            location,
            title: None,
            caption: None,
        }
    }
}

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    Serialize,
    Deserialize,
    strum::EnumString,
    strum::IntoStaticStr,
)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum ChangeFrequency {
    Always,
    Hourly,
    Daily,
    Weekly,
    Monthly,
    Yearly,
    Never,
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::{ChangeFrequency, SitemapDocument, SitemapUrl};

    fn url(location: &str) -> SitemapUrl {
        SitemapUrl {
            location: location.to_string(),
            source: "stale".to_string(),
            last_modified: None,
            change_frequency: None,
            priority: None,
            images: Vec::new(),
        }
    }

    #[test]
    fn all_urls_sets_source_from_owning_document() {
        let products = SitemapDocument {
            location: Some("https://example.com/sitemap_products.xml".into()),
            urls: vec![
                url("https://example.com/products/a"),
                url("https://example.com/products/b"),
            ],
            ..SitemapDocument::default()
        };

        let root = SitemapDocument {
            location: Some("https://example.com/sitemap.xml".into()),
            urls: vec![url("https://example.com/")],
            children: vec![products],
            ..SitemapDocument::default()
        };

        let urls = root
            .all_urls("https://example.com/fallback-sitemap.xml")
            .collect::<Vec<_>>();

        assert_eq!(urls[0].source, "https://example.com/sitemap.xml");
        assert_eq!(urls[1].source, "https://example.com/sitemap_products.xml");
    }

    #[test]
    fn parses_and_formats_change_frequency() {
        let value = ChangeFrequency::from_str("daily").expect("daily is valid");
        let formatted: &'static str = value.into();

        assert_eq!(value, ChangeFrequency::Daily);
        assert_eq!(formatted, "daily");
    }

    #[test]
    fn rejects_unknown_change_frequency() {
        assert!(ChangeFrequency::from_str("sometimes").is_err());
    }

    #[test]
    fn parses_all_change_frequency_variants() {
        let cases = [
            ("always", ChangeFrequency::Always),
            ("hourly", ChangeFrequency::Hourly),
            ("daily", ChangeFrequency::Daily),
            ("weekly", ChangeFrequency::Weekly),
            ("monthly", ChangeFrequency::Monthly),
            ("yearly", ChangeFrequency::Yearly),
            ("never", ChangeFrequency::Never),
        ];

        for (input, expected) in cases {
            let parsed = ChangeFrequency::from_str(input).expect("must parse");
            let formatted: &'static str = parsed.into();
            assert_eq!(parsed, expected, "failed roundtrip for {input}");
            assert_eq!(formatted, input, "display mismatch for {input}");
        }
    }
}
