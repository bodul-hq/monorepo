//! Version 1 of the offer details.
//!
//! `fr.ugreen.com` is a Shopify Liquid storefront on the same theme as `ugreende`
//! (`lion/main-fr`, Horizon 2.1.6, with the `ug_main_product` section). Surveyed on 306 saved
//! pages, all of them product pages of this store (shop currency EUR).
//!
//! Unlike `ugreende`, the product ld+json is read: every block on every page parses, prices use
//! a decimal point (`"9.59"`) and agree with the analytics prices for all 598 variants. The risk
//! `ugreende` hit (hand-written islands with `"price": 43,99`) has not shown up here; if it does,
//! the whole extraction fails with `InvalidJson`, so it can't pass unnoticed.
//!
//! Where the facts are:
//! - Title: the theme's ld+json `name` (equal to the main section's `h1` on every page).
//! - Variants: the theme's ld+json, a `Product` with one `offers` (262 pages) or a
//!   `ProductGroup` with `hasVariant` (44 pages, 12 of them with a single variant). Its variants
//!   match the analytics `meta` variants on every page; a variant's id is only in its offer URL
//!   (`?variant=`), the analytics carries it as a number.
//! - SKU: ld+json `sku` and analytics `sku`, missing (`null` / `""`) only on the three gift
//!   cards (`ugreen-carte-cadeau-*`).
//! - GTIN: ld+json `gtin` on 139 pages for every variant, on 11 for some, on 156 for none. Six
//!   values on 16 variants end in a no-break space (`"6938624439712\u{a0}"`); trim them.
//! - Price and currency: ld+json `offers.price` (a decimal string) and `priceCurrency` (EUR on
//!   every offer), the Shopify variant price; analytics `price` in cents. See `price` below for
//!   the price the theme shows.
//! - Compare-at: only for the selected variant, in the theme's `product-price` and, on a
//!   product with a picker, the picker's variant JSON.
//! - Availability: ld+json `offers.availability` per variant (the add-to-cart button's
//!   `disabled` agrees on every page).
//! - Images: the main gallery's image slides; ld+json carries one `image` on a `Product` and one
//!   per variant on a `ProductGroup`.
//!
//! No island this library can read carries every variant's compare-at price. The `product |
//! json` records that do (the Lai Reviews app's `dataShopFromLiquidLAI.productMainInfo['product']`
//! and the automatic-discount app's `window.aiod_product_data`) are a property assignment and a
//! JavaScript object literal, which `Attr::javascript_vars` can't read. The theme's
//! `script[data-ug-gallery-discount-variants]` maps every variant id to a rendered discount badge
//! (a rounded percentage, empty when there is none), not a price, so it isn't read either. There
//! is no Apple Pay shop-capabilities script on any page.

use crate::ugreenfr::offer_details::v1::normalized::{NormalizeError, UgreenFrNormalizedDocument};
use html_extractor::io::{Architecture, Attr, collection, particle, trash};
use html_extractor::serde_json;
use retailer_sauce::structuralized::StructuralizedData;
use structuralize::UgreenFrDocument;

pub mod normalized;
pub mod raw;
mod structuralize;

/// The main product section. The page's other sections render recommendations with their own
/// prices and forms.
const MAIN: &str = r#"[id^="shopify-section-template--"][id*="__ug_main_product_"]"#;

/// The main product form; the sticky bar inside the section renders a second one.
const FORM: &str = r#"product-form-component:not([data-section-id$="-sticky-bar"])"#;

fn main(selector: &str) -> String {
    format!("{MAIN} {selector}")
}

