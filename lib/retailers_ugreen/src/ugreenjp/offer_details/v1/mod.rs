//! Version 1 of the offer details.
//!
//! `ugreen.jp` is a Shopify Liquid storefront on a Dawn-derived theme (`dragon-jp/main`, schema
//! `Digital` 1.0.4, the same theme family as `ugreennasjp`'s `nas-jp/main`). Survey of the 132
//! saved pages:
//!
//! - Every page carries two valid ld+json islands, an `Organization` and a `Product`. The
//!   `Product`'s `offers` is always a list with one `Offer` per variant (`url` ends in
//!   `?variant=<id>`), so it holds the whole product: `name`, a single featured `image`, `sku`,
//!   and per offer `price` as a JPY float (`13980.0`), `priceCurrency` (`JPY`), `availability`
//!   and `sku`. The `sku` is missing on the 19 variants that have none anywhere on the page. No
//!   GTIN.
//! - Every page carries Shopify's analytics `meta`: product id and handle, and the same variants
//!   (same ids, same prices) with `price` in Shopify's minor units (`1398000` is ¥13,980: JPY has
//!   no minor unit, but Shopify still multiplies by 100), `sku` and `public_title` (`null` on a
//!   `Default Title` product). It names no currency.
//! - Every page carries the PayPal metadata tag (`data-currency="JPY"`); there is no Apple Pay
//!   shop-capabilities script.
//! - The 69 products with a variant picker render Liquid's variants in a
//!   `variant-radios script[type="application/json"]`: `available`, `compare_at_price`,
//!   `barcode` (null on every variant), options. The 63 `Default Title` products render no picker
//!   and so no such script.
//! - The rendered price block is the only compare-at price of a `Default Title` product: two NAS
//!   pages are `price--on-sale` there (¥52,700 against ¥65,880) with nothing in any JSON island.
//!   Where the picker's JSON sets a compare-at price, it is either the price itself (8 variants)
//!   or `0` (1 variant), never a discount.
//!
//! The rendered DOM is read only for the main product section's price block, its selected
//! variant and its gallery.

use html_extractor::io::{Architecture, Attr, collection, particle, segment, trash};
use html_extractor::serde_json;
use normalized::{NormalizeError, UgreenJpNormalizedDocument};
use retailer_sauce::structuralized::StructuralizedData;
use structuralize::UgreenJpDocument;

pub mod normalized;
pub mod raw;
mod structuralize;

/// The main product section; recommendations and the footer sit in other sections.
const MAIN: &str = r#"[id^="shopify-section-template--"][id$="__main"]"#;

fn main(selector: &str) -> String {
    format!("{MAIN} {selector}")
}

