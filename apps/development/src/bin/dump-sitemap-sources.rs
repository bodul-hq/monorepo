//! Downloads every fetched sitemap file from `retailer_sourcing`'s
//! `/api/development/sitemaps/sources`, a page at a time, and writes each to
//! `<data_dir>/<retailer>/sitemaps/sources/<retrieval id>/<url slug>.xml`, the
//! retailer in lowercase as the data folders name it, and one folder per crawl.
//!
//! Usage: `RETAILER_SOURCING_URL=http://localhost:3000 dump-sitemap-sources <data_dir>`
//!
//! `RETAILER_SOURCING_URL` defaults to `http://localhost:3000`. The development
//! API needs a bearer token; `RETAILER_SOURCING_TOKEN` overrides the one the
//! service hardcodes for now.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::{env, fs, process};

use serde::Deserialize;
use sitemap::file_name;

/// How many sources to request per page. A shorter page is the last one.
const PAGE_SIZE: usize = 100;

const DEFAULT_URL: &str = "http://localhost:3000";

/// The bearer token `retailer_sourcing` hardcodes for its development API.
const DEFAULT_TOKEN: &str = "8bcb74f24405708e90b0aa1be5679f422ea2ba199ae8ab0d0f1e80acd8c9c97d";

#[derive(Deserialize)]
struct SourcesResponse {
    sources: Vec<Source>,
}

#[derive(Deserialize)]
struct Source {
    id: String,
    retrieval_id: String,
    url: String,
    retailer_code: Option<String>,
    body: String,
}

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
        .ok_or("usage: dump-sitemap-sources <data_dir>")?;
    let base_url = env::var("RETAILER_SOURCING_URL").unwrap_or_else(|_| DEFAULT_URL.to_string());
    let token = env::var("RETAILER_SOURCING_TOKEN").unwrap_or_else(|_| DEFAULT_TOKEN.to_string());
    let endpoint = format!(
        "{}/api/development/sitemaps/sources",
        base_url.trim_end_matches('/')
    );
    let client = reqwest::blocking::Client::new();

    // The URL written to each path, so two URLs with the same slug don't
    // overwrite each other.
    let mut written_urls = HashMap::new();
    let mut written = 0;
    let mut skipped = 0;
    let mut offset = 0;
    loop {
        let page = fetch_page(&client, &endpoint, &token, offset)?;
        for source in &page {
            match write_source(&data_dir, source, &mut written_urls) {
                Ok(path) => {
                    println!("{}", path.display());
                    written += 1;
                }
                Err(error) => {
                    eprintln!("skipped {}: {error}", source.id);
                    skipped += 1;
                }
            }
        }
        if page.len() < PAGE_SIZE {
            break;
        }
        offset += PAGE_SIZE;
    }

    println!(
        "wrote {written} sources to {}, skipped {skipped}",
        data_dir.display()
    );
    Ok(())
}

fn fetch_page(
    client: &reqwest::blocking::Client,
    endpoint: &str,
    token: &str,
    offset: usize,
) -> Result<Vec<Source>, String> {
    let url = format!("{endpoint}?limit={PAGE_SIZE}&offset={offset}");
    let response = client
        .get(&url)
        .bearer_auth(token)
        .send()
        .map_err(|e| format!("{url}: {}", with_causes(&e)))?;
    let status = response.status();
    if !status.is_success() {
        let body = response.text().unwrap_or_default();
        return Err(format!("{url}: {status} {body}"));
    }
    let page: SourcesResponse = response
        .json()
        .map_err(|e| format!("{url}: {}", with_causes(&e)))?;
    Ok(page.sources)
}

/// The error followed by each of its causes, since reqwest's own message leaves out why a request
/// failed, such as the connection being refused.
fn with_causes(error: &dyn std::error::Error) -> String {
    let mut message = error.to_string();
    let mut source = error.source();
    while let Some(cause) = source {
        message.push_str(&format!(": {cause}"));
        source = cause.source();
    }
    message
}

fn write_source(
    data_dir: &Path,
    source: &Source,
    written_urls: &mut HashMap<PathBuf, String>,
) -> Result<PathBuf, String> {
    // The API names the retailer as `UgreenEu`; its data folder is `ugreeneu`.
    let retailer = source
        .retailer_code
        .as_deref()
        .ok_or("no retailer; the command that fetched it is gone")?
        .to_ascii_lowercase();
    // Both end up in the path, so keep them from leaving the data folder.
    check_path_segment("retailer code", &retailer)?;
    check_path_segment("retrieval id", &source.retrieval_id)?;

    let dir = data_dir
        .join(&retailer)
        .join("sitemaps")
        .join("sources")
        .join(&source.retrieval_id);
    let path = dir.join(file_name(&source.url).map_err(|e| e.to_string())?);
    if let Some(other) = written_urls
        .get(&path)
        .filter(|other| **other != source.url)
    {
        return Err(format!(
            "{} and {other} would both be written to {}",
            source.url,
            path.display()
        ));
    }

    fs::create_dir_all(&dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    fs::write(&path, &source.body).map_err(|e| format!("{}: {e}", path.display()))?;
    written_urls.insert(path.clone(), source.url.clone());
    Ok(path)
}

fn check_path_segment(what: &str, value: &str) -> Result<(), String> {
    let valid = !value.is_empty()
        && value
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_');
    if valid {
        Ok(())
    } else {
        Err(format!("{what} {value:?} isn't safe to use as a file name"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_retailer_codes_and_uuids() {
        assert!(check_path_segment("retailer code", "ankerau").is_ok());
        assert!(check_path_segment("id", "01a1395d-efab-7113-83ed-e06eaad2efcd").is_ok());
    }

    #[test]
    fn rejects_segments_that_would_leave_the_folder() {
        for value in ["", ".", "..", "../etc", "a/b", "a\\b"] {
            assert!(check_path_segment("id", value).is_err(), "{value:?}");
        }
    }
}