pub fn architecture() -> Architecture {
    Architecture::new(vec![
        // Must run first: `trash` detaches its match, and svg elements sit inside body, so
        // anything claiming body ahead of this would carry them off first.
        trash(r#"svg"#),
        // On every page: the `Organization`, the theme's `Product` or `ProductGroup` (no class),
        // and on 29 pages Judge.me's review `Product` (`class="jdgm-server-jld"`, no offers).
        // `Attr::all()` alongside the JSON read: a stray attribute appearing on the tag fails the
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
        // Shopify's analytics `meta` sits in an anonymous inline script. The usual anchor,
        // `script:has(+ script.analytics)`, works only on the Scraper backend: the Kuchiki
        // backends reject `:has` as an invalid selector. So it is anchored forwards from the
        // monorail prefetch link, past the session-beacon, Trekkie-shim and web-pixels-loader
        // scripts, a sequence identical on all 306 saved pages. Being positional, this breaks if
        // Shopify adds or drops a script between the two; the strict raw schema then reports a
        // missing `analytics` or `meta`. `Attr::all()` adds nothing until the script gains an
        // attribute.
        particle(
            concat!(
                r#"link[rel="dns-prefetch"][href="https://monorail-edge.shopifysvc.com"]"#,
                " + script + script + script + script"
            ),
            "analytics",
            vec![Attr::all(), Attr::javascript_vars("meta", "meta")],
        ),
        // The selected variant's prices in cents, and the shop currency. On 266 pages
        // `base_price_cents` is the Shopify variant price and `base_compare_cents` its
        // compare-at price (equal to the price when there is none; 249 pages show a higher one).
        // On the 40 pages whose form carries `data-ug-auto-discount` (see `product_form`),
        // `base_price_cents` is the price after the product's automatic discount code, below the
        // ld+json and analytics price, and `base_compare_cents` is the Shopify compare-at price.
        // Which one is the offer is for `normalize` to decide. Read by name rather than with
        // `Attr::all()`: the tag also carries a block id and inline style.
        particle(
            &main("product-price"),
            "price",
            vec![
                Attr::pick_as("data-product-id", "product_id"),
                Attr::pick_as("data-base-price-cents", "base_price_cents"),
                Attr::pick_as("data-base-compare-cents", "base_compare_cents"),
                Attr::pick_as("data-currency", "currency"),
            ],
        ),
        // `auto_discount` (`"true"`) and `discount_code` (`DL<sku>`) are present on exactly the
        // 40 pages whose `price` is discounted, and missing on the other 266.
        particle(
            &main(FORM),
            "product_form",
            vec![
                Attr::pick_as("data-product-id", "product_id"),
                Attr::pick_as("data-product-url", "product_url"),
                Attr::pick_as("data-ug-auto-discount", "auto_discount"),
                Attr::pick_as("data-ug-discount-code", "discount_code"),
            ],
        ),
        // The variant `price` describes: the first variant on 303 pages, another one on 3.
        particle(
            &main(&format!(r#"{FORM} input[name="id"]"#)),
            "selected_variant_id",
            vec![Attr::standard("value")],
        ),
        // The picker renders only on the 44 `ProductGroup` pages; on the others this is empty and
        // `selected_variant` is missing. The legend's title only; its other span repeats the
        // selected value.
        collection(
            &main("variant-picker fieldset legend > span.variant-option__legend-title"),
            "option_names",
            vec![particle("span", "", vec![Attr::inner_text("text")])],
        ),
        // The selected variant's `product | json` record, with `compare_at_price`, `barcode`
        // and `options`.
        particle(
            &main(r#"variant-picker script[type="application/json"]"#),
            "selected_variant",
            vec![Attr::all(), Attr::json("", "data")],
        ),
        // Image slides only: a video slide (on 6 pages) renders its poster thumbnail twice.
        // `src` is protocol-relative (`//fr.ugreen.com/cdn/shop/…`).
        collection(
            &main("media-gallery slideshow-slide.product-media-container--image img"),
            "images",
            vec![particle(
                "img",
                "",
                vec![Attr::standard("src"), Attr::standard("alt")],
            )],
        ),
    ])
}

pub fn normalize(json: &str) -> Result<bodul_shared::NormalizedData, String> {
    let raw: raw::UgreenFrRawDocument = serde_json::from_str(json).map_err(|e| e.to_string())?;

    let normalized = UgreenFrNormalizedDocument::from_raw(raw)
        .map_err(|error: NormalizeError| error.to_string())?;

    let document_json = serde_json::to_string(&normalized).map_err(|e| e.to_string())?;

    Ok(bodul_shared::NormalizedData::just_document_json(
        document_json,
    ))
}

pub fn structuralize(json: &str) -> Result<StructuralizedData, String> {
    let normalized: UgreenFrNormalizedDocument =
        serde_json::from_str(json).map_err(|e| e.to_string())?;

    let document =
        UgreenFrDocument::from_normalized(normalized).map_err(|error| error.to_string())?;

    let document_json = serde_json::to_string(&document).map_err(|e| e.to_string())?;

    Ok(StructuralizedData::just_document_json(document_json))
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use html_extractor::Backend;
    use html_extractor::serde_json::Value;

    use super::{architecture, normalize, structuralize};

    /// Saved product pages, kept outside the repository next to the monorepo checkout.
    const DATA_DIR: &str = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../../data/ugreenfr/offer_details/sources"
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

    /// A string attribute parsed as an integer.
    fn int(value: &Value) -> Option<i64> {
        value.as_str().and_then(|s| s.parse().ok())
    }

    /// The variant id at the end of an ld+json offer URL.
    fn variant_id(offer: &Value) -> Option<i64> {
        offer["url"]
            .as_str()
            .and_then(|url| url.rsplit_once("?variant="))
            .and_then(|(_, id)| id.parse().ok())
    }

    #[test]
    fn extracts_ld_json_analytics_and_main_section_from_every_page() {
        let Some(pages) = pages() else { return };
        let arch = architecture();
        for page in pages {
            let name = page.display();
            let json = Backend::Scraper.extract(&read(&page), &arch).unwrap().json;

            let ld_json = json["ld-json"]
                .as_array()
                .unwrap_or_else(|| panic!("{name}: no ld-json"));
            assert!(
                ld_json
                    .iter()
                    .any(|entry| entry["data"]["@type"] == "Organization"),
                "{name}: no Organization island"
            );
            // The theme's island carries no class; Judge.me's review `Product` does.
            let products: Vec<&Value> = ld_json
                .iter()
                .filter(|entry| entry.get("class").is_none())
                .map(|entry| &entry["data"])
                .filter(|data| data["@type"] == "Product" || data["@type"] == "ProductGroup")
                .collect();
            assert_eq!(products.len(), 1, "{name}: theme product islands");
            let product = products[0];
            assert!(
                product["name"].as_str().is_some_and(|t| !t.is_empty()),
                "{name}: no title"
            );
            let variants: Vec<&Value> = match product["hasVariant"].as_array() {
                Some(variants) => variants.iter().collect(),
                None => vec![product],
            };
            assert!(!variants.is_empty(), "{name}: no variants");

            let meta = &json["analytics"]["meta"]["product"];
            let meta_variants = meta["variants"]
                .as_array()
                .unwrap_or_else(|| panic!("{name}: no analytics variants"));
            assert_eq!(variants.len(), meta_variants.len(), "{name}: variant count");
            let handle = meta["handle"]
                .as_str()
                .unwrap_or_else(|| panic!("{name}: no handle"));
            let gift_card = handle.starts_with("ugreen-carte-cadeau-");

            for (variant, meta_variant) in variants.iter().zip(meta_variants) {
                let offer = &variant["offers"];
                assert_eq!(offer["priceCurrency"], "EUR", "{name}");
                let price: f64 = offer["price"]
                    .as_str()
                    .and_then(|p| p.parse().ok())
                    .unwrap_or_else(|| panic!("{name}: price {}", offer["price"]));
                assert_eq!(
                    Some((price * 100.0).round() as i64),
                    meta_variant["price"].as_i64(),
                    "{name}: ld+json and analytics prices"
                );
                assert_eq!(
                    variant_id(offer),
                    meta_variant["id"].as_i64(),
                    "{name}: variant id"
                );
                assert!(
                    offer["availability"]
                        .as_str()
                        .is_some_and(|a| a.starts_with("http://schema.org/")),
                    "{name}: availability {}",
                    offer["availability"]
                );
                // Only the gift cards have no SKU.
                assert_eq!(
                    variant["sku"].as_str().is_some_and(|s| !s.is_empty()),
                    !gift_card,
                    "{name}: sku {}",
                    variant["sku"]
                );
            }

            let price = &json["price"];
            assert_eq!(price["currency"], "EUR", "{name}");
            assert_eq!(
                int(&price["product_id"]),
                meta["id"].as_i64(),
                "{name}: product id"
            );
            let selected = int(&json["selected_variant_id"]["value"])
                .unwrap_or_else(|| panic!("{name}: no selected variant id"));
            let selected_price = meta_variants
                .iter()
                .find(|v| v["id"].as_i64() == Some(selected))
                .and_then(|v| v["price"].as_i64())
                .unwrap_or_else(|| panic!("{name}: selected variant {selected} not in analytics"));
            let base_price =
                int(&price["base_price_cents"]).unwrap_or_else(|| panic!("{name}: no base price"));
            assert!(
                int(&price["base_compare_cents"]).is_some(),
                "{name}: no base compare price"
            );
            // The theme shows the automatic-discount price exactly where the form says so.
            let form = &json["product_form"];
            assert_eq!(int(&form["product_id"]), meta["id"].as_i64(), "{name}");
            if form["auto_discount"] == "true" {
                assert!(
                    form["discount_code"].is_string(),
                    "{name}: no discount code"
                );
                assert!(base_price < selected_price, "{name}: discounted price");
            } else {
                assert!(form.get("auto_discount").is_none(), "{name}");
                assert_eq!(base_price, selected_price, "{name}: price");
            }

            // The picker, with its option names and selected variant, renders exactly on the
            // `ProductGroup` pages.
            let option_names = json["option_names"]
                .as_array()
                .unwrap_or_else(|| panic!("{name}: no option_names"));
            if product["@type"] == "ProductGroup" {
                assert!(!option_names.is_empty(), "{name}: no option names");
                assert_eq!(
                    json["selected_variant"]["data"]["id"].as_i64(),
                    Some(selected),
                    "{name}: selected variant"
                );
            } else {
                assert!(option_names.is_empty(), "{name}: option names");
                assert!(json.get("selected_variant").is_none(), "{name}");
            }

            let images = json["images"]
                .as_array()
                .unwrap_or_else(|| panic!("{name}: no images"));
            assert!(!images.is_empty(), "{name}: no images");
            for image in images {
                assert!(
                    image["src"].as_str().is_some_and(|s| s.starts_with("//")),
                    "{name}: image {}",
                    image["src"]
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
