//! Version 1 of the offer details.
//!
//! `www.ugreen.com` is UGREEN's global brand site: a Shopify Liquid storefront on a customised
//! Horizon 2.1.6 theme (`whale/main`) whose shop sits in Hong Kong (`ugreen-brandsite`, settled
//! in HKD) and presents prices in USD. It does not sell: the product section renders no price, no
//! variant picker and no cart form, only a "Shop Now" button that links out to a regional store
//! (`us.ugreen.com`, `ai.ugreen.com`).
//!
//! Survey of the 66 saved pages (all `products-*`):
//!
//! - Every page carries exactly one ld+json block, and it is the shop's `Organization`. No page
//!   has a `Product` island. Every block parses as JSON.
//! - No page has Apple Pay shop capabilities, a `product | json` script, a theme product-JSON
//!   script or a variant picker.
//! - Shopify's analytics `meta` is on every page and is the only record of the product: id,
//!   handle, vendor and one variant (id, price in cents, name, SKU). Every product has exactly
//!   one variant, with `public_title: null`, so its `name` is the product title (equal to
//!   `og:title` and to the rendered `h1` on every page).
//! - The analytics price is a placeholder, not an offer: 65 pages list `130000` (USD 1,300.00)
//!   for every product, hubs and NAS alike, and the remaining one (`us-85185`, vendor
//!   `UGREEN Official Site`) lists `500`. `og:price:amount` and the Klaviyo script repeat it;
//!   Klaviyo's compare-at price is `$0.00` on every page.
//! - Not on any page: a barcode or GTIN, a compare-at price and a stock status. The only
//!   availability is the `"available": true` that trekkie's `Viewed Product` call passes as a
//!   function argument, which no structure can read.
//!
//! Where the facts are:
//!
//! | Fact          | Structure and key                                                       |
//! | ------------- | ----------------------------------------------------------------------- |
//! | Product id    | `analytics.meta.product.id` (and `.handle`)                             |
//! | Title         | `og_title.content`; `analytics.meta.product.variants[].name` agrees     |
//! | SKU           | `analytics.meta.product.variants[].sku`; `shop_now.data-product-sku`    |
//! | GTIN, barcode | none                                                                    |
//! | Price         | `analytics.meta.product.variants[].price`, cents, a placeholder         |
//! | Currency      | `paypal.data-currency` (`USD`); the analytics `meta` carries none       |
//! | Compare-at    | none                                                                    |
//! | Availability  | none                                                                    |
//! | Variants      | `analytics.meta.product.variants`, always one                           |
//! | Images        | `media[]`: `src`, `alt`, `id`, `media_type`                             |
//! | Canonical URL | `canonical.href`                                                        |
//! | Sales channel | `shop_now.channels.channel_url_list`, missing on one page               |

use crate::ugreencom::offer_details::v1::normalized::{
    NormalizeError, UgreenComNormalizedDocument,
};
use html_extractor::io::{Architecture, Attr, collection, particle, trash};
use html_extractor::serde_json;
use retailer_sauce::structuralized::StructuralizedData;
use structuralize::UgreenComDocument;

pub mod normalized;
pub mod raw;
mod structuralize;

/// The main product section. The product nav section below it renders a second "Shop Now"
/// button, and the recommendations their own product cards.
const MAIN: &str = r#"[id^="shopify-section-template--"][id*="__ug_main_product_"]"#;

fn main(selector: &str) -> String {
    format!("{MAIN} {selector}")
}

