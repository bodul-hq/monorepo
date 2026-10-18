//! Writes each retailer's latest sitemap crawl to `<data_dir>/<retailer>/sitemap`, one folder
//! per stage, each named for the crawl's retrieval id:
//!
//! - `sources/<retrieval id>/<url path>.xml`, every sitemap file as fetched,
//! - `processed/<retrieval id>.json`, the files parsed into one sitemap tree,
//! - `consolidated/<retrieval id>.json`, the tree's URLs grouped by kind of page, when the crawl
//!   has been consolidated.
//!
//! Requires DATABASE_URL, OBJECT_STORAGE_URL and OBJECT_STORAGE_TOKEN.
//! Usage: `export_sitemaps <data_dir>`

use retailer_sourcing::json;
use retailer_sourcing::object_storage::{
    CONSOLIDATED_SITEMAPS_BUCKET, ObjectStorage, ObjectStorageMetadata, PROCESSED_SITEMAPS_BUCKET,
    RAW_SITEMAPS_BUCKET,
};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::{env, fs, process};

use bodul_shared::retailer::RetailerCode;
use diesel::prelude::*;
use diesel::sql_types::{Jsonb, Nullable, Text, Uuid as SqlUuid};
use percent_encoding::percent_decode_str;
use reqwest::Url;
use serde_json::Value;
use uuid::Uuid;

/// Each retailer's most recently processed crawl, with its consolidated content if it has one.
const CRAWLS_QUERY: &str = include_str!("export_sitemaps.sql");

/// The sitemap files fetched by the crawl `$1`.
const SOURCES_QUERY: &str = "
    SELECT r.id, r.url, r.storage
    FROM sitemaps_raw r
    WHERE r.retrieval_id = $1
    ORDER BY r.url";

struct Crawl {
    retailer_code: String,
    retrieval_id: Uuid,
    document: Value,
    content: Option<Value>,
}

#[derive(QueryableByName)]
struct CrawlRecord {
    #[diesel(sql_type = SqlUuid)]
    id: Uuid,
    #[diesel(sql_type = Text)]
    retailer_code: String,
    #[diesel(sql_type = SqlUuid)]
    retrieval_id: Uuid,
    #[diesel(sql_type = Jsonb)]
    storage: json::Json<ObjectStorageMetadata>,
    #[diesel(sql_type = Nullable<SqlUuid>)]
    consolidated_id: Option<Uuid>,
    #[diesel(sql_type = Nullable<Jsonb>)]
    consolidated_storage: Option<json::Json<ObjectStorageMetadata>>,
}

fn fetch_crawl(record: CrawlRecord, objects: &ObjectStorage) -> Result<Crawl, String> {
    let body = objects
        .get(PROCESSED_SITEMAPS_BUCKET, record.id, &record.storage.0)
        .map_err(|e| e.to_string())?;
    let document = serde_json::from_str(&body).map_err(|e| e.to_string())?;
    let content = match (record.consolidated_id, record.consolidated_storage) {
        (Some(id), Some(storage)) => {
            let body = objects
                .get(CONSOLIDATED_SITEMAPS_BUCKET, id, &storage.0)
                .map_err(|e| e.to_string())?;
            Some(serde_json::from_str(&body).map_err(|e| e.to_string())?)
        }
        (None, None) => None,
        _ => return Err("incomplete consolidated sitemap reference".into()),
    };
    Ok(Crawl {
        retailer_code: record.retailer_code,
        retrieval_id: record.retrieval_id,
        document,
        content,
    })
}

struct Source {
    url: String,
    body: String,
}

#[derive(QueryableByName)]
struct SourceRecord {
    #[diesel(sql_type = SqlUuid)]
    id: Uuid,
    #[diesel(sql_type = Text)]
    url: String,
    #[diesel(sql_type = Jsonb)]
    storage: json::Json<ObjectStorageMetadata>,
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
        .ok_or("usage: export_sitemaps <data_dir>")?;
    let database_url = env::var("DATABASE_URL").map_err(|_| "DATABASE_URL must be set")?;
    let runtime = tokio::runtime::Runtime::new().map_err(|error| error.to_string())?;
    let objects =
        ObjectStorage::from_env(runtime.handle().clone()).map_err(|error| error.to_string())?;
    let mut connection = PgConnection::establish(&database_url).map_err(|e| e.to_string())?;

    let crawls: Vec<CrawlRecord> = diesel::sql_query(CRAWLS_QUERY)
        .load(&mut connection)
        .map_err(|e| e.to_string())?;

