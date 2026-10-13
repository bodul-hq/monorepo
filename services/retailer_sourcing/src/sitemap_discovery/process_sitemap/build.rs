//! Builds one sitemap tree from the stored sitemap files.

use std::collections::{HashMap, HashSet};

use chrono::{DateTime, Utc};

use super::super::MAX_SITEMAP_DEPTH;
use super::super::document::{RawSitemapDocument, SitemapDocument};
use super::super::parse::{ChildRef, Parsed, SitemapParseError, parse};

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
/// unnamed document holds them as children.
pub fn build_tree(
    root_urls: &[String],
    documents: &[RawSitemapDocument],
) -> Result<(SitemapDocument, Vec<BuildError>), BuildError> {
    if root_urls.is_empty() {
        return Err(BuildError::NoRootSitemaps);
    }

    let mut builder = TreeBuilder {
        documents: documents
            .iter()
            .map(|document| (document.url.as_str(), document))
            .collect(),
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

struct TreeBuilder<'a> {
    documents: HashMap<&'a str, &'a RawSitemapDocument>,
    active: HashSet<String>,
}

impl TreeBuilder<'_> {
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

        let raw = *self.documents.get(item.url.as_str()).ok_or_else(|| {
            BuildError::MissingRawDocument {
                url: item.url.clone(),
            }
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