pub fn architecture() -> Architecture {
    Architecture::new(vec![
        // Must run first: `trash` detaches its match, and svg elements sit inside body, so
        // anything claiming body ahead of this would carry them off first.
        trash(r#"svg"#),
        // The shop currency, on all 66 pages. The analytics `meta` variable names none: the
        // currency is a separate assignment (`meta.currency = 'USD'`) in the same script.
        particle(
            "meta#in-context-paypal-metadata",
            "paypal",
            vec![Attr::all()],
        ),
        // One block on every page, the shop's `Organization`; no page has a `Product` island.
        // Read anyway so a `Product` island the theme adds later reaches the raw schema.
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
        // Shopify's analytics `meta`. On all 66 pages, the product's only record: id, handle,
        // vendor, and one variant with id, price in cents, name and SKU.
        //
        // It sits in an anonymous inline script. The other Shopify storefronts anchor it as
        // `script:has(+ script.analytics)`, but only the Scraper backend parses `:has`; the
        // Kuchiki backends reject the selector. So it is anchored forwards instead: on every page
        // it is the fourth script after Shopify's `monorail-edge` dns-prefetch link, the start of
        // the injected `content_for_header` block. If Shopify inserts a script there, this
        // matches another anonymous script, `meta` is not found and `analytics` comes out without
        // it, which the data-backed test catches.
        particle(
            concat!(
                r#"link[rel="dns-prefetch"][href="https://monorail-edge.shopifysvc.com"]"#,
                " + script + script + script + script"
            ),
            "analytics",
            vec![Attr::all(), Attr::javascript_vars("meta", "meta")],
        ),
        // The product title. The section renders it in two `h1`s (desktop and mobile), so a
        // single-match particle can't read the heading.
        particle(
            r#"meta[property="og:title"]"#,
            "og_title",
            vec![Attr::standard("content")],
        ),
        particle(
            r#"link[rel="canonical"]"#,
            "canonical",
            vec![Attr::standard("href")],
        ),
        // The regional store selling the product. `data-product-url` is JSON:
        // `{"channel_url_list": [...]}`, one `us.ugreen.com` or `ai.ugreen.com` link. Missing on
        // `us-85185`, whose product has no sales channel; present on the other 65 pages.
        particle(
            &main("ug-product-shop-now button"),
            "shop_now",
            vec![Attr::all(), Attr::json("data-product-url", "channels")],
        ),
        // The main gallery, 1 to 7 images per page and no video. The item's `data-media-id` is
        // prefixed with the section id; the `img` carries the bare media id. `src` ends in a
        // `&width=1445` display width.
        collection(
            &main(".product__media-item"),
            "media",
            vec![
                particle(
                    ".product__media-item",
                    "",
                    vec![Attr::pick_as("data-media-type", "media_type")],
                ),
                particle(
                    "img",
                    "",
                    vec![
                        Attr::pick_as("data-media-id", "id"),
                        Attr::standard("src"),
                        Attr::standard("alt"),
                    ],
                ),
            ],
        ),
    ])
}

pub fn normalize(json: &str) -> Result<bodul_shared::NormalizedData, String> {
    let raw: raw::UgreenComRawDocument = serde_json::from_str(json).map_err(|e| e.to_string())?;

    let normalized = UgreenComNormalizedDocument::from_raw(raw)
        .map_err(|error: NormalizeError| error.to_string())?;

    let document_json = serde_json::to_string(&normalized).map_err(|e| e.to_string())?;

    Ok(bodul_shared::NormalizedData::just_document_json(
        document_json,
    ))
}

pub fn structuralize(json: &str) -> Result<StructuralizedData, String> {
    let normalized: UgreenComNormalizedDocument =
        serde_json::from_str(json).map_err(|e| e.to_string())?;

    let document =
        UgreenComDocument::from_normalized(normalized).map_err(|error| error.to_string())?;

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
        "/../../../data/ugreencom/offer_details/sources"
    );

    /// Pages whose product links to no sales channel, so the section renders no "Shop Now"
    /// button.
    const WITHOUT_SHOP_NOW: &[&str] = &["products-us-85185.html"];

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
    fn extracts_currency_analytics_title_and_media_from_every_page() {
        let Some(pages) = pages() else { return };
        let arch = architecture();
        for page in pages {
            let name = page.display();
            let file = page
                .file_name()
                .and_then(|f| f.to_str())
                .unwrap_or_default();
            let json = Backend::Scraper.extract(&read(&page), &arch).unwrap().json;

            assert_eq!(json["paypal"]["data-currency"], "USD", "{name}");

            let ld_json = json["ld-json"]
                .as_array()
                .unwrap_or_else(|| panic!("{name}: no ld-json"));
            assert!(
                ld_json
                    .iter()
                    .any(|entry| entry["data"]["@type"] == "Organization"),
                "{name}: no Organization island"
            );

            let title = json["og_title"]["content"]
                .as_str()
                .unwrap_or_else(|| panic!("{name}: no og:title"));
            assert!(!title.is_empty(), "{name}: empty title");

            assert!(
                json["canonical"]["href"]
                    .as_str()
                    .is_some_and(|href| href.starts_with("https://www.ugreen.com/products/")),
                "{name}: canonical {}",
                json["canonical"]
            );

            let product = &json["analytics"]["meta"]["product"];
            assert!(
                product["id"].is_u64(),
                "{name}: product id {}",
                product["id"]
            );
            assert!(
                product["handle"].as_str().is_some_and(|h| !h.is_empty()),
                "{name}: no handle"
            );
            let variants = product["variants"]
                .as_array()
                .unwrap_or_else(|| panic!("{name}: no variants"));
            assert_eq!(variants.len(), 1, "{name}: variants");
            let variant = &variants[0];
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
            assert_eq!(variant["name"], title, "{name}: variant name");
            let sku = variant["sku"]
                .as_str()
                .filter(|sku| !sku.is_empty())
                .unwrap_or_else(|| panic!("{name}: no sku"));

            let shop_now = &json["shop_now"];
            if WITHOUT_SHOP_NOW.contains(&file) {
                assert!(shop_now.is_null(), "{name}: unexpected Shop Now {shop_now}");
            } else {
                assert_eq!(shop_now["data-market"], "USA", "{name}");
                assert_eq!(shop_now["data-product-sku"], sku, "{name}: button sku");
                let channels = shop_now["channels"]["channel_url_list"]
                    .as_array()
                    .unwrap_or_else(|| panic!("{name}: no channel list"));
                assert!(!channels.is_empty(), "{name}: no channels");
            }

            let media = json["media"]
                .as_array()
                .unwrap_or_else(|| panic!("{name}: no media"));
            assert!(!media.is_empty(), "{name}: no media");
            for item in media {
                assert_eq!(item["media_type"], "image", "{name}");
                assert!(
                    item["src"].as_str().is_some_and(|src| !src.is_empty()),
                    "{name}: media src {item}"
                );
                assert!(
                    item["id"].as_str().is_some_and(|id| !id.is_empty()),
                    "{name}: media id {item}"
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