    drop(connection);
    let mut exported = 0;
    let mut skipped = 0;
    for record in crawls {
        let crawl = fetch_crawl(record, &objects)?;
        let mut connection = PgConnection::establish(&database_url).map_err(|e| e.to_string())?;
        let records: Vec<SourceRecord> = diesel::sql_query(SOURCES_QUERY)
            .bind::<SqlUuid, _>(crawl.retrieval_id)
            .load(&mut connection)
            .map_err(|e| e.to_string())?;

        drop(connection);
        let sources = fetch_sources(records, &objects, crawl.retrieval_id)?;
        match write_crawl(&data_dir, &crawl, &sources) {
            Ok(()) => {
                println!(
                    "{} {}: {} sources, {}",
                    crawl.retailer_code,
                    crawl.retrieval_id,
                    sources.len(),
                    if crawl.content.is_some() {
                        "processed and consolidated"
                    } else {
                        "processed, not consolidated yet"
                    }
                );
                exported += 1;
            }
            Err(error) => {
                eprintln!("skipped {}: {error}", crawl.retailer_code);
                skipped += 1;
            }
        }
    }

    println!(
        "exported {exported} sitemaps to {}, skipped {skipped}",
        data_dir.display()
    );
    Ok(())
}

fn fetch_sources(
    records: Vec<SourceRecord>,
    objects: &ObjectStorage,
    retrieval_id: Uuid,
) -> Result<Vec<Source>, String> {
    if records.is_empty() {
        return Err(format!(
            "crawl {retrieval_id} has no raw sources; retrieve fresh sitemaps after the object-storage cutover"
        ));
    }
    records
        .into_iter()
        .map(|record| {
            let body = objects
                .get(RAW_SITEMAPS_BUCKET, record.id, &record.storage.0)
                .map_err(|e| e.to_string())?;
            Ok(Source {
                url: record.url,
                body,
            })
        })
        .collect()
}

fn write_crawl(data_dir: &Path, crawl: &Crawl, sources: &[Source]) -> Result<(), String> {
    let retailer =
        RetailerCode::try_from(crawl.retailer_code.as_str()).map_err(|e| e.to_string())?;
    let dir = data_dir.join(retailer.as_str()).join("sitemap");

    // Named before anything is written, so a crawl whose URLs collide leaves no partial export.
    let mut named = BTreeMap::new();
    for source in sources {
        let name = file_name(&source.url)?;
        if let Some(other) = named.insert(name.clone(), source) {
            return Err(format!(
                "{} and {} would both be written to {name}",
                other.url, source.url
            ));
        }
    }

    let retrieval_id = crawl.retrieval_id.to_string();
    let sources_dir = dir.join("sources").join(&retrieval_id);
    fs::create_dir_all(&sources_dir).map_err(|e| e.to_string())?;
    for (name, source) in named {
        write(&sources_dir.join(name), &source.body)?;
    }

    write_json(&dir.join("processed"), &retrieval_id, &crawl.document)?;
    if let Some(content) = &crawl.content {
        write_json(&dir.join("consolidated"), &retrieval_id, content)?;
    }
    Ok(())
}

fn write_json(dir: &Path, retrieval_id: &str, value: &Value) -> Result<(), String> {
    fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    let json = serde_json::to_string_pretty(value).map_err(|e| e.to_string())?;
    write(&dir.join(format!("{retrieval_id}.json")), &json)
}

fn write(path: &Path, contents: &str) -> Result<(), String> {
    fs::write(path, contents).map_err(|e| format!("{}: {e}", path.display()))
}

/// The decoded URL path with `/` replaced by `-`, as for offer pages, followed by the query with
/// `&` replaced by `_`, since Shopify pages its sitemaps by query:
/// `https://www.ankerjapan.com/sitemap_products_1.xml?from=1&to=2` becomes
/// `sitemap_products_1_from=1_to=2.xml`.
fn file_name(url: &str) -> Result<String, String> {
    let url = Url::parse(url).map_err(|e| e.to_string())?;
    // Decoded, since percent-encoded non-Latin paths exceed the 255-byte limit
    // on file names.
    let path = percent_decode_str(url.path()).decode_utf8_lossy();
    let path = path.trim_matches('/').replace('/', "-");
    if path.is_empty() {
        return Err("the URL has no path".to_string());
    }

    let stem = path.strip_suffix(".xml").unwrap_or(&path);
    match url.query().filter(|query| !query.is_empty()) {
        Some(query) => Ok(format!("{stem}_{}.xml", query.replace('&', "_"))),
        None => Ok(format!("{stem}.xml")),
    }
}

