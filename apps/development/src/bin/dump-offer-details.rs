//! Downloads every fetched offer page from `retailer_sourcing`'s
//! `/api/development/offer-details/sources`, a page at a time, and writes each to
//! `<data_dir>/<retailer>/offer_details/sources/<url path>.html`, the retailer in
//! lowercase as the data folders name it.
//!
//! The API lists pages newest first, so when a URL was fetched more than once
//! only its newest page is kept.
//!
//! Usage: `RETAILER_SOURCING_URL=http://localhost:3000 dump-offer-details <data_dir>`
//!
//! `RETAILER_SOURCING_URL` defaults to `http://localhost:3000`. The development
//! API needs a bearer token; `RETAILER_SOURCING_TOKEN` overrides the one the
//! service hardcodes for now.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::{env, fs, process};

use percent_encoding::percent_decode_str;
use reqwest::Url;
use serde::Deserialize;

/// How many sources to request per page. A shorter page is the last one.
const PAGE_SIZE: usize = 200;

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
    url: String,
    retailer_code: String,
    body: String,
}

enum Written {
    Newest(PathBuf),
    /// An older page of a URL already written.
    Older,
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
        .ok_or("usage: dump-offer-details <data_dir>")?;
    let base_url = env::var("RETAILER_SOURCING_URL").unwrap_or_else(|_| DEFAULT_URL.to_string());
    let token = env::var("RETAILER_SOURCING_TOKEN").unwrap_or_else(|_| DEFAULT_TOKEN.to_string());
    let endpoint = format!(
        "{}/api/development/offer-details/sources",
        base_url.trim_end_matches('/')
    );
    let client = reqwest::blocking::Client::new();

    // The URL written to each path, so an older page of the same URL, or
    // another URL with the same path, doesn't overwrite it.
    let mut written_urls = HashMap::new();
    let mut written = 0;
    let mut older = 0;
    let mut skipped = 0;
    let mut offset = 0;
    loop {
        let page = fetch_page(&client, &endpoint, &token, offset)?;
        for source in &page {
            match write_source(&data_dir, source, &mut written_urls) {
                Ok(Written::Newest(path)) => {
                    println!("{}", path.display());
                    written += 1;
                }
                Ok(Written::Older) => older += 1,
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
        "wrote {written} pages to {}, left out {older} older copies, skipped {skipped}",
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
) -> Result<Written, String> {
    // The data folders name retailers in lowercase, such as `ugreeneu`.
    let retailer = source.retailer_code.to_ascii_lowercase();
    // It ends up in the path, so keep it from leaving the data folder.
    check_path_segment("retailer code", &retailer)?;

    let dir = data_dir
        .join(&retailer)
        .join("offer_details")
        .join("sources");
    let path = dir.join(file_name(&source.url)?);
    if let Some(other) = written_urls.get(&path) {
        if *other == source.url {
            return Ok(Written::Older);
        }
        return Err(format!(
            "{} and {other} would both be written to {}",
            source.url,
            path.display()
        ));
    }

    fs::create_dir_all(&dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    fs::write(&path, &source.body).map_err(|e| format!("{}: {e}", path.display()))?;
    written_urls.insert(path.clone(), source.url.clone());
    Ok(Written::Newest(path))
}

/// The decoded URL path with `/` replaced by `-`, as `export_offer_sources`
/// names them: `https://www.anker.com/au/products/a1256` becomes
/// `au-products-a1256.html`.
fn file_name(url: &str) -> Result<String, String> {
    let url = Url::parse(url).map_err(|e| format!("{url}: {e}"))?;
    // Decoded, since percent-encoded non-Latin paths exceed the 255-byte limit
    // on file names.
    let path = percent_decode_str(url.path()).decode_utf8_lossy();
    let path = path.trim_matches('/').replace('/', "-");
    if path.is_empty() {
        return Err(format!("{url} has no path"));
    }
    if path.starts_with('.') {
        return Err(format!("{url} would be a hidden file"));
    }
    let name = format!("{path}.html");
    if name.len() > 255 {
        return Err(format!("{url} is too long for a file name"));
    }
    Ok(name)
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
    fn names_a_file_after_the_url_path() {
        assert_eq!(
            file_name("https://www.anker.com/au/products/a1256?variant=1").unwrap(),
            "au-products-a1256.html"
        );
    }

    #[test]
    fn decodes_a_percent_encoded_path() {
        assert_eq!(
            file_name("https://ankerkorea.co.kr/product/%EC%95%A4%EC%BB%A4-a3936/93/").unwrap(),
            "product-앤커-a3936-93.html"
        );
    }

    #[test]
    fn rejects_a_url_without_a_path() {
        assert!(file_name("https://www.anker.com/").is_err());
    }

    #[test]
    fn rejects_a_path_that_would_leave_the_folder() {
        assert!(file_name("https://www.anker.com/%2E%2E").is_err());
        assert_eq!(
            file_name("https://www.anker.com/a%2F..%2F..%2Fb").unwrap(),
            "a-..-..-b.html"
        );
    }

    #[test]
    fn keeps_the_newest_page_of_a_url() {
        let dir = env::temp_dir().join(format!("dump-offer-details-{}", process::id()));
        let source = |body: &str| Source {
            id: "1".to_string(),
            url: "https://www.anker.com/au/products/a1256".to_string(),
            retailer_code: "AnkerAu".to_string(),
            body: body.to_string(),
        };
        let mut written_urls = HashMap::new();

        let first = write_source(&dir, &source("newest"), &mut written_urls).unwrap();
        let second = write_source(&dir, &source("older"), &mut written_urls).unwrap();

        let Written::Newest(path) = first else {
            panic!("the first page wasn't written")
        };
        assert!(matches!(second, Written::Older));
        assert!(path.ends_with("ankerau/offer_details/sources/au-products-a1256.html"));
        assert_eq!(fs::read_to_string(&path).unwrap(), "newest");
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn rejects_unsafe_retailer_codes() {
        for value in ["", ".", "..", "../etc", "a/b", "a\\b"] {
            assert!(check_path_segment("id", value).is_err(), "{value:?}");
        }
    }
}
