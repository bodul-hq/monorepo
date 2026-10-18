//! Version 1 of the offer details.
//!
//! `www.ugreenindia.com` is a Shopify Liquid storefront on the Enterprise theme (2.3.0). Reads
//! the Open Graph price-currency tag (the shop currency), the schema.org islands, Shopify's
//! analytics `meta` and the Bitespeed app's `product | json`; the rendered DOM is not claimed.
//!
//! The store serves every product twice: under `/products/…` in English and under
//! `/hi/products/…` in Hindi, saved as `products-*.html` and `hi-products-*.html` (704 pages
//! each). Both are the same shop (`990f9c-b7.myshopify.com`, `Shopify.currency` INR) and render
//! the same islands on every page. Ids, SKUs, barcodes, prices, compare-at prices, availability
//! and images are identical between the two; what differs is the language of the title,
//! description, product type, option names and variant names, and the `/hi` prefix on every
//! URL the islands carry.
//!
//! The theme renders no `product | json` of its own and no Apple Pay shop capabilities; the
//! survey found these islands, each on all 1,408 pages:
//!
//! - `meta[property="og:price:currency"]`: `INR`, the one currency statement outside a script.
//! - ld+json: `Organization`, `BreadcrumbList` and the product, plus `FAQPage` (681 pages per
//!   locale) and Judge.me's review-only `Product` (`class="jdgm-server-jld"`, 246 per locale).
//!   Every block parses as JSON.
//! - Shopify's analytics `meta`: product id and every variant's id, price in paise, SKU and name.
//! - `script[name="BSPD-SHOPIFY-VARS"]`: `var BspdCurrentProduct = …`, Liquid's whole
//!   `product | json` with every variant.
//!
//! Where the facts are:
//!
//! - Title: `product_json.data.title`; the theme's ld+json `name` agrees on every page.
//! - SKU: `product_json.data.variants[].sku`, set on every variant; the analytics `meta` and the
//!   theme's ld+json (`sku`, or `hasVariant[].sku`) agree.
//! - GTIN: `product_json.data.variants[].barcode`. 13 digits on 449 pages per locale, 12 on one,
//!   `null` on 223, and on 30 pages per locale a spreadsheet-mangled `6.96E+12` or `6.94E+12`
//!   that is no GTIN. The ld+json `gtin` repeats the barcode, mangled values included.
//! - Price: `product_json.data.variants[].price`, integer paise; the analytics `meta` agrees,
//!   and the ld+json offer gives the same amount as a decimal string.
//! - Currency: `og_currency.content`, also the ld+json offers' `priceCurrency`.
//! - Compare-at price: `product_json.data.variants[].compare_at_price`, paise, above the price
//!   on 700 pages per locale and `null` on 4. No other island has it.
//! - Availability: `product_json.data.variants[].available`; the ld+json offers' `availability`
//!   agrees on every page.
//! - Variants: `product_json.data.variants`. 702 products per locale have one variant, two have
//!   3 and 6; those two and one single-variant product render a `ProductGroup` with
//!   `hasVariant` in ld+json instead of a `Product` with one `offers`.
//! - Images: `product_json.data.images` (protocol-relative URLs) and `media`, with
//!   `variants[].featured_image`; ld+json carries a single `image` on a `Product` only.

use crate::ugreenin::offer_details::v1::normalized::{NormalizeError, UgreenInNormalizedDocument};
use html_extractor::io::{Architecture, Attr, collection, particle, trash};
use html_extractor::serde_json;
use retailer_sauce::structuralized::StructuralizedData;
use structuralize::UgreenInDocument;

pub mod normalized;
pub mod raw;
mod structuralize;

