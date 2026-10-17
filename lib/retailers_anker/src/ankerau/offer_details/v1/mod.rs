//! Version 1 of the offer details.
//!
//! `anker.com/au` is a Next.js Pages Router storefront. Reads the page's two data islands
//! (`ld+json` and `__NEXT_DATA__`); the rendered DOM is not claimed.

use html_extractor::io::{Architecture, Attr, collection, particle, trash};
use html_extractor::serde_json;
use normalized::{AnkerAuNormalizedDocument, NormalizeError};
use raw::AnkerAuRawDocument;
use retailer_sauce::structuralized::StructuralizedData;
use structuralize::AnkerAuDocument;

pub mod normalized;
pub mod raw;
mod structuralize;

// #[cfg(test)]
// mod stage_files_tests;

pub fn architecture() -> Architecture {
    Architecture::new(vec![
        // Must run first: `trash` detaches its match, and svg elements sit inside body, so
        // anything claiming body ahead of this would carry them off first.
        trash(r#"svg"#),
        collection(
            r#"script[type="application/ld+json"]"#,
            "ld-json",
            vec![particle(
                "script",
                "",
                // `Attr::all()` alongside the JSON read: a stray attribute appearing on the tag
                // fails the strict raw schema instead of disappearing silently.
                vec![Attr::all(), Attr::json("", "data")],
            )],
        ),
        particle(
            "script#__NEXT_DATA__",
            "next_data",
            vec![Attr::all(), Attr::json("", "data")],
        ),
    ])
}

pub fn normalize(json: &str) -> Result<bodul_shared::NormalizedData, String> {
    let document = serde_json::from_str(json).map_err(|e| e.to_string())?;
    if crate::shared::is_other_storefront(&document, "au") {
        return Ok(bodul_shared::NormalizedData::just_document_json(None));
    }
    let raw: AnkerAuRawDocument = serde_json::from_value(document).map_err(|e| e.to_string())?;

    let normalized = AnkerAuNormalizedDocument::from_raw(raw)
        .map_err(|error: NormalizeError| error.to_string())?;

    let document_json = serde_json::to_string(&normalized).map_err(|e| e.to_string())?;

    Ok(bodul_shared::NormalizedData::just_document_json(
        document_json,
    ))
}

pub fn structuralize(json: &str) -> Result<StructuralizedData, String> {
    let normalized: AnkerAuNormalizedDocument =
        serde_json::from_str(json).map_err(|e| e.to_string())?;

    let document =
        AnkerAuDocument::from_normalized(normalized).map_err(|error| error.to_string())?;

    let document_json = serde_json::to_string(&document).map_err(|e| e.to_string())?;

    Ok(StructuralizedData::just_document_json(document_json))
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use html_extractor::Backend;

    use super::{architecture, normalize, structuralize};

    /// Saved product pages, kept outside the repository next to the monorepo checkout.
    const DATA_DIR: &str = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../../data/ankerau/offer_details/sources"
    );

    /// The saved pages, or `None` when the data checkout is missing so the tests skip.
    fn pages() -> Option<Vec<PathBuf>> {
        let dir = Path::new(DATA_DIR);
        if !dir.is_dir() {
            eprintln!("skipping: {} not found", dir.display());
            return None;
        }
        let mut pages: Vec<PathBuf> = std::fs::read_dir(dir)
            .expect("read data dir")
            .map(|entry| entry.expect("read data dir entry").path())
            // Exports name files after the URL path. Older crawls also saved other
            // regions here; those pages belong to their own storefront pipelines.
            .filter(|path| {
                path.extension().is_some_and(|ext| ext == "html")
                    && path
                        .file_name()
                        .and_then(|name| name.to_str())
                        .is_some_and(|name| name.starts_with("au-products-"))
            })
            .collect();
        pages.sort();
        assert!(
            !pages.is_empty(),
            "no au-products-*.html pages in {}",
            dir.display()
        );
        Some(pages)
    }

    fn read(page: &Path) -> String {
        std::fs::read_to_string(page).unwrap_or_else(|e| panic!("{}: {e}", page.display()))
    }

    #[test]
    fn extracts_product_and_next_data_from_every_page() {
        let Some(pages) = pages() else { return };
        let arch = architecture();
        for page in pages {
            let name = page.display();
            let json = Backend::Scraper.extract(&read(&page), &arch).unwrap().json;

            assert!(
                json["next_data"]["data"].is_object(),
                "{name}: no __NEXT_DATA__"
            );

            let ld_json = json["ld-json"]
                .as_array()
                .unwrap_or_else(|| panic!("{name}: no ld-json"));
            let products: Vec<_> = ld_json
                .iter()
                .map(|entry| &entry["data"])
                .filter(|data| data["@type"] == "Product")
                .collect();
            assert_eq!(products.len(), 1, "{name}: expected one Product");
            let product = products[0];

            let sku = product["sku"]
                .as_str()
                .unwrap_or_else(|| panic!("{name}: no sku"));
            // Not tied to the model number in the file name: bundles, multi-packs, `B`-prefixed
            // variants and slug-named pages all carry a SKU of their own.
            assert!(!sku.is_empty(), "{name}: empty sku");
            assert!(
                product["name"].as_str().is_some_and(|n| !n.is_empty()),
                "{name}: no name"
            );

            let offer = &product["offers"];
            assert_eq!(offer["priceCurrency"], "AUD", "{name}");
            assert!(
                offer["price"].is_number(),
                "{name}: price {}",
                offer["price"]
            );
            assert!(offer["availability"].is_string(), "{name}: no availability");
        }
    }

    #[test]
    fn backends_agree_on_every_page() {
        let Some(pages) = pages() else { return };
        let arch = architecture();
        for page in pages {
            let html = read(&page);
            let expected = Backend::Scraper.extract(&html, &arch).unwrap().json;
            for backend in Backend::ALL {
                let json = backend.extract(&html, &arch).unwrap().json;
                assert_eq!(json, expected, "{}: {backend:?}", page.display());
            }
        }
    }

    #[test]
    fn normalizes_and_structuralizes_every_page() {
        let Some(pages) = pages() else { return };
        let arch = architecture();
        let failures: Vec<String> = pages
            .iter()
            .filter_map(|page| {
                let json = Backend::Scraper.extract(&read(page), &arch).unwrap().json;
                let pipeline = normalize(&json.to_string())
                    .map_err(|e| format!("normalize: {e}"))
                    .and_then(|normalized| {
                        normalized
                            .document_json
                            .ok_or_else(|| "no normalized document".to_string())
                    })
                    .and_then(|normalized| {
                        structuralize(&normalized).map_err(|e| format!("structuralize: {e}"))
                    })
                    .and_then(|structuralized| {
                        structuralized
                            .document_json
                            .ok_or_else(|| "no structuralized document".to_string())
                    });
                pipeline.err().map(|e| format!("{}: {e}", page.display()))
            })
            .collect();
        assert!(failures.is_empty(), "{}", failures.join("\n"));
    }
}
