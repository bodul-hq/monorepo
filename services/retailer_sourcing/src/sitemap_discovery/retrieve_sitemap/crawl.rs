//! Breadth-first crawl from the root sitemaps through every child index.

use std::collections::{HashSet, VecDeque};

use chrono::{DateTime, Utc};

use super::handler::FetchSitemapBody;
use crate::platform::http::FetchError;
use sitemap::{ChildRef, MAX_SITEMAP_DEPTH, Parsed, RawSitemapDocument, SitemapParseError, parse};

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
/// `greylisted` rejected and the problems met along the way. Calls `pause`
/// between actual fetch attempts, never for skipped URLs or before the first.
pub fn crawl(
    fetcher: &impl FetchSitemapBody,
    root_urls: &[String],
    greylisted: impl Fn(&str) -> bool,
    pause: impl FnMut(),
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

struct Crawl<'a, F, I, P> {
    fetcher: &'a F,
    greylisted: I,
    pause: P,
    /// Downloads attempted so far, failed ones included.
    downloads: usize,
    already_fetched: HashSet<String>,
    active: HashSet<String>,
    not_greylisted: Vec<String>,
    errors: Vec<CrawlError>,
}

impl<F: FetchSitemapBody, I: Fn(&str) -> bool, P: FnMut()> Crawl<'_, F, I, P> {
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

        if self.downloads > 0 {
            (self.pause)();
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

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::sync::Mutex;

    use super::*;

    struct Pages {
        bodies: HashMap<String, String>,
        order: Mutex<Vec<String>>,
    }
    impl FetchSitemapBody for Pages {
        fn fetch(&self, url: &str) -> Result<String, FetchError> {
            self.order.lock().unwrap().push(url.to_owned());
            self.bodies.get(url).cloned().ok_or_else(|| FetchError {
                message: "fixture fetch failure".into(),
            })
        }
    }
    fn index(urls: &[&str]) -> String {
        format!(
            "<sitemapindex xmlns=\"http://www.sitemaps.org/schemas/sitemap/0.9\">{}</sitemapindex>",
            urls.iter()
                .map(|url| format!("<sitemap><loc>{url}</loc></sitemap>"))
                .collect::<String>()
        )
    }
    const LEAF: &str = "<urlset xmlns=\"http://www.sitemaps.org/schemas/sitemap/0.9\"></urlset>";

    #[test]
    fn pauses_only_between_attempts_in_breadth_first_order_even_after_failures() {
        let pages = Pages {
            bodies: [
                (
                    "https://shop.test/root",
                    index(&[
                        "https://shop.test/child",
                        "https://shop.test/skip",
                        "https://shop.test/root",
                        "https://shop.test/missing",
                    ]),
                ),
                ("https://shop.test/bad", "malformed XML".into()),
                ("https://shop.test/child", LEAF.into()),
            ]
            .into_iter()
            .map(|(url, body)| (url.into(), body))
            .collect(),
            order: Mutex::default(),
        };
        let roots = [
            "https://shop.test/root",
            "https://shop.test/missing",
            "https://shop.test/bad",
        ]
        .map(str::to_owned);
        let (documents, rejected, errors) = crawl(
            &pages,
            &roots,
            |url| !url.ends_with("/skip"),
            || {
                pages.order.lock().unwrap().push("pause".into());
            },
        );
        assert_eq!(
            *pages.order.lock().unwrap(),
            [
                "https://shop.test/root",
                "pause",
                "https://shop.test/missing",
                "pause",
                "https://shop.test/bad",
                "pause",
                "https://shop.test/child",
            ]
        );
        assert_eq!(documents.len(), 3);
        assert_eq!(rejected, ["https://shop.test/skip"]);
        assert_eq!(errors.len(), 4); // Fetch, parse, duplicate root, repeated failed URL.
    }

    #[test]
    fn depth_limit_and_duplicate_roots_do_not_add_pauses() {
        let bodies = (0..=MAX_SITEMAP_DEPTH)
            .map(|depth| {
                (
                    format!("https://shop.test/{depth}"),
                    index(&[&format!("https://shop.test/{}", depth + 1)]),
                )
            })
            .collect();
        let pages = Pages {
            bodies,
            order: Mutex::default(),
        };
        let roots = ["https://shop.test/0".into(), "https://shop.test/0".into()];
        let mut pauses = 0;
        let (documents, _, errors) = crawl(&pages, &roots, |_| true, || pauses += 1);
        assert_eq!(documents.len(), MAX_SITEMAP_DEPTH + 1);
        assert_eq!(pauses, MAX_SITEMAP_DEPTH);
        assert!(
            errors
                .iter()
                .any(|error| matches!(error, CrawlError::MaximumDepthExceeded { .. }))
        );
    }

    #[test]
    fn empty_and_single_fetch_crawls_never_pause() {
        let pages = Pages {
            bodies: HashMap::new(),
            order: Mutex::default(),
        };
        crawl(&pages, &[], |_| true, || panic!("empty crawl paused"));
        crawl(
            &pages,
            &["https://shop.test/missing".into()],
            |_| true,
            || panic!("single fetch paused"),
        );
    }
}
