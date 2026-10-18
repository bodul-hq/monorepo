//! Version 1 of the offer details.
//!
//! `minisforumpc.fr` is a Shopify Liquid storefront with the same page layout as
//! `minisforumeu`, rendered in two locales: French at `/products/<handle>` and English at
//! `/en/products/<handle>`. Reads the PayPal metadata tag (the shop currency and the page
//! locale), the schema.org islands and the app-rendered `product | json` in
//! `script#xcotton_pp_variants`; the rendered DOM is not claimed.

use crate::minisforumfr::offer_details::v1::normalized::{
    MinisforumFrNormalizedDocument, NormalizeError,
};
use html_extractor::io::{Architecture, Attr, collection, particle, trash};
use html_extractor::serde_json;
use retailer_sauce::structuralized::StructuralizedData;
use structuralize::MinisforumFrDocument;

pub mod normalized;
pub mod raw;
mod structuralize;

// #[cfg(test)]
// mod stage_files_tests;

pub fn architecture() -> Architecture {
    Architecture::new(vec![
        trash(r#"svg"#),
        // The one statement of the shop currency on every page: Liquid's record names none, and
        // the schema.org island is missing where the product has no price.
        particle(
            "meta#in-context-paypal-metadata",
            "paypal",
            vec![Attr::all()],
        ),
        // `Attr::all()` alongside the JSON reads: a stray attribute appearing on a tag fails the
        // strict raw schema instead of disappearing silently.
        collection(
            r#"script[type="application/ld+json"]"#,
            "ld-json",
            vec![particle(
                "script",
                "",
                vec![Attr::all(), Attr::json("", "data")],
            )],
        ),
        // Liquid's `product | json`, rendered by a storefront app on every product page,
        // including the one whose product is announced without a price.
        particle(
            "script#xcotton_pp_variants",
            "product_json",
            vec![Attr::all(), Attr::json("", "data")],
        ),
    ])
}

pub fn normalize(json: &str) -> Result<bodul_shared::NormalizedData, String> {
    let raw: raw::MinisforumFrRawDocument =
        serde_json::from_str(json).map_err(|e| e.to_string())?;

    let normalized = MinisforumFrNormalizedDocument::from_raw(raw)
        .map_err(|error: NormalizeError| error.to_string())?;

    let document_json = serde_json::to_string(&normalized).map_err(|e| e.to_string())?;

    Ok(bodul_shared::NormalizedData::just_document_json(
        document_json,
    ))
}

pub fn structuralize(json: &str) -> Result<StructuralizedData, String> {
    let normalized: MinisforumFrNormalizedDocument =
        serde_json::from_str(json).map_err(|e| e.to_string())?;

    let document =
        MinisforumFrDocument::from_normalized(normalized).map_err(|error| error.to_string())?;

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
        "/../../../data/minisforumfr/offer_details/sources"
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
            .filter(|path| path.extension().is_some_and(|ext| ext == "html"))
            .collect();
        pages.sort();
        assert!(!pages.is_empty(), "no .html pages in {}", dir.display());
        Some(pages)
    }

    fn read(page: &Path) -> String {
        std::fs::read_to_string(page).unwrap_or_else(|e| panic!("{}: {e}", page.display()))
    }

    #[test]
    fn extracts_the_currency_islands_and_record_from_every_page() {
        let Some(pages) = pages() else { return };
        let arch = architecture();
        for page in pages {
            let name = page.display();
            let json = Backend::Scraper.extract(&read(&page), &arch).unwrap().json;

            assert_eq!(json["paypal"]["data-currency"], "EUR", "{name}");

            let ld_json = json["ld-json"]
                .as_array()
                .unwrap_or_else(|| panic!("{name}: no ld-json"));
            assert!(
                ld_json
                    .iter()
                    .any(|entry| entry["data"]["@type"] == "Organization"),
                "{name}: no Organization island"
            );

            let record = &json["product_json"]["data"];
            assert!(
                record["title"].as_str().is_some_and(|t| !t.is_empty()),
                "{name}: no title"
            );
            assert!(
                record["handle"].as_str().is_some_and(|h| !h.is_empty()),
                "{name}: no handle"
            );
            let variants = record["variants"]
                .as_array()
                .unwrap_or_else(|| panic!("{name}: no variants"));
            assert!(!variants.is_empty(), "{name}: no variants");
            for variant in variants {
                assert!(
                    variant["id"].is_u64(),
                    "{name}: variant id {}",
                    variant["id"]
                );
                assert!(
                    variant["price"].is_i64(),
                    "{name}: price {}",
                    variant["price"]
                );
            }
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
