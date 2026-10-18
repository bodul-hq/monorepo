//! Processes the sitemap files `dump-sitemap-sources` wrote, the way
//! `retailer_sourcing` processes a crawl: each folder
//! `<data_dir>/<retailer>/sitemaps/sources/<retrieval id>/` becomes one sitemap
//! tree, starting at the retailer's root sitemaps, written to
//! `<data_dir>/<retailer>/sitemaps/processed/<retrieval id>.json`.
//!
//! Usage: `process-sitemap-sources <data_dir>`
//!
//! The dump keeps only each file's body, so the tree has no `Last-Modified`
//! dates from the fetch, only the `<lastmod>` dates the sitemap indexes list.

use std::borrow::Cow;
use std::collections::{BTreeSet, HashMap};
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::{env, fs, process};

use bodul_shared::retailer::RetailerCode;
use sitemap::{RawSitemapDocument, SitemapDocument, build_tree_with, file_name};

fn main() {
    if let Err(error) = run() {
        eprintln!("error: {error}");
        process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let data_dir = env::args()
        .nth(1)
        .map(PathBuf::from)
        .ok_or("usage: process-sitemap-sources <data_dir>")?;

    let mut processed = 0;
    let mut skipped = 0;
    for retailer_dir in subfolders(&data_dir)? {
        let sources_dir = retailer_dir.join("sitemaps").join("sources");
        if !sources_dir.is_dir() {
            continue;
        }
        let name = folder_name(&retailer_dir);
        let root_urls = match root_sitemaps(&name) {
            Ok(root_urls) => root_urls,
            Err(error) => {
                eprintln!("skipped {name}: {error}");
                skipped += 1;
                continue;
            }
        };

        let processed_dir = retailer_dir.join("sitemaps").join("processed");
        for retrieval_dir in subfolders(&sources_dir)? {
            let retrieval_id = folder_name(&retrieval_dir);
            let greylisted = |url: &str| sitemap_greylisted(&name, url);
            match process_retrieval(&retrieval_dir, &root_urls, greylisted) {
                Ok(retrieval) => {
                    let path = processed_dir.join(format!("{retrieval_id}.json"));
                    write_json(&path, &retrieval.document)?;
                    retrieval.report(&name, &retrieval_id);
                    processed += 1;
                }
                Err(error) => {
                    eprintln!("skipped {name} {retrieval_id}: {error}");
                    skipped += 1;
                }
            }
        }
    }

    println!(
        "processed {processed} retrievals in {}, skipped {skipped}",
        data_dir.display()
    );
    Ok(())
}

/// The root sitemaps of the retailer whose data folder is `name`, such as
/// `ugreeneu`.
fn root_sitemaps(name: &str) -> Result<Vec<String>, String> {
    let retailer = RetailerCode::try_from(name).map_err(|e| e.to_string())?;
    retailer_data_extractor::sitemap_config_for(&retailer)
        .map(|config| config.sitemap_url)
        .ok_or_else(|| "the retailer has no sitemap configuration".to_string())
}

/// Whether the retailer whose data folder is `name` downloads the child sitemap
/// at `url`. Its crawl never fetched one it doesn't, so the tree leaves it out.
fn sitemap_greylisted(name: &str, url: &str) -> bool {
    RetailerCode::try_from(name)
        .ok()
        .and_then(|retailer| retailer_data_extractor::sitemap_greylisted_for(retailer, url))
        .unwrap_or(true)
}

/// One retrieval's tree, and what went wrong building it.
struct Retrieval {
    document: SitemapDocument,
    /// Sitemaps left out of the tree, such as a root whose file is missing.
    left_out: Vec<String>,
    /// Files the tree needed but couldn't use.
    unreadable: Vec<String>,
    /// How many files the tree is built from.
    used: usize,
    /// Files in the folder the tree never reached.
    unused: usize,
}

impl Retrieval {
    fn report(&self, retailer: &str, retrieval_id: &str) {
        let url_count = self.document.all_urls("").count();
        println!(
            "{retailer} {retrieval_id}: {url_count} URLs from {} files, {} files unused",
            self.used, self.unused
        );
        for problem in self.unreadable.iter().chain(&self.left_out) {
            eprintln!("  {retailer} {retrieval_id}: {problem}");
        }
    }
}

fn process_retrieval(
    dir: &Path,
    root_urls: &[String],
    greylisted: impl Fn(&str) -> bool,
) -> Result<Retrieval, String> {
    // The URL each file was read for, since two URLs can share a file name.
    let mut read_for: HashMap<String, String> = HashMap::new();
    let mut unreadable = Vec::new();

    let (document, errors) = build_tree_with(root_urls, greylisted, |url| {
        let name = match file_name(url) {
            Ok(name) => name,
            Err(error) => {
                unreadable.push(error.to_string());
                return None;
            }
        };
        if let Some(other) = read_for.get(&name).filter(|other| *other != url) {
            unreadable.push(format!("{url} and {other} share the file {name}"));
            return None;
        }

        let body = match fs::read_to_string(dir.join(&name)) {
            Ok(body) => body,
            // Reported as left out of the tree.
            Err(error) if error.kind() == ErrorKind::NotFound => return None,
            Err(error) => {
                unreadable.push(format!("{name}: {error}"));
                return None;
            }
        };
        read_for.insert(name, url.to_string());
        Some(Cow::Owned(RawSitemapDocument {
            url: url.to_string(),
            last_modified: None,
            body_size: body.len(),
            body,
        }))
    })
    .map_err(|e| e.to_string())?;

    let files: BTreeSet<String> = fs::read_dir(dir)
        .map_err(|e| format!("{}: {e}", dir.display()))?
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.file_name().to_string_lossy().into_owned())
        .collect();
    let used = read_for.len();
    let unused = files
        .iter()
        .filter(|file| !read_for.contains_key(*file))
        .count();

    Ok(Retrieval {
        document,
        left_out: errors
            .iter()
            .map(|error| format!("left out of the tree: {error}"))
            .collect(),
        unreadable,
        used,
        unused,
    })
}

