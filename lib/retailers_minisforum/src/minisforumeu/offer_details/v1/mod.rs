//! Version 1 of the offer details.
//!
//! `minisforumpc.eu` is a Shopify Liquid storefront with the same page layout as
//! `minisforumau`. Reads the PayPal metadata tag (the shop currency and the page's locale), the
//! schema.org islands and the app-rendered `product | json` in `script#xcotton_pp_variants`;
//! the rendered DOM is not claimed.
//!
//! What the 274 saved pages (137 products, each in two locales) show, and where it is handled:
//!
//! - Every product is served as `/products/<handle>` (`en_US`) and `/de/products/<handle>`
//!   (`de_DE`); the sitemap lists both as products. The same record is rendered in both, with
//!   translated text, so the two are separate offers with the same ids. The locale comes from
//!   the PayPal tag and decides the URL prefix (`normalized`).
//! - 3 to 5 ld+json islands, from four apps (`raw::ldjson`). Only the theme's `ProductGroup` or
//!   `Product` is checked against the record; the rest are typed but unread.
//! - The theme's island is absent on 2 pages, the product `s5-all-flash-nas` that is announced
//!   without a price; its one variant is priced at 0 and has no SKU.
//! - 18 pages have a feed-app price such as `4.159.00`, which no number parser reads; that
//!   island is why the record, not an island, is the source of the price.
//! - 8 pages carry a video in the record's `media`, which is never read.

use crate::minisforumeu::offer_details::v1::normalized::{
    MinisforumEuNormalizedDocument, NormalizeError,
};
use html_extractor::io::{Architecture, Attr, collection, particle, trash};
use html_extractor::serde_json;
use retailer_sauce::structuralized::StructuralizedData;
use structuralize::MinisforumEuDocument;

pub mod normalized;
pub mod raw;
mod structuralize;

// #[cfg(test)]
// mod stage_files_tests;

pub fn architecture() -> Architecture {
    Architecture::new(vec![
        trash(r#"svg"#),
        // The one statement of the shop currency and locale on every page: Liquid's record names
        // neither, and the schema.org islands disagree on how they format a price.
        particle(
            "meta#in-context-paypal-metadata",
            "paypal",
            vec![Attr::all()],
        ),
        // `Attr::all()` alongside the JSON reads: a stray attribute appearing on a tag fails the
        // strict raw schema instead of disappearing silently. Four to five islands per page: the
        // theme's Product and ProductGroup, the SEO app's `@graph`, the Organization, and the
        // reviews app's Product on pages that have reviews.
        collection(
            r#"script[type="application/ld+json"]"#,
            "ld-json",
            vec![particle(
                "script",
                "",
                vec![Attr::all(), Attr::json("", "data")],
            )],
        ),
        // Liquid's `product | json`, rendered by a storefront app on every product page.
        particle(
            "script#xcotton_pp_variants",
            "product_json",
            vec![Attr::all(), Attr::json("", "data")],
        ),
    ])
}

pub fn normalize(json: &str) -> Result<bodul_shared::NormalizedData, String> {
    let raw: raw::MinisforumEuRawDocument =
        serde_json::from_str(json).map_err(|e| e.to_string())?;

    let normalized = MinisforumEuNormalizedDocument::from_raw(raw)
        .map_err(|error: NormalizeError| error.to_string())?;

    let document_json = serde_json::to_string(&normalized).map_err(|e| e.to_string())?;

    Ok(bodul_shared::NormalizedData::just_document_json(
        document_json,
    ))
}

pub fn structuralize(json: &str) -> Result<StructuralizedData, String> {
    let normalized: MinisforumEuNormalizedDocument =
        serde_json::from_str(json).map_err(|e| e.to_string())?;

    let document =
        MinisforumEuDocument::from_normalized(normalized).map_err(|error| error.to_string())?;

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
        "/../../../data/minisforumeu/offer_details/sources"
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
