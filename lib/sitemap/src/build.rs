//! Builds one sitemap tree from the stored sitemap files.

use std::borrow::Cow;
use std::collections::{HashMap, HashSet};

use chrono::{DateTime, Utc};

use crate::MAX_SITEMAP_DEPTH;
use crate::document::{RawSitemapDocument, SitemapDocument};
use crate::parse::{ChildRef, Parsed, SitemapParseError, parse};

#[derive(Debug, Clone, thiserror::Error)]
pub enum BuildError {
    #[error("failed to parse {url}: {source}")]
    Parse {
        url: String,
        #[source]
        source: SitemapParseError,
    },

    #[error("raw sitemap document is missing for {url}")]
    MissingRawDocument { url: String },

    #[error("sitemap index contains a cycle at {url}")]
    CyclicReference { url: String },

    #[error("sitemap nesting exceeds the maximum depth of {max_depth} at {url}")]
    MaximumDepthExceeded { url: String, max_depth: usize },

    #[error("sitemap configuration contains no root URLs")]
    NoRootSitemaps,
}

/// Builds the tree under `root_urls` from `documents`.
///
/// A root that can't be built is left out and its error returned alongside the
/// tree. With one root, the tree is that root's document; with several, an
/// unnamed document holds them as children. A child sitemap that `greylisted`
/// rejects is left out without an error, since the crawl never downloaded it.
pub fn build_tree(
    root_urls: &[String],
    documents: &[RawSitemapDocument],
    greylisted: impl Fn(&str) -> bool,
) -> Result<(SitemapDocument, Vec<BuildError>), BuildError> {
    let documents: HashMap<&str, &RawSitemapDocument> = documents
        .iter()
        .map(|document| (document.url.as_str(), document))
        .collect();
    build_tree_with(root_urls, greylisted, |url| {
        documents.get(url).map(|document| Cow::Borrowed(*document))
    })
}

/// Builds the tree under `root_urls` like [`build_tree`], asking `document`
/// for each sitemap file by its URL, for callers that load files on demand.
pub fn build_tree_with<'a>(
    root_urls: &[String],
    greylisted: impl Fn(&str) -> bool,
    document: impl FnMut(&str) -> Option<Cow<'a, RawSitemapDocument>>,
) -> Result<(SitemapDocument, Vec<BuildError>), BuildError> {
    if root_urls.is_empty() {
        return Err(BuildError::NoRootSitemaps);
    }

    let mut builder = TreeBuilder {
        greylisted,
        document,
        active: HashSet::new(),
    };

    let mut roots = Vec::new();
    let mut errors = Vec::new();
    for url in root_urls {
        match builder.node(SitemapItem::root(url)) {
            Ok(document) => roots.push(document),
            Err(error) => errors.push(error),
        }
    }

    let tree = if roots.len() == 1 {
        roots.remove(0)
    } else {
        SitemapDocument {
            children: roots,
            ..SitemapDocument::default()
        }
    };

    Ok((tree, errors))
}

/// A sitemap file to place in the tree.
struct SitemapItem {
    url: String,
    last_modified: Option<DateTime<Utc>>,
    depth: usize,
}

impl SitemapItem {
    fn root(url: &str) -> Self {
        Self {
            url: url.to_string(),
            last_modified: None,
            depth: 0,
        }
    }

    fn child(child: &ChildRef, depth: usize) -> Self {
        Self {
            url: child.location.to_string(),
            last_modified: child.last_modified,
            depth,
        }
    }
}

struct TreeBuilder<I, F> {
    greylisted: I,
    document: F,
    active: HashSet<String>,
}

impl<'a, I, F> TreeBuilder<I, F>
where
    I: Fn(&str) -> bool,
    F: FnMut(&str) -> Option<Cow<'a, RawSitemapDocument>>,
{
    fn node(&mut self, item: SitemapItem) -> Result<SitemapDocument, BuildError> {
        if item.depth > MAX_SITEMAP_DEPTH {
            return Err(BuildError::MaximumDepthExceeded {
                url: item.url,
                max_depth: MAX_SITEMAP_DEPTH,
            });
        }

        if !self.active.insert(item.url.clone()) {
            return Err(BuildError::CyclicReference { url: item.url });
        }

        let raw = (self.document)(&item.url).ok_or_else(|| BuildError::MissingRawDocument {
            url: item.url.clone(),
        })?;

        let parsed = parse(&raw.body, &item.url).map_err(|source| BuildError::Parse {
            url: item.url.clone(),
            source,
        })?;

        let mut urls = Vec::new();
        let mut children = Vec::new();
        match parsed {
            Parsed::UrlSet(url_set) => urls = url_set,
            Parsed::Index(child_refs) => {
                for child in &child_refs {
                    if !(self.greylisted)(&child.location) {
                        continue;
                    }
                    children.push(self.node(SitemapItem::child(child, item.depth + 1))?);
                }
            }
        }

        self.active.remove(&item.url);

        let last_modified = item.last_modified.or(raw.last_modified);
        Ok(SitemapDocument::build(
            Some(item.url),
            last_modified,
            urls,
            children,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ROOT: &str = "https://shop.test/sitemap.xml";
    const PRODUCTS: &str = "https://shop.test/sitemap_products.xml";
    const BLOGS: &str = "https://shop.test/sitemap_blogs.xml";

    fn raw(url: &str, body: String) -> RawSitemapDocument {
        RawSitemapDocument {
            url: url.to_string(),
            last_modified: None,
            body_size: body.len(),
            body,
        }
    }

    fn index(children: &[&str]) -> String {
        let entries: String = children
            .iter()
            .map(|child| format!("<sitemap><loc>{child}</loc></sitemap>"))
            .collect();
        format!("<sitemapindex>{entries}</sitemapindex>")
    }

    fn url_set(page: &str) -> String {
        format!("<urlset><url><loc>{page}</loc></url></urlset>")
    }

    #[test]
    fn leaves_out_a_child_that_is_not_greylisted_without_an_error() {
        // The crawl skipped the blogs sitemap, so only the products one is stored.
        let documents = [
            raw(ROOT, index(&[PRODUCTS, BLOGS])),
            raw(PRODUCTS, url_set("https://shop.test/products/a")),
        ];

        let (tree, errors) =
            build_tree(&[ROOT.to_string()], &documents, |url| url != BLOGS).unwrap();

        assert!(errors.is_empty(), "{errors:?}");
        let children: Vec<_> = tree
            .children
            .iter()
            .map(|c| c.location.as_deref())
            .collect();
        assert_eq!(children, [Some(PRODUCTS)]);
    }

    #[test]
    fn a_missing_greylisted_child_fails_its_root() {
        let documents = [raw(ROOT, index(&[PRODUCTS, BLOGS]))];

        let (_, errors) = build_tree(&[ROOT.to_string()], &documents, |_| true).unwrap();

        assert!(matches!(
            errors.as_slice(),
            [BuildError::MissingRawDocument { url }] if url == PRODUCTS
        ));
    }
}