#[cfg(test)]
#[path = "../../tests/support/mod.rs"]
mod test_objects;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_a_file_after_the_url_path() {
        assert_eq!(
            file_name("https://www.anker.com/server-sitemap-index-products.xml").unwrap(),
            "server-sitemap-index-products.xml"
        );
        assert_eq!(
            file_name("https://www.anker.com/au/sitemap/products.xml").unwrap(),
            "au-sitemap-products.xml"
        );
    }

    #[test]
    fn keeps_the_query_that_pages_a_shopify_sitemap() {
        assert_eq!(
            file_name(
                "https://www.ankerjapan.com/sitemap_products_1.xml?from=5917111746721&to=7761509187745"
            )
            .unwrap(),
            "sitemap_products_1_from=5917111746721_to=7761509187745.xml"
        );
    }

    #[test]
    fn gives_a_file_without_an_xml_extension_one() {
        assert_eq!(
            file_name("https://example.com/sitemap").unwrap(),
            "sitemap.xml"
        );
    }

    #[test]
    fn rejects_a_url_without_a_path() {
        assert!(file_name("https://www.anker.com/").is_err());
    }
    #[test]
    fn exports_object_storage_bodies_and_rejects_missing_sources() {
        let server = test_objects::Server::new();
        let objects = server.client();
        let id = Uuid::now_v7();
        let body = "<urlset>ž 🔎</urlset>";
        let object_id = objects.create(RAW_SITEMAPS_BUCKET, id, body).unwrap();
        let records = || {
            vec![SourceRecord {
                id,
                url: "https://shop.test/sitemap.xml".into(),
                storage: json::Json(ObjectStorageMetadata {
                    content_size: body.len() as u64,
                    object_storage_id: object_id,
                }),
            }]
        };
        let crawl = Crawl {
            retailer_code: "ankerau".into(),
            retrieval_id: Uuid::now_v7(),
            document: serde_json::json!({}),
            content: None,
        };
        let sources = fetch_sources(records(), &objects, crawl.retrieval_id).unwrap();
        let output = std::env::temp_dir().join(format!("sitemap-export-{}", Uuid::now_v7()));
        write_crawl(&output, &crawl, &sources).unwrap();
        let path = output
            .join("ankerau/sitemap/sources")
            .join(crawl.retrieval_id.to_string())
            .join("sitemap.xml");
        assert_eq!(std::fs::read_to_string(path).unwrap(), body);
        std::fs::remove_dir_all(output).unwrap();
        assert!(fetch_sources(vec![], &objects, crawl.retrieval_id).is_err());
        server.data.lock().unwrap().get_response = Some((404, "{}".into()));
        assert!(fetch_sources(records(), &objects, crawl.retrieval_id).is_err());
    }

    #[test]
    fn exports_processed_and_consolidated_objects_and_rejects_invalid_content() {
        let server = test_objects::Server::new();
        let objects = server.client();
        let id = Uuid::now_v7();
        let consolidated_id = Uuid::now_v7();
        let document = serde_json::json!({"urls": ["ž 🔎"]});
        let content = serde_json::json!({"product": ["https://shop.test/a"]});
        let upload = |bucket, id, body: &str| {
            json::Json(ObjectStorageMetadata {
                content_size: body.len() as u64,
                object_storage_id: objects.create(bucket, id, body).unwrap(),
            })
        };
        let storage = upload(PROCESSED_SITEMAPS_BUCKET, id, &document.to_string());
        let consolidated_storage = upload(
            CONSOLIDATED_SITEMAPS_BUCKET,
            consolidated_id,
            &content.to_string(),
        );
        let record = || CrawlRecord {
            id,
            retailer_code: "ankerau".into(),
            retrieval_id: Uuid::now_v7(),
            storage: storage.clone(),
            consolidated_id: Some(consolidated_id),
            consolidated_storage: Some(consolidated_storage.clone()),
        };
        let crawl = fetch_crawl(record(), &objects).unwrap();
        assert_eq!(crawl.document, document);
        assert_eq!(crawl.content, Some(content.clone()));
        let dir = std::env::temp_dir().join(format!("crawl-export-{}", Uuid::now_v7()));
        write_crawl(&dir, &crawl, &[]).unwrap();
        let base = dir.join("ankerau/sitemap");
        for (stage, expected) in [("processed", document), ("consolidated", content)] {
            let text = fs::read_to_string(
                base.join(stage)
                    .join(format!("{}.json", crawl.retrieval_id)),
            )
            .unwrap();
            assert_eq!(serde_json::from_str::<Value>(&text).unwrap(), expected);
        }
        fs::remove_dir_all(dir).unwrap();
        let mut unconsolidated = record();
        unconsolidated.consolidated_id = None;
        unconsolidated.consolidated_storage = None;
        assert!(
            fetch_crawl(unconsolidated, &objects)
                .unwrap()
                .content
                .is_none()
        );
        let mut invalid = record();
        invalid.id = Uuid::now_v7();
        invalid.storage = upload(PROCESSED_SITEMAPS_BUCKET, invalid.id, "not json");
        assert!(fetch_crawl(invalid, &objects).is_err());
        server.data.lock().unwrap().get_response = Some((404, "{}".into()));
        assert!(fetch_crawl(record(), &objects).is_err());
    }
}
