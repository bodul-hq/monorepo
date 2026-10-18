//! Version 1 of the offer details.
//!
//! `eu.ugreen.com` is a Shopify Liquid storefront on the same `ug_main_product` theme as
//! `ugreende`. One shop serves every EU market under locale paths: language only (`/de/`, `/sv/`,
//! …), language plus country (`/de-at/`, `/en-se/`, …), and the unprefixed English root. Each
//! saved page's file name carries that path prefix, and its canonical URL is on `eu.ugreen.com`
//! under the same prefix (12,487 pages, 66 prefixes, none from another storefront).
//!
//! Language follows the path (`Shopify.locale`, `<html lang>`): titles and descriptions are
//! translated per prefix. Currency does not: every page in every market, including CH, DK, SE,
//! PL, HU, CZ, RO and BG, prices in EUR, and every currency statement on the page (ld+json
//! `priceCurrency`, `product-price[data-currency]`, `Shopify.currency.active`, the analytics
//! currency and the PayPal meta tag) says so. Language-only prefixes and the root render as
//! market `DE`.
//!
//! Every saved page is a product page and carries the islands read below, each once, except
//! where a structure's comment says otherwise. The ld+json parses as JSON on every page: it is
//! Shopify's machine-generated `structured_data`, with dot decimals in string prices
//! (`"10.99"`). The hand-written island with German decimals that `ugreende` avoids does not
//! appear here.
//!
//! Where the facts are:
//!
//! - title, SKU, GTIN, price, currency, availability, variants, per-variant image: the product
//!   ld+json. A single-variant product is a `Product` with one `offers` (10,639 pages); a
//!   multi-variant one is a `ProductGroup` with `hasVariant` (1,848 pages, up to 100 variants).
//!   `gtin` is missing on some variants (2,875 pages carry none).
//! - Shopify product id, variant ids, handle, vendor, type, prices in cents: the analytics
//!   `meta`. Its variants match the ld+json's one for one, in the same order, with equal SKUs
//!   and prices, on every page.
//! - Displayed price and list price: `product-price`, for the selected variant only. See its
//!   structure for how they differ from the Shopify variant price.
//! - Compare-at price and barcode of the selected variant: the variant picker's JSON, on
//!   multi-variant pages only.
//! - Images: the main gallery.
//!
//! No island on the page that this library can read carries every variant's compare-at price.
//! The `product | json` records that do (the reviews app's `productMainInfo['product']`, the
//! Klarna block's `productVariants:` property, `_ReStockConfig.product` and
//! `window.aiod_product_data`) are object properties or JavaScript literals rather than JSON
//! assigned to a variable, so `Attr::javascript_vars` cannot reach them. There is no Apple Pay
//! shop-capabilities script on any page.

use crate::ugreeneu::offer_details::v1::normalized::{NormalizeError, UgreenEuNormalizedDocument};
use html_extractor::io::{Architecture, Attr, collection, particle, trash};
use html_extractor::serde_json;
use retailer_sauce::structuralized::StructuralizedData;
use structuralize::UgreenEuDocument;

pub mod normalized;
pub mod raw;
mod structuralize;

