//! Version 1 of the offer details.
//!
//! `minisforum.hk` is a Shopify Liquid storefront whose theme does not render the
//! `xcotton_pp_variants` record the other Minisforum shops carry. Reads the PayPal metadata tag
//! (the shop currency), the schema.org islands, the variant list and the selected variant that the
//! theme's `bm_product_*` scripts render, the product id and title from `seal_product`, and the
//! variant selector's option names, the one place the page names its options; the rest of the
//! rendered DOM is not claimed.

use crate::minisforumhk::offer_details::v1::normalized::{
    MinisforumHkNormalizedDocument, NormalizeError,
};
use html_extractor::io::{Architecture, Attr, collection, particle, trash};
use html_extractor::serde_json;
use retailer_sauce::structuralized::StructuralizedData;
use structuralize::MinisforumHkDocument;

pub mod normalized;
pub mod raw;
mod structuralize;

// #[cfg(test)]
// mod stage_files_tests;

pub fn architecture() -> Architecture {
    Architecture::new(vec![
        trash(r#"svg"#),
        // The one statement of the shop currency on every page.
        particle(
            "meta#in-context-paypal-metadata",
            "paypal",
            vec![Attr::all()],
        ),
        // `Attr::all()` alongside the JSON reads: a stray attribute appearing on a tag fails the
        // strict raw schema instead of disappearing silently. Three islands per page: Product,
        // BreadcrumbList and Organization.
        collection(
            r#"script[type="application/ld+json"]"#,
            "ld-json",
            vec![particle(
                "script",
                "",
                vec![Attr::all(), Attr::json("", "data")],
            )],
        ),
        // Every variant of the product, as a JSON array.
        particle(
            "script#bm_product_variants",
            "variants",
            vec![Attr::all(), Attr::json("", "data")],
        ),
        // The variant the page opens on: the one in the URL, else the first available.
        particle(
            "script#bm_product_selected_or_first_available_variant",
            "selected_variant",
            vec![Attr::all(), Attr::json("", "data")],
        ),
        // The product id and title, which the variant list does not carry.
        particle(
            "script#seal_product",
            "seal_product",
            vec![Attr::all(), Attr::json("", "data")],
        ),
        // The variant selector's labels, such as `處理器:`: no script names the options, and a
        // product without options renders no selector.
        collection(
            "span.product-form__option-name",
            "option_names",
            vec![particle("span", "", vec![Attr::inner_text("text")])],
        ),
    ])
}

pub fn normalize(json: &str) -> Result<bodul_shared::NormalizedData, String> {
    let raw: raw::MinisforumHkRawDocument =
        serde_json::from_str(json).map_err(|e| e.to_string())?;

    let normalized = MinisforumHkNormalizedDocument::from_raw(raw)
        .map_err(|error: NormalizeError| error.to_string())?;

    let document_json = serde_json::to_string(&normalized).map_err(|e| e.to_string())?;

    Ok(bodul_shared::NormalizedData::just_document_json(
        document_json,
    ))
}

pub fn structuralize(json: &str) -> Result<StructuralizedData, String> {
    let normalized: MinisforumHkNormalizedDocument =
        serde_json::from_str(json).map_err(|e| e.to_string())?;

    let document =
        MinisforumHkDocument::from_normalized(normalized).map_err(|error| error.to_string())?;

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
        "/../../../data/minisforumhk/offer_details/sources"
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
    fn extracts_the_currency_islands_and_variants_from_every_page() {
        let Some(pages) = pages() else { return };
        let arch = architecture();
        for page in pages {
            let name = page.display();
            let json = Backend::Scraper.extract(&read(&page), &arch).unwrap().json;

            assert_eq!(json["paypal"]["data-currency"], "HKD", "{name}");

            let ld_json = json["ld-json"]
                .as_array()
                .unwrap_or_else(|| panic!("{name}: no ld-json"));
            assert!(
                ld_json
                    .iter()
                    .any(|entry| entry["data"]["@type"] == "Organization"),
                "{name}: no Organization island"
            );

            let seal = &json["seal_product"]["data"];
            assert!(seal["id"].is_u64(), "{name}: seal id {}", seal["id"]);
            assert!(
                seal["title"].as_str().is_some_and(|t| !t.is_empty()),
                "{name}: no title"
            );

            let variants = json["variants"]["data"]
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
            assert!(
                json["selected_variant"]["data"]["id"].is_u64(),
                "{name}: no selected variant"
            );

            // One selector label per option; a product without options has no selector and
            // Liquid's single `Default Title` variant.
            let option_names = json["option_names"]
                .as_array()
                .unwrap_or_else(|| panic!("{name}: no option_names"));
            let options = variants[0]["options"].as_array().expect("options");
            if variants[0]["title"] == "Default Title" {
                assert!(option_names.is_empty(), "{name}: labels for Default Title");
            } else {
                assert_eq!(option_names.len(), options.len(), "{name}: option labels");
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

    #[test]
    fn structuralizes_one_priced_variation_per_variant() {
        let Some(pages) = pages() else { return };
        let arch = architecture();
        let mut variations = 0;
        for page in pages {
            let name = page.display();
            let json = Backend::Scraper.extract(&read(&page), &arch).unwrap().json;
            let variants = json["variants"]["data"].as_array().expect("variants").len();

            let normalized = normalize(&json.to_string())
                .unwrap_or_else(|e| panic!("{name}: {e}"))
                .document_json
                .expect("a normalized document");
            let structuralized = structuralize(&normalized)
                .unwrap_or_else(|e| panic!("{name}: {e}"))
                .document_json
                .expect("a structuralized document");
            let document: serde_json::Value = serde_json::from_str(&structuralized).expect("JSON");

            let list = document["variations"].as_array().expect("variations");
            assert_eq!(list.len(), variants, "{name}");
            for variation in list {
                assert_eq!(variation["price"]["currency"], "HKD", "{name}");
                assert_eq!(
                    variation["images"].as_array().map(Vec::len),
                    Some(1),
                    "{name}"
                );
                assert!(
                    variation["url"]
                        .as_str()
                        .is_some_and(|url| url.starts_with("https://minisforum.hk/products/")),
                    "{name}: {}",
                    variation["url"]
                );
            }
            variations += list.len();
        }
        // The size of the saved corpus: every variant of the 29 pages.
        assert_eq!(variations, 78);
    }
}
