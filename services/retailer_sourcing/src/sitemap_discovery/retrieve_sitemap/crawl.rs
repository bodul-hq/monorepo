//! Breadth-first crawl from the root sitemaps through every child index.

use std::collections::{HashSet, VecDeque};
use std::thread;
use std::time::Duration;

use chrono::{DateTime, Utc};

use super::handler::FetchSitemapBody;
use crate::platform::http::FetchError;
use sitemap::{ChildRef, MAX_SITEMAP_DEPTH, Parsed, RawSitemapDocument, SitemapParseError, parse};

/// How many downloads the crawl makes between pauses, so storefronts don't
/// rate-limit it.
const DOWNLOADS_PER_PAUSE: usize = 3;

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
/// list that `greylisted` accepts. Returns the files fetched, the child URLs
/// `greylisted` rejected and the problems met along the way. Sleeps for
/// `pause` after every [`DOWNLOADS_PER_PAUSE`] downloads.
pub fn crawl(
    fetcher: &impl FetchSitemapBody,
    root_urls: &[String],
    greylisted: impl Fn(&str) -> bool,
    pause: Duration,
) -> (Vec<RawSitemapDocument>, Vec<String>, Vec<CrawlError>) {
    let mut crawl = Crawl {
        fetcher,
        greylisted,
        pause,
        downloads: 0,
        already_fetched: HashSet::new(),
        active: HashSet::new(),
        not_greylisted: Vec::new(),
        errors: Vec::new(),
    };
    let documents = crawl.run(root_urls);

    (documents, crawl.not_greylisted, crawl.errors)
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

struct Crawl<'a, F, I> {
    fetcher: &'a F,
    greylisted: I,
    pause: Duration,
    /// Downloads attempted so far, failed ones included.
    downloads: usize,
    already_fetched: HashSet<String>,
    active: HashSet<String>,
    not_greylisted: Vec<String>,
    errors: Vec<CrawlError>,
}

impl<F: FetchSitemapBody, I: Fn(&str) -> bool> Crawl<'_, F, I> {
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
                    if !(self.greylisted)(&child.location) {
                        self.not_greylisted.push(child.location.to_string());
                        continue;
                    }
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

        if pauses_before(self.downloads) {
            thread::sleep(self.pause);
        }
        self.downloads += 1;

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

/// Whether the crawl pauses before its download number `downloads`, counting
/// from 0: after each group of [`DOWNLOADS_PER_PAUSE`], never before the first.
fn pauses_before(downloads: usize) -> bool {
    downloads > 0 && downloads.is_multiple_of(DOWNLOADS_PER_PAUSE)
}

#[cfg(test)]
mod tests {
    use super::pauses_before;

    #[test]
    fn pauses_after_every_third_download() {
        let pauses: Vec<usize> = (0..10).filter(|&n| pauses_before(n)).collect();

        assert_eq!(pauses, [3, 6, 9]);
    }
}