/// The main product section. The page's other sections render recommendations, mega menus and
/// a sticky bar with their own prices and forms.
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
        // Two islands on every page: the shop's `Organization`, then the product as `Product` or
        // `ProductGroup`. `Attr::all()` alongside the JSON read: a stray attribute appearing on
        // the tag fails the strict raw schema instead of disappearing silently.
        collection(
            r#"script[type="application/ld+json"]"#,
            "ld-json",
            vec![particle(
                "script",
                "",
                vec![Attr::all(), Attr::json("", "data")],
            )],
        ),
        // Shopify's analytics `meta` sits in an anonymous inline script with no attributes of its
        // own, so it is anchored forwards from the one Shopify-rendered tag four siblings before
        // it in `head`: the monorail `dns-prefetch` link, followed by three other anonymous
        // scripts. That holds on every saved page. `script:has(+ script.analytics)`, which
        // anchors on the script that follows it, only parses on `Backend::Scraper`; the Kuchiki
        // backends reject `:has` as an invalid selector. Being positional, this anchor breaks
        // if Shopify adds or drops a script between the two; the strict raw schema then reports
        // a missing `analytics`. `Attr::all()` adds nothing until the script gains an attribute.
        particle(
            r#"link[rel="dns-prefetch"][href="https://monorail-edge.shopifysvc.com"] + script + script + script + script"#,
            "analytics",
            vec![Attr::all(), Attr::javascript_vars("meta", "meta")],
        ),
        // The selected variant's prices in cents. `base_price_cents` is the price the theme
        // shows, after the shop's automatic discount: on 2,237 pages it is below the Shopify
        // variant price in ld+json and analytics. `base_compare_cents` is the variant's
        // compare-at price, or its Shopify price when it has no compare-at price or is
        // discounted, so it is never below `base_price_cents`.
        //
        // Named attributes rather than `Attr::all()`: the tag also carries `class`, `style` and a
        // theme block id, which change with the theme's layout rather than with the offer.
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
        // The variant `product-price` describes: the first available one, or the first variant.
        // Always one of the analytics variant ids.
        particle(
            &main(&format!(r#"{FORM} input[name="id"]"#)),
            "selected_variant_id",
            vec![Attr::standard("value")],
        ),
        // The selected variant's Liquid record (`compare_at_price`, `barcode`, `available`, …).
        // Only multi-variant products render the picker, so this is missing on the 10,639
        // single-variant pages and present on all 1,848 others.
        particle(
            &main(r#"variant-picker script[type="application/json"]"#),
            "selected_variant",
            vec![Attr::all(), Attr::json("", "data")],
        ),
        // Image slides only: a video slide (331 pages) renders its poster thumbnail. Every page
        // has at least one image slide, each with one `img` and a non-empty `src`.
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
    let raw: raw::UgreenEuRawDocument = serde_json::from_str(json).map_err(|e| e.to_string())?;

    let normalized = UgreenEuNormalizedDocument::from_raw(raw)
        .map_err(|error: NormalizeError| error.to_string())?;

    let document_json = serde_json::to_string(&normalized).map_err(|e| e.to_string())?;

    Ok(bodul_shared::NormalizedData::just_document_json(
        document_json,
    ))
}

pub fn structuralize(json: &str) -> Result<StructuralizedData, String> {
    let normalized: UgreenEuNormalizedDocument =
        serde_json::from_str(json).map_err(|e| e.to_string())?;

    let document =
        UgreenEuDocument::from_normalized(normalized).map_err(|error| error.to_string())?;

    let document_json = serde_json::to_string(&document).map_err(|e| e.to_string())?;

    Ok(StructuralizedData::just_document_json(document_json))
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};
    use std::sync::Mutex;
    use std::thread;

    use html_extractor::Backend;
    use html_extractor::io::Architecture;
    use html_extractor::serde_json::Value;

    use super::{architecture, normalize, structuralize};

    /// Saved product pages, kept outside the repository next to the monorepo checkout.
    const DATA_DIR: &str = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../../data/ugreeneu/offer_details/sources"
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

    /// Runs `check` on every page across all cores, since the corpus is 12k pages of about
    /// 1.8 MB each, and returns every failure it reports, so one run shows all of them.
    fn failures_over(
        pages: &[PathBuf],
        check: impl Fn(&Path) -> Vec<String> + Sync,
    ) -> Vec<String> {
        let workers = thread::available_parallelism().map_or(4, |n| n.get());
        let failures = Mutex::new(Vec::new());
        thread::scope(|scope| {
            for chunk in pages.chunks(pages.len().div_ceil(workers)) {
                let (check, failures) = (&check, &failures);
                scope.spawn(move || {
                    for page in chunk {
                        let found = check(page);
                        if !found.is_empty() {
                            failures.lock().expect("failures lock").extend(found);
                        }
                    }
                });
            }
        });
        let mut failures = failures.into_inner().expect("failures lock");
        failures.sort();
        failures
    }

    fn assert_none(failures: Vec<String>) {
        assert!(
            failures.is_empty(),
            "{} failures:\n{}",
            failures.len(),
            failures.join("\n")
        );
    }

    fn is_cents(value: &Value) -> bool {
        value.as_str().is_some_and(|v| v.parse::<u64>().is_ok())
    }

    fn non_empty(value: &Value) -> bool {
        value.as_str().is_some_and(|v| !v.is_empty())
    }

    /// The variants of the product ld+json with their offers: `hasVariant[]` for a
    /// `ProductGroup`, the product itself for a single `Product`.
    fn ld_variants(product: &Value) -> Vec<(&Value, &Value)> {
        match product["@type"].as_str() {
            Some("ProductGroup") => product["hasVariant"]
                .as_array()
                .map(|variants| variants.iter().map(|v| (v, &v["offers"])).collect())
                .unwrap_or_default(),
            _ => vec![(product, &product["offers"])],
        }
    }

    /// Every island the architecture reads, and the facts normalization needs from it.
    fn check_page(page: &Path, arch: &Architecture) -> Vec<String> {
        let json = match Backend::Scraper.extract(&read(page), arch) {
            Ok(result) => result.json,
            Err(e) => return vec![format!("{}: extract: {e}", page.display())],
        };
        let mut failures = Vec::new();
        macro_rules! fail {
            ($($arg:tt)*) => {
                failures.push(format!("{}: {}", page.display(), format!($($arg)*)))
            };
        }

        // ld+json: the shop's Organization and exactly one product island.
        let ld_json = json["ld-json"].as_array().cloned().unwrap_or_default();
        if ld_json.len() != 2 {
            fail!("{} ld-json islands", ld_json.len());
        }
        for entry in &ld_json {
            if entry["type"] != "application/ld+json"
                || entry.as_object().map(|e| e.len()) != Some(2)
            {
                fail!(
                    "ld-json attributes {:?}",
                    entry.as_object().map(|e| e.keys().collect::<Vec<_>>())
                );
            }
        }
        if !ld_json
            .iter()
            .any(|entry| entry["data"]["@type"] == "Organization")
        {
            fail!("no Organization island");
        }
        let products: Vec<&Value> = ld_json
            .iter()
            .map(|entry| &entry["data"])
            .filter(|data| matches!(data["@type"].as_str(), Some("Product" | "ProductGroup")))
            .collect();
        let [product] = products.as_slice() else {
            fail!("{} product islands", products.len());
            return failures;
        };
        if !non_empty(&product["name"]) {
            fail!("no product name");
        }
        let variants = ld_variants(product);
        if variants.is_empty() {
            fail!("no ld+json variants");
        }
        for (variant, offer) in &variants {
            if !non_empty(&variant["sku"]) {
                fail!("ld+json sku {}", variant["sku"]);
            }
            if !offer["price"]
                .as_str()
                .is_some_and(|p| p.parse::<f64>().is_ok())
            {
                fail!("ld+json price {}", offer["price"]);
            }
            if offer["priceCurrency"] != "EUR" {
                fail!("ld+json currency {}", offer["priceCurrency"]);
            }
            if !offer["availability"]
                .as_str()
                .is_some_and(|a| a.starts_with("http://schema.org/"))
            {
                fail!("ld+json availability {}", offer["availability"]);
            }
        }

        // Analytics meta: the product id and one variant per ld+json variant.
        let analytics = &json["analytics"];
        if analytics.as_object().map(|a| a.len()) != Some(1) {
            fail!(
                "analytics keys {:?}",
                analytics.as_object().map(|a| a.keys().collect::<Vec<_>>())
            );
        }
        let meta_product = &analytics["meta"]["product"];
        let product_id = meta_product["id"].as_u64();
        if product_id.is_none() {
            fail!("analytics product id {}", meta_product["id"]);
        }
        let meta_variants = meta_product["variants"]
            .as_array()
            .cloned()
            .unwrap_or_default();
        if meta_variants.len() != variants.len() {
            fail!(
                "{} analytics variants for {} ld+json variants",
                meta_variants.len(),
                variants.len()
            );
        }
        for variant in &meta_variants {
            if !variant["id"].is_u64() || !variant["price"].is_u64() || !non_empty(&variant["sku"])
            {
                fail!("analytics variant {variant}");
            }
        }
        let variant_ids: Vec<String> = meta_variants
            .iter()
            .filter_map(|v| v["id"].as_u64())
            .map(|id| id.to_string())
            .collect();

        // The selected variant's displayed and list prices.
        let price = &json["price"];
        if product_id.map(|id| id.to_string()).as_deref() != price["product_id"].as_str() {
            fail!("price product id {}", price["product_id"]);
        }
        if !is_cents(&price["base_price_cents"]) || !is_cents(&price["base_compare_cents"]) {
            fail!("price cents {price}");
        }
        if price["currency"] != "EUR" {
            fail!("price currency {}", price["currency"]);
        }
        let selected = json["selected_variant_id"]["value"].as_str();
        if !selected.is_some_and(|id| variant_ids.iter().any(|v| v == id)) {
            fail!("selected variant {selected:?} not among {variant_ids:?}");
        }

        // The picker's record of the selected variant, on multi-variant products only.
        let record = &json["selected_variant"];
        if product["@type"] == "ProductGroup" {
            if record["type"] != "application/json"
                || record.as_object().map(|r| r.len()) != Some(2)
            {
                fail!(
                    "selected_variant attributes {:?}",
                    record.as_object().map(|r| r.keys().collect::<Vec<_>>())
                );
            }
            let data = &record["data"];
            if data["id"].as_u64().map(|id| id.to_string()).as_deref() != selected {
                fail!("selected_variant id {}", data["id"]);
            }
            if !data["price"].is_u64() || data.get("compare_at_price").is_none() {
                fail!("selected_variant prices");
            }
        } else if !record.is_null() {
            fail!("selected_variant on a single-variant product");
        }

        // The gallery.
        let images = json["images"].as_array().cloned().unwrap_or_default();
        if images.is_empty() {
            fail!("no images");
        }
        for image in &images {
            if !non_empty(&image["src"]) {
                fail!("image {image}");
            }
        }

        failures
    }

    #[test]
    fn extracts_ld_json_analytics_price_and_gallery_from_every_page() {
        let Some(pages) = pages() else { return };
        let arch = architecture();
        assert_none(failures_over(&pages, |page| check_page(page, &arch)));
    }

    #[test]
    fn backends_agree_on_every_page() {
        let Some(pages) = pages() else { return };
        let arch = architecture();
        assert_none(failures_over(&pages, |page| {
            let html = read(page);
            let expected = Backend::Scraper.extract(&html, &arch).map(|r| r.json);
            Backend::ALL
                .iter()
                .filter(|backend| **backend != Backend::Scraper)
                .filter_map(|backend| {
                    let json = backend.extract(&html, &arch).map(|r| r.json);
                    match (json, &expected) {
                        (Ok(json), Ok(expected)) if json == *expected => None,
                        (json, expected) => Some(format!(
                            "{}: {backend:?} {} vs Scraper {}",
                            page.display(),
                            json.map_or_else(|e| e.to_string(), |_| "differs".to_string()),
                            expected
                                .as_ref()
                                .map_or_else(|e| e.to_string(), |_| "ok".to_string()),
                        )),
                    }
                })
                .collect()
        }));
    }

    #[test]
    fn normalizes_and_structuralizes_every_page() {
        let Some(pages) = pages() else { return };
        let arch = architecture();
        assert_none(failures_over(&pages, |page| {
            let pipeline = Backend::Scraper
                .extract(&read(page), &arch)
                .map_err(|e| format!("extract: {e}"))
                .and_then(|result| {
                    normalize(&result.json.to_string()).map_err(|e| format!("normalize: {e}"))
                })
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
            pipeline
                .err()
                .map(|e| format!("{}: {e}", page.display()))
                .into_iter()
                .collect()
        }));
    }
}