/// The folders directly inside `dir`, sorted by name.
fn subfolders(dir: &Path) -> Result<Vec<PathBuf>, String> {
    let mut folders: Vec<PathBuf> = fs::read_dir(dir)
        .map_err(|e| format!("{}: {e}", dir.display()))?
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| path.is_dir())
        .collect();
    folders.sort();
    Ok(folders)
}

fn folder_name(path: &Path) -> String {
    path.file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_default()
}

fn write_json(path: &Path, document: &SitemapDocument) -> Result<(), String> {
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    }
    let json = serde_json::to_string_pretty(document).map_err(|e| e.to_string())?;
    fs::write(path, json).map_err(|e| format!("{}: {e}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;

    const ROOT: &str = "https://shop.test/sitemap.xml";

    fn folder(files: &[(&str, &str)]) -> PathBuf {
        let dir = env::temp_dir().join(format!(
            "process-sitemap-sources-{}-{}",
            process::id(),
            files.len()
        ));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        for (name, body) in files {
            fs::write(dir.join(name), body).unwrap();
        }
        dir
    }

    #[test]
    fn builds_the_tree_from_the_files_named_after_their_urls() {
        let dir = folder(&[
            (
                "sitemap.xml",
                "<sitemapindex><sitemap><loc>https://shop.test/sitemap_products_1.xml?from=1&amp;to=2</loc></sitemap></sitemapindex>",
            ),
            (
                "sitemap_products_1_from=1_to=2.xml",
                "<urlset><url><loc>https://shop.test/products/a</loc></url><url><loc>https://shop.test/products/b</loc></url></urlset>",
            ),
            ("stray.xml", "<urlset></urlset>"),
        ]);

        let retrieval = process_retrieval(&dir, &[ROOT.to_string()], |_| true).unwrap();
        fs::remove_dir_all(&dir).unwrap();

        assert_eq!(retrieval.document.all_urls("").count(), 2);
        assert_eq!(retrieval.document.location.as_deref(), Some(ROOT));
        assert_eq!(retrieval.used, 2);
        assert_eq!(retrieval.unused, 1);
        assert!(retrieval.left_out.is_empty());
        assert!(retrieval.unreadable.is_empty());
    }

    #[test]
    fn leaves_out_a_root_whose_file_is_missing() {
        let dir = folder(&[]);

        let retrieval = process_retrieval(&dir, &[ROOT.to_string()], |_| true).unwrap();
        fs::remove_dir_all(&dir).unwrap();

        assert_eq!(retrieval.document.all_urls("").count(), 0);
        assert_eq!(retrieval.left_out.len(), 1);
    }
}
