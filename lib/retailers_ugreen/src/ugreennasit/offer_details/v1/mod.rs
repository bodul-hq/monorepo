//! Version 1 of the offer details.
//!
//! `ai-it.ugreen.com` is a Shopify Liquid storefront on a Dawn 13.0.1 theme (`nas/main-it`),
//! shared with `ugreennaseu`, `ugreennasfr` and `ugreennasuk`. The 18 saved pages are in Italian
//! and price in EUR.
//!
//! The ld+json is deliberately not read. A `Product` island is rendered only by Dawn's default
//! product template, which few pages use, and on some `ugreennasit` pages the theme's
//! hand-written `WebPage` island holds a raw control character, which is not JSON and would
//! fail the whole extraction.
//!
//! Most pages use the theme's landing template; the rest, including Seel's
//! `products-worry-free-purchase` add-on, use Dawn's default one. Both render, on every page,
//! the PayPal metadata tag, Shopify's analytics `meta`, the title, the main product form and the
//! gallery. The variant picker is absent on some products, and the price block is absent on
//! worry-free-purchase. Glood's loader embeds the complete Liquid product on every saved page,
//! including the two-variant French HDMI cable with no picker. Normalize cross-checks these
//! records against analytics and any picker/back-in-stock records. Structuralize emits one
//! offer per variant, except purchase protection, which produces no offers.
//!
//! `og:url` supplies the regional product URL and EU locale path. Canonical links can point at
//! another store or even another product, so they do not select an offer's URL.

use html_extractor::serde_json;
use retailer_sauce::structuralized::StructuralizedData;

pub mod normalized;
#[cfg(test)]
mod pipeline_tests;
pub mod raw;
mod structuralize;

use html_extractor::io::{Architecture, Attr, collection, particle, trash};

/// The main product section. Other sections render recommendations with their own prices.
const MAIN: &str = r#"[id^="shopify-section-template--"][id$="__main"]"#;

/// The product's own price block. The sticky bottom bar inside the section renders a copy
/// outside the info container.
const PRICE: &str = ".product__info-container .product__price";

fn main(selector: &str) -> String {
    format!("{MAIN} {selector}")
}