pub fn architecture() -> Architecture {
    Architecture::new(vec![
        // Must run first: `trash` detaches its match, and svg elements sit inside body, so
        // anything claiming body ahead of this would carry them off first.
        trash(r#"svg"#),
        // The shop currency, stated once per page; the analytics prices name none.
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
        // Shopify's analytics `meta` sits in an anonymous inline script just before
        // `script.analytics`. The other storefronts' `script:has(+ script.analytics)` parses only
        // on the Scraper backend; the Kuchiki ones reject `:has`. The script itself has no id or
        // attribute on any saved page, so it is anchored forwards instead, through Shopify's fixed
        // `content_for_header` sequence: the `monorail-edge` dns-prefetch link, then the
        // session-beacon, Trekkie-shim, web-pixels-loader and `meta` scripts, identical on all
        // 132 saved pages. If Shopify changes that sequence, this lands on a script without
        // `var meta` and `analytics` comes out without `meta`.
        particle(
            r#"link[rel="dns-prefetch"][href="https://monorail-edge.shopifysvc.com"] + script + script + script + script"#,
            "analytics",
            vec![Attr::all(), Attr::javascript_vars("meta", "meta")],
        ),
        // The picker's variants with `available` and `compare_at_price`; missing on a
        // `Default Title` product, which renders no picker.
        particle(
            &main(r#"variant-radios script[type="application/json"]"#),
            "variants",
            vec![Attr::all(), Attr::json("", "data")],
        ),
        // The variant the price block below is rendered for.
        particle(
            &main(r#"form[action$="/cart/add"] input[name="id"]"#),
            "selected_variant_id",
            vec![Attr::standard("value")],
        ),
        // Dawn's price snippet. `price--on-sale` in the class marks a real compare-at price; when
        // there is none, the `<s>` under `.price__sale` is empty (94 pages), repeats the price (35)
        // or reads `¥0` (1, a variant whose `compare_at_price` is `0`).
        segment(
            &main(".price--product"),
            "price",
            vec![
                particle(".price--product", "", vec![Attr::standard("class")]),
                particle(
                    ".price__regular .price-item--regular",
                    "regular",
                    vec![Attr::inner_text("text")],
                ),
                particle(
                    ".price__sale .price-item--sale",
                    "sale",
                    vec![Attr::inner_text("text")],
                ),
                particle(
                    ".price__sale s.price-item--regular",
                    "compare_at",
                    vec![Attr::inner_text("text")],
                ),
            ],
        ),
        // The slider's slides; `.product__media-grid-noscript` repeats them for no-JS visitors.
        // Every slide is an image (no video or 3D model on any saved page), and the one inside
        // `modal-opener` is the rendered one; `<noscript>` holds a copy. `data-media-id` is the
        // media id prefixed with the section id (`template--…__main-44439502291224`).
        collection(
            &main(".product__media-list.js-media-list .product__media-item"),
            "media",
            vec![
                particle(
                    ".product__media-item",
                    "",
                    vec![Attr::pick_as("data-media-id", "id")],
                ),
                particle(
                    "modal-opener img",
                    "",
                    vec![Attr::standard("src"), Attr::standard("alt")],
                ),
            ],
        ),
    ])
}

pub fn normalize(json: &str) -> Result<bodul_shared::NormalizedData, String> {
    let raw: raw::UgreenJpRawDocument = serde_json::from_str(json).map_err(|e| e.to_string())?;

    let normalized = UgreenJpNormalizedDocument::from_raw(raw)
        .map_err(|error: NormalizeError| error.to_string())?;

    let document_json = serde_json::to_string(&normalized).map_err(|e| e.to_string())?;

    Ok(bodul_shared::NormalizedData::just_document_json(
        document_json,
    ))
}

pub fn structuralize(json: &str) -> Result<StructuralizedData, String> {
    let normalized: UgreenJpNormalizedDocument =
        serde_json::from_str(json).map_err(|e| e.to_string())?;

    let document =
        UgreenJpDocument::from_normalized(normalized).map_err(|error| error.to_string())?;

    let document_json = serde_json::to_string(&document).map_err(|e| e.to_string())?;

    Ok(StructuralizedData::just_document_json(document_json))
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;
    use std::path::{Path, PathBuf};

    use html_extractor::Backend;
    use html_extractor::serde_json::Value;

    use super::{architecture, normalize, structuralize};

    /// Saved product pages, kept outside the repository next to the monorepo checkout.
    const DATA_DIR: &str = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../../data/ugreenjp/offer_details/sources"
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

    /// The variant ids of a list of variant records.
    fn ids(variants: &[Value]) -> BTreeSet<u64> {
        variants
            .iter()
            .map(|variant| variant["id"].as_u64().expect("variant id"))
            .collect()
    }

    #[test]
    fn extracts_paypal_ld_json_analytics_and_product_section_from_every_page() {
        let Some(pages) = pages() else { return };
        let arch = architecture();
        for page in pages {
            let name = page.display();
            let json = Backend::Scraper.extract(&read(&page), &arch).unwrap().json;

            assert_eq!(json["paypal"]["data-currency"], "JPY", "{name}");

            let ld_json = json["ld-json"]
                .as_array()
                .unwrap_or_else(|| panic!("{name}: no ld-json"));
            let types: Vec<&Value> = ld_json
                .iter()
                .map(|entry| &entry["data"]["@type"])
                .collect();
            assert_eq!(types, ["Organization", "Product"], "{name}");
            let product = &ld_json[1]["data"];
            assert!(
                product["name"].as_str().is_some_and(|n| !n.is_empty()),
                "{name}: no name"
            );
            let offers = product["offers"]
                .as_array()
                .unwrap_or_else(|| panic!("{name}: no offers"));
            let offer_ids: BTreeSet<u64> = offers
                .iter()
                .map(|offer| {
                    assert_eq!(offer["priceCurrency"], "JPY", "{name}");
                    assert!(offer["price"].is_f64(), "{name}: price {}", offer["price"]);
                    let url = offer["url"].as_str().unwrap_or_default();
                    let (_, id) = url
                        .split_once("?variant=")
                        .unwrap_or_else(|| panic!("{name}: offer url {url}"));
                    id.parse().unwrap_or_else(|e| panic!("{name}: {url}: {e}"))
                })
                .collect();

            let meta = &json["analytics"]["meta"]["product"];
            assert!(meta["id"].is_u64(), "{name}: product id {}", meta["id"]);
            assert!(
                meta["handle"].as_str().is_some_and(|h| !h.is_empty()),
                "{name}: no handle"
            );
            let variants = meta["variants"]
                .as_array()
                .unwrap_or_else(|| panic!("{name}: no variants"));
            assert!(!variants.is_empty(), "{name}: no variants");
            for variant in variants {
                // Shopify's minor units even for JPY, which has none: always whole yen × 100.
                let price = variant["price"].as_i64();
                assert!(
                    price.is_some_and(|p| p % 100 == 0),
                    "{name}: price {}",
                    variant["price"]
                );
            }
            // ld+json and analytics both carry every variant.
            assert_eq!(ids(variants), offer_ids, "{name}");

            // The picker's JSON is missing exactly on `Default Title` products.
            let default_title = variants.len() == 1 && variants[0]["public_title"].is_null();
            match json["variants"]["data"].as_array() {
                Some(picker) => {
                    assert!(!default_title, "{name}: picker on a Default Title product");
                    assert_eq!(ids(picker), offer_ids, "{name}");
                }
                None => assert!(default_title, "{name}: no picker JSON"),
            }

            let selected: u64 = json["selected_variant_id"]["value"]
                .as_str()
                .and_then(|id| id.parse().ok())
                .unwrap_or_else(|| panic!("{name}: no selected variant"));
            assert!(offer_ids.contains(&selected), "{name}: selected {selected}");

            let price = &json["price"];
            let class: Vec<&str> = price["class"]
                .as_str()
                .unwrap_or_default()
                .split_whitespace()
                .collect();
            assert!(class.contains(&"price--product"), "{name}: {class:?}");
            for key in ["regular", "sale"] {
                assert!(
                    price[key]["text"]
                        .as_str()
                        .is_some_and(|t| t.starts_with('¥')),
                    "{name}: price {key} {}",
                    price[key]
                );
            }
            // A compare-at price only under `price--on-sale`; otherwise the `<s>` is empty,
            // repeats the price or reads `¥0`.
            let sale = &price["sale"]["text"];
            let compare_at = &price["compare_at"]["text"];
            if class.contains(&"price--on-sale") {
                assert!(
                    compare_at.as_str().is_some_and(|t| t.starts_with('¥')) && compare_at != sale,
                    "{name}: compare-at {compare_at} on sale at {sale}"
                );
            } else {
                assert!(
                    compare_at == "" || compare_at == sale || compare_at == "¥0",
                    "{name}: compare-at {compare_at} off sale at {sale}"
                );
            }

            let media = json["media"]
                .as_array()
                .unwrap_or_else(|| panic!("{name}: no media"));
            assert!(!media.is_empty(), "{name}: no media");
            for slide in media {
                assert!(
                    slide["id"].as_str().is_some_and(|id| !id.is_empty()),
                    "{name}: media id {}",
                    slide["id"]
                );
                assert!(
                    slide["src"].as_str().is_some_and(|src| !src.is_empty()),
                    "{name}: media src {}",
                    slide["src"]
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
