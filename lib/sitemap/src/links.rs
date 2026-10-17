//! Sorts a sitemap tree's URLs by kind of page.

use bodul_shared::link::LinkKind;
use serde::{Deserialize, Serialize};

use crate::{ROOT_SITEMAP_SOURCE, SitemapDocument, SitemapUrl};

/// A sitemap tree's URLs, sorted and de-duplicated within each kind.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct GroupedLinks {
    pub product: Vec<String>,
    pub catalog: Vec<String>,
    pub content: Vec<String>,
    pub not_interested: Vec<String>,
    pub unknown: Vec<String>,
}

impl GroupedLinks {
    pub fn from_document(
        document: &SitemapDocument,
        classify: impl Fn(&SitemapUrl) -> LinkKind,
    ) -> Self {
        let mut links = Self::default();
        for url in document.all_urls(ROOT_SITEMAP_SOURCE) {
            let group = match classify(&url) {
                LinkKind::Product => &mut links.product,
                LinkKind::Catalog => &mut links.catalog,
                LinkKind::Content => &mut links.content,
                LinkKind::NotInterested => &mut links.not_interested,
                LinkKind::Unknown => &mut links.unknown,
            };
            group.push(url.location);
        }
        for group in [
            &mut links.product,
            &mut links.catalog,
            &mut links.content,
            &mut links.not_interested,
            &mut links.unknown,
        ] {
            group.sort_unstable();
            group.dedup();
        }
        links
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn url(location: &str) -> SitemapUrl {
        SitemapUrl {
            location: location.to_string(),
            source: String::new(),
            last_modified: None,
            change_frequency: None,
            priority: None,
            images: Vec::new(),
        }
    }

    #[test]
    fn sorts_and_deduplicates_each_kind() {
        let document = SitemapDocument {
            urls: vec![
                url("https://example.com/products/b"),
                url("https://example.com/products/a"),
                url("https://example.com/products/b"),
                url("https://example.com/collections/x"),
            ],
            ..SitemapDocument::default()
        };

        let links = GroupedLinks::from_document(&document, |url| {
            if url.location.contains("/products/") {
                LinkKind::Product
            } else {
                LinkKind::Catalog
            }
        });

        assert_eq!(
            links.product,
            vec![
                "https://example.com/products/a",
                "https://example.com/products/b",
            ]
        );
        assert_eq!(links.catalog, vec!["https://example.com/collections/x"]);
        assert!(links.unknown.is_empty());
    }

    #[test]
    fn round_trips_through_json() {
        let links = GroupedLinks {
            product: vec!["https://example.com/products/a".to_string()],
            catalog: vec!["https://example.com/collections/x".to_string()],
            ..GroupedLinks::default()
        };

        let value = serde_json::to_value(&links).unwrap();

        assert_eq!(
            serde_json::from_value::<GroupedLinks>(value).unwrap(),
            links
        );
    }
}