pub fn architecture() -> Architecture {
    Architecture::new(vec![
        // Must run first: `trash` detaches its match, and svg elements sit inside body, so
        // anything claiming body ahead of this would carry them off first.
        trash(r#"svg"#),
        // Canonical links can point at another product/store; og:url retains the regional locale.
        particle(
            r#"meta[property="og:url"]"#,
            "product_url",
            vec![Attr::standard("content")],
        ),
        // Glood embeds the full Liquid product even where the theme omits the variant picker.
        // Read only this script; normalization deserializes its product JSON without running JS.
        particle(
            "script#glood-loader",
            "glood",
            vec![Attr::all(), Attr::inner_text("text")],
        ),
        // The shop currency; nothing in the analytics `meta` or the price text names it.
        particle(
            "meta#in-context-paypal-metadata",
            "paypal",
            vec![Attr::all()],
        ),
        // Shopify's analytics `meta`: product id, vendor, handle and every variant's id, price in
        // cents, name and SKU. It sits in an anonymous inline script. `script:has(+
        // script.analytics)` only parses on `Backend::Scraper`, so it is anchored forwards, as on
        // `ugreenin`: on every saved page it is the fourth script after Shopify's `monorail-edge`
        // dns-prefetch link. If Shopify inserts a script there, `meta` is not found and the
        // data-backed test catches it.
        particle(
            concat!(
                r#"link[rel="dns-prefetch"][href="https://monorail-edge.shopifysvc.com"]"#,
                " + script + script + script + script"
            ),
            "analytics",
            vec![Attr::all(), Attr::javascript_vars("meta", "meta")],
        ),
        // The landing template renders `h1.product__title`, Dawn's default product template
        // `.product__title > h1`. Some pages also render an A+ content `h1` in the section.
        particle(
            &format!("{MAIN} h1.product__title, {MAIN} .product__title > h1"),
            "title",
            vec![Attr::inner_text("text")],
        ),
        // The variant the price block shows: the selected or first available one.
        particle(
            &main(r#"form[id^="product-form-template--"] input[name="id"]"#),
            "selected_variant_id",
            vec![Attr::standard("value")],
        ),
        // Liquid's variant records, with compare-at prices, availability and barcodes. Dawn
        // renders the picker only for a product with a choice, so single-variant pages lack it.
        particle(
            &main(r#"variant-selects script[type="application/json"]"#),
            "variants",
            vec![Attr::all(), Attr::json("", "data")],
        ),
        // The selected variant's displayed prices, formatted for the locale (`2.349,99€`). The
        // block's class says whether it is on sale (`price--on-sale`) or sold out
        // (`price--sold-out`).
        particle(
            &main(&format!("{PRICE} .price")),
            "price",
            vec![Attr::standard("class")],
        ),
        particle(
            &main(&format!("{PRICE} .price__regular .price-item--regular")),
            "price_regular",
            vec![Attr::inner_text("text")],
        ),
        particle(
            &main(&format!("{PRICE} .price__sale .price-item--sale")),
            "price_sale",
            vec![Attr::inner_text("text")],
        ),
        particle(
            &main(&format!("{PRICE} .price__sale s.price-item--regular")),
            "price_compare",
            vec![Attr::inner_text("text")],
        ),
        // The gallery's main images: the landing template wraps each in an
        // `.image-zoom-wrapper`, Dawn's default template in a `.product__media`. Thumbnails,
        // icons and discount flags in the gallery are not product images.
        collection(
            &format!(
                "{MAIN} media-gallery .image-zoom-wrapper img, {MAIN} media-gallery .product__media img"
            ),
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
    let raw: raw::UgreenNasItRawDocument = serde_json::from_str(json).map_err(|e| e.to_string())?;
    let normalized =
        normalized::UgreenNasItNormalizedDocument::from_raw(raw).map_err(|e| e.to_string())?;
    let json = serde_json::to_string(&normalized).map_err(|e| e.to_string())?;
    Ok(bodul_shared::NormalizedData::just_document_json(json))
}

pub fn structuralize(json: &str) -> Result<StructuralizedData, String> {
    let normalized = serde_json::from_str(json).map_err(|e| e.to_string())?;
    let document = structuralize::UgreenNasItDocument::from_normalized(normalized)
        .map_err(|e| e.to_string())?;
    let json = serde_json::to_string(&document).map_err(|e| e.to_string())?;
    Ok(StructuralizedData::just_document_json(json))
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use html_extractor::Backend;

    use super::architecture;

    /// Saved product pages, kept outside the repository next to the monorepo checkout.
    const DATA_DIR: &str = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../../data/ugreennasit/offer_details/sources"
    );

    /// The saved pages, or `None` when the data checkout is missing so the tests skip.
    pub(super) fn pages() -> Option<Vec<PathBuf>> {
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
    fn extracts_the_islands_from_every_page() {
        let Some(pages) = pages() else { return };
        let arch = architecture();
        for page in pages {
            let name = page.display();
            let json = Backend::Scraper.extract(&read(&page), &arch).unwrap().json;

            assert_eq!(json["paypal"]["data-currency"], "EUR", "{name}");

            let variant_ids: Vec<String> = json["analytics"]["meta"]["product"]["variants"]
                .as_array()
                .unwrap_or_else(|| panic!("{name}: no analytics variants"))
                .iter()
                .map(|variant| variant["id"].to_string())
                .collect();
            assert!(!variant_ids.is_empty(), "{name}: no analytics variants");

            assert!(
                json["title"]["text"]
                    .as_str()
                    .is_some_and(|t| !t.trim().is_empty()),
                "{name}: no title"
            );

            let selected = json["selected_variant_id"]["value"]
                .as_str()
                .unwrap_or_else(|| panic!("{name}: no selected variant"));
            assert!(
                variant_ids.iter().any(|id| id == selected),
                "{name}: selected variant {selected} not in analytics"
            );

            assert!(
                json["images"].as_array().is_some_and(|i| !i.is_empty()),
                "{name}: no images"
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
}