pub fn architecture() -> Architecture {
    Architecture::new(vec![
        // Must run first: `trash` detaches its match, and svg elements sit inside body, so
        // anything claiming body ahead of this would carry them off first.
        trash(r#"svg"#),
        // The one statement of the shop currency outside a script on every page: Liquid's record
        // names none, the analytics script sets it in a string assignment `javascript_vars`
        // can't read, and a `ProductGroup` carries it only per variant.
        particle(
            r#"meta[property="og:price:currency"]"#,
            "og_currency",
            vec![Attr::all()],
        ),
        // `Attr::all()` alongside the JSON reads: a stray attribute appearing on a tag fails the
        // strict raw schema instead of disappearing silently. Two blocks can describe the product:
        // the theme's, and Judge.me's reviews (`class="jdgm-server-jld"`) with the same `@id`.
        collection(
            r#"script[type="application/ld+json"]"#,
            "ld-json",
            vec![particle(
                "script",
                "",
                vec![Attr::all(), Attr::json("", "data")],
            )],
        ),
        // Shopify's analytics `meta`: a cross-check on the app's record below from Shopify
        // itself, and the product's id, prices and SKUs should the app go away. Ids, prices and
        // SKUs agree with the app's record on every page.
        //
        // It sits in an anonymous inline script with no attributes. Other Shopify storefronts
        // anchor it as `script:has(+ script.analytics)`, but only the Scraper backend parses
        // `:has`; the Kuchiki backends reject the selector. So it is anchored forwards, as on
        // ugreencom: on all 1,408 pages it is the fourth script after Shopify's `monorail-edge`
        // dns-prefetch link, the start of the injected `content_for_header` block, and the one
        // right before `script.analytics`. If Shopify inserts a script there, this matches another
        // anonymous script, `meta` is not found and `analytics` comes out without it, which the
        // data-backed test catches.
        particle(
            concat!(
                r#"link[rel="dns-prefetch"][href="https://monorail-edge.shopifysvc.com"]"#,
                " + script + script + script + script"
            ),
            "analytics",
            vec![Attr::all(), Attr::javascript_vars("meta", "meta")],
        ),
        // Liquid's `product | json`, rendered by the Bitespeed app embed on every product page.
        // The back-in-stock helper renders it too, but as `_BISConfig.product = …`, a property
        // assignment `javascript_vars` doesn't locate; the other copies are commented out.
        particle(
            r#"script[name="BSPD-SHOPIFY-VARS"]"#,
            "product_json",
            vec![
                Attr::all(),
                Attr::javascript_vars("BspdCurrentProduct", "data"),
            ],
        ),
    ])
}

pub fn normalize(json: &str) -> Result<bodul_shared::NormalizedData, String> {
    let raw: raw::UgreenInRawDocument = serde_json::from_str(json).map_err(|e| e.to_string())?;

    let normalized = UgreenInNormalizedDocument::from_raw(raw)
        .map_err(|error: NormalizeError| error.to_string())?;

    let document_json = serde_json::to_string(&normalized).map_err(|e| e.to_string())?;

    Ok(bodul_shared::NormalizedData::just_document_json(
        document_json,
    ))
}

pub fn structuralize(json: &str) -> Result<StructuralizedData, String> {
    let normalized: UgreenInNormalizedDocument =
        serde_json::from_str(json).map_err(|e| e.to_string())?;

    let document =
        UgreenInDocument::from_normalized(normalized).map_err(|error| error.to_string())?;

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
        "/../../../data/ugreenin/offer_details/sources"
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

    /// The theme's product block: a `Product`, or a `ProductGroup` for some multi-variant
    /// products. Judge.me's review-only `Product` is told apart by its class.
    fn theme_product<'a>(ld_json: &'a [Value], name: &str) -> &'a Value {
        let products: Vec<&Value> = ld_json
            .iter()
            .filter(|entry| entry["class"].is_null())
            .map(|entry| &entry["data"])
            .filter(|data| data["@type"] == "Product" || data["@type"] == "ProductGroup")
            .collect();
        assert_eq!(products.len(), 1, "{name}: theme product blocks");
        products[0]
    }

    /// The offers of the theme's product block, one per variant.
    fn offers(product: &Value) -> Vec<&Value> {
        match product["hasVariant"].as_array() {
            Some(variants) => variants.iter().map(|variant| &variant["offers"]).collect(),
            None => vec![&product["offers"]],
        }
    }

    fn ids(variants: &Value) -> Vec<&Value> {
        variants
            .as_array()
            .map(|variants| variants.iter().map(|variant| &variant["id"]).collect())
            .unwrap_or_default()
    }

    #[test]
    fn extracts_currency_ld_json_analytics_and_product_json_from_every_page() {
        let Some(pages) = pages() else { return };
        let arch = architecture();
        for page in pages {
            let name = page.display().to_string();
            let json = Backend::Scraper.extract(&read(&page), &arch).unwrap().json;

            assert_eq!(json["og_currency"]["content"], "INR", "{name}");

            let ld_json = json["ld-json"]
                .as_array()
                .unwrap_or_else(|| panic!("{name}: no ld-json"));
            for entry in ld_json {
                assert!(entry["data"].is_object(), "{name}: empty ld-json block");
            }
            assert!(
                ld_json
                    .iter()
                    .any(|entry| entry["data"]["@type"] == "Organization"),
                "{name}: no Organization island"
            );
            let product = theme_product(ld_json, &name);
            let offers = offers(product);
            for offer in &offers {
                assert_eq!(offer["priceCurrency"], "INR", "{name}");
                assert!(
                    offer["price"].is_string(),
                    "{name}: price {}",
                    offer["price"]
                );
            }

            // The Hindi copy of the store prefixes every URL with `/hi`.
            let file = page
                .file_name()
                .and_then(|f| f.to_str())
                .unwrap_or_default();
            let base = if file.starts_with("hi-") {
                "https://www.ugreenindia.com/hi/products/"
            } else {
                "https://www.ugreenindia.com/products/"
            };
            assert!(
                product["url"]
                    .as_str()
                    .is_some_and(|url| url.starts_with(base)),
                "{name}: url {}",
                product["url"]
            );

            let record = &json["product_json"]["data"];
            assert!(record["id"].is_u64(), "{name}: id {}", record["id"]);
            assert!(
                record["title"].as_str().is_some_and(|t| !t.is_empty()),
                "{name}: no title"
            );
            assert_eq!(record["title"], product["name"], "{name}");
            assert!(
                record["handle"].as_str().is_some_and(|h| !h.is_empty()),
                "{name}: no handle"
            );
            let variants = record["variants"]
                .as_array()
                .unwrap_or_else(|| panic!("{name}: no variants"));
            assert!(!variants.is_empty(), "{name}: no variants");
            assert_eq!(variants.len(), offers.len(), "{name}: variants vs offers");
            for variant in variants {
                assert!(
                    variant["id"].is_u64(),
                    "{name}: variant id {}",
                    variant["id"]
                );
                assert!(
                    variant["sku"].as_str().is_some_and(|s| !s.is_empty()),
                    "{name}: sku {}",
                    variant["sku"]
                );
                assert!(
                    variant["price"].is_u64(),
                    "{name}: price {}",
                    variant["price"]
                );
                assert!(
                    variant["available"].is_boolean(),
                    "{name}: available {}",
                    variant["available"]
                );
            }

            let meta = &json["analytics"]["meta"]["product"];
            assert_eq!(meta["id"], record["id"], "{name}: analytics product id");
            assert_eq!(
                ids(&meta["variants"]),
                ids(&record["variants"]),
                "{name}: analytics variant ids"
            );
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
