//! Consolidates the sitemap trees `process-sitemap-sources` wrote, the way
//! `retailer_sourcing` consolidates a processed sitemap: each file
//! `<data_dir>/<retailer>/sitemaps/processed/<retrieval id>.json` has its URLs
//! sorted by kind of page with the retailer's link rules, and the groups are
//! written to `<data_dir>/<retailer>/sitemaps/consolidate/<retrieval id>.json`.
//!
//! Usage: `consolidate-sitemap-sources <data_dir>`

use std::path::{Path, PathBuf};
use std::{env, fs, process};

use bodul_shared::link::LinkKind;
use bodul_shared::retailer::RetailerCode;
use sitemap::{GroupedLinks, SitemapDocument, SitemapUrl};

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
        .ok_or("usage: consolidate-sitemap-sources <data_dir>")?;

    let mut consolidated = 0;
    let mut skipped = 0;
    for retailer_dir in subfolders(&data_dir)? {
        let processed_dir = retailer_dir.join("sitemaps").join("processed");
        if !processed_dir.is_dir() {
            continue;
        }
        let name = file_name(&retailer_dir);
        let retailer = match RetailerCode::try_from(name.as_str()) {
            Ok(retailer) => retailer,
            Err(error) => {
                eprintln!("skipped {name}: {error}");
                skipped += 1;
                continue;
            }
        };

        let consolidate_dir = retailer_dir.join("sitemaps").join("consolidate");
        for path in json_files(&processed_dir)? {
            let file = file_name(&path);
            let retrieval_id = file.trim_end_matches(".json");
            match read_document(&path) {
                Ok(document) => {
                    let links = consolidate(retailer, &document);
                    write_json(&consolidate_dir.join(&file), &links)?;
                    report(&name, retrieval_id, &links);
                    consolidated += 1;
                }
                Err(error) => {
                    eprintln!("skipped {name} {retrieval_id}: {error}");
                    skipped += 1;
                }
            }
        }
    }

    println!(
        "consolidated {consolidated} retrievals in {}, skipped {skipped}",
        data_dir.display()
    );
    Ok(())
}

/// Sorts the tree's URLs by kind with the retailer's link rules, the way
/// `retailer_sourcing` does.
fn consolidate(retailer: RetailerCode, document: &SitemapDocument) -> GroupedLinks {
    GroupedLinks::from_document(document, |url: &SitemapUrl| {
        retailer_data_extractor::classify_link_for(
            retailer,
            &url.location,
            &url.source,
            url.images.len(),
        )
        .unwrap_or(LinkKind::Unknown)
    })
}

fn report(retailer: &str, retrieval_id: &str, links: &GroupedLinks) {
    println!(
        "{retailer} {retrieval_id}: {} products, {} catalogs, {} content, {} not interested, {} unknown",
        links.product.len(),
        links.catalog.len(),
        links.content.len(),
        links.not_interested.len(),
        links.unknown.len()
    );
}

fn read_document(path: &Path) -> Result<SitemapDocument, String> {
    let json = fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
    serde_json::from_str(&json).map_err(|e| format!("{}: {e}", path.display()))
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

/// The `.json` files directly inside `dir`, sorted by name.
fn json_files(dir: &Path) -> Result<Vec<PathBuf>, String> {
    let mut files: Vec<PathBuf> = fs::read_dir(dir)
        .map_err(|e| format!("{}: {e}", dir.display()))?
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| path.is_file() && path.extension().is_some_and(|ext| ext == "json"))
        .collect();
    files.sort();
    Ok(files)
}

fn file_name(path: &Path) -> String {
    path.file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_default()
}

fn write_json(path: &Path, links: &GroupedLinks) -> Result<(), String> {
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    }
    let json = serde_json::to_string_pretty(links).map_err(|e| e.to_string())?;
    fs::write(path, json).map_err(|e| format!("{}: {e}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_a_processed_tree_and_groups_its_urls() {
        let dir = env::temp_dir().join(format!("consolidate-sitemap-sources-{}", process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("tree.json");
        let tree = SitemapDocument {
            urls: vec![SitemapUrl {
                location: "https://shop.test/a".to_string(),
                source: String::new(),
                last_modified: None,
                change_frequency: None,
                priority: None,
                images: Vec::new(),
            }],
            ..SitemapDocument::default()
        };
        fs::write(&path, serde_json::to_string(&tree).unwrap()).unwrap();

        let document = read_document(&path).unwrap();
        let links = GroupedLinks::from_document(&document, |_| LinkKind::Unknown);
        fs::remove_dir_all(&dir).unwrap();

        assert_eq!(links.unknown, vec!["https://shop.test/a"]);
    }
}
