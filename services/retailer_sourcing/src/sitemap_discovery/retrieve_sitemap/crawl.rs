//! Breadth-first crawl from the root sitemaps through every child index.

use std::collections::{HashSet, VecDeque};

use chrono::{DateTime, Utc};

use super::super::MAX_SITEMAP_DEPTH;
use super::super::document::RawSitemapDocument;
use super::super::parse::{ChildRef, Parsed, SitemapParseError, parse};
use super::handler::FetchSitemapBody;
use crate::platform::http::FetchError;

/// A problem with one sitemap file. The crawl skips that file and carries on.
#[derive(Debug, Clone, thiserror::Error)]
pub enum CrawlError {
    #[error("failed to fetch {url}: {source}")]
    Fetch {
        url: String,
        #[source]
        source: FetchError,
    },

    #[error("failed to parse {url}: {source}")]
    Parse {
        url: String,
        #[source]
        source: SitemapParseError,
    },

    #[error("sitemap index contains a cycle at {url}")]
    CyclicReference { url: String },

    #[error("already fetched url {url}")]
    AlreadyFetched { url: String },

    #[error("sitemap nesting exceeds the maximum depth of {max_depth} at {url}")]
    MaximumDepthExceeded { url: String, max_depth: usize },
}

/// Fetches every root sitemap and every child sitemap the fetched indexes
/// list. Returns the files fetched and the problems met along the way.
pub fn crawl(
    fetcher: &impl FetchSitemapBody,
    root_urls: &[String],
) -> (Vec<RawSitemapDocument>, Vec<CrawlError>) {
    let mut crawl = Crawl {
        fetcher,
        already_fetched: HashSet::new(),
        active: HashSet::new(),
        errors: Vec::new(),
    };
    let documents = crawl.run(root_urls);

    (documents, crawl.errors)
}

/// A sitemap waiting to be fetched.
struct SitemapItem {
    location: String,
    last_modified: Option<DateTime<Utc>>,
    depth: usize,
}

impl SitemapItem {
    fn root(url: &str) -> Self {
        Self {
            location: url.to_string(),
            last_modified: None,
            depth: 0,
        }
    }

    fn child(child: &ChildRef, depth: usize) -> Self {
        Self {
            location: child.location.to_string(),
            last_modified: child.last_modified,
            depth,
        }
    }
}

struct Crawl<'a, F> {
    fetcher: &'a F,
    already_fetched: HashSet<String>,
    active: HashSet<String>,
    errors: Vec<CrawlError>,
}

impl<F: FetchSitemapBody> Crawl<'_, F> {
    fn run(&mut self, root_urls: &[String]) -> Vec<RawSitemapDocument> {
        let mut documents = Vec::new();
        let mut queue: VecDeque<SitemapItem> =
            root_urls.iter().map(|url| SitemapItem::root(url)).collect();

        while let Some(item) = queue.pop_front() {
            let Some(document) = self.fetch_sitemap(&item) else {
                continue;
            };
            documents.push(document.clone());

            let parsed = match parse(&document.body, &item.location) {
                Ok(parsed) => parsed,
                Err(source) => {
                    self.errors.push(CrawlError::Parse {
                        url: item.location.clone(),
                        source,
                    });
                    continue;
                }
            };

            if let Parsed::Index(children) = parsed {
                for child in &children {
                    queue.push_back(SitemapItem::child(child, item.depth + 1));
                }
            }
        }

        documents
    }

    fn fetch_sitemap(&mut self, item: &SitemapItem) -> Option<RawSitemapDocument> {
        let url = &item.location;

        if item.depth > MAX_SITEMAP_DEPTH {
            self.errors.push(CrawlError::MaximumDepthExceeded {
                url: url.clone(),
                max_depth: MAX_SITEMAP_DEPTH,
            });
            return None;
        }

        if self.already_fetched.contains(url) {
            self.errors
                .push(CrawlError::AlreadyFetched { url: url.clone() });
            return None;
        }

        if !self.active.insert(url.clone()) {
            self.errors
                .push(CrawlError::CyclicReference { url: url.clone() });
            return None;
        }

        let body = match self.fetcher.fetch(url) {
            Ok(body) => body,
            Err(source) => {
                self.errors.push(CrawlError::Fetch {
                    url: url.clone(),
                    source,
                });
                return None;
            }
        };

        self.active.remove(url);
        self.already_fetched.insert(url.clone());

        Some(RawSitemapDocument {
            url: url.clone(),
            last_modified: item.last_modified,
            body_size: body.len(),
            body,
        })
    }
}
