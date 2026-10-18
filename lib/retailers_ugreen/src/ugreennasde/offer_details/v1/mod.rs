//! Version 1 of the offer details.
//!
//! `ai-de.ugreen.com` is a Shopify Liquid storefront (shop `ugreen-de-nas`, theme `nas/main-de`),
//! one regional build of the theme `ugreennasau`, `ugreennasca` and `ugreennases` share. The
//! theme is Dawn-derived: its main product section renders Dawn's price snippet, a
//! `variant-selects` picker with the `product.variants | json` script, and a `product-form`. It
//! has none of the Horizon islands `ugreenfr` and `ugreeneu` read (`product-price`,
//! `variant-picker`, `product-form-component`, `data-ug-auto-discount`); those names appear on
//! the page only in stylesheet URLs. It is also not `ugreennasjp`'s `nas-jp/main` theme. The one
//! difference from the other three builds is the gallery; see `images`.
//!
//! Survey of the 23 saved pages, all from this shop (`Shopify.shop`, PayPal shop id) and all
//! product pages; one is `worry-free-purchase`, a 0,98 € purchase-protection add-on with 100
//! variants, rendered by the stock Dawn product template:
//!
//! - Currency: EUR on every page, in the PayPal metadata tag, `Shopify.currency.active` and the
//!   rendered prices (`2.349,99€`). The analytics prices name none.
//! - ld+json: every block parses. Decimal commas appear only in rendered text, never in a JSON
//!   number: the risk `ugreende` hit (`"price": 43,99`) has not shown up here; if it does, the
//!   whole extraction fails with invalid JSON, so it can't pass unnoticed. Each page carries the
//!   shop's `Organization` twice, a `WebPage` (whose `description` is a rendered Liquid error
//!   object on every product page) and a `WebSite`, but the theme renders no product ld+json:
//!   only `worry-free-purchase` has a `Product`, with one `Offer` per variant. Read anyway, so a
//!   product island showing up later reaches the raw schema.
//! - Analytics `meta`: on every page. Product id, handle, and every variant (140 in all, 40
//!   outside the add-on) with id, price in cents, `sku` (never empty) and `public_title`.
//! - Picker JSON: on 20 pages. Missing on the 3 pages whose product renders no picker
//!   (`ugreen-nas-10gbps-m2-nvme-ssd-enclosure`,
//!   `ugreen-nas-72w-dc-uninterruptible-power-supply`, `ugreen-nas-backup-power-120w-12000mah`);
//!   all three are single-variant products, but 9 other single-variant products do render one.
//!   Where present, its variants match the analytics variants one for one (same ids, order, SKUs
//!   and prices).
//! - Price block: on every page, for the selected variant, which is the first variant on every
//!   page. `price--on-sale` on 14 pages, `price--sold-out` on 2
//!   (`ugreen-nas-10gbps-m2-nvme-ssd-enclosure`, which has no picker, and
//!   `ugreen-nasync-dxp4800-plus-refurbished-4-bay-10gbe-nas`); one more variant is unavailable
//!   on `ugreen-nasync-refurbished-dxp2800-2-bay`. Off sale, the `<s>` repeats the price.
//! - No automatic-discount price: no page carries `data-ug-auto-discount` or a price other than
//!   the Shopify variant price; the gallery's `ug-discount-flag` and the sticky bar's discount
//!   tag restate the compare-at saving.
//!
//! Where the facts are:
//!
//! - Title: the main section's `h1` (German; on
//!   `ugreen-nas-72w-dc-uninterruptible-power-supply` it ends in a shipping note,
//!   `(Versand ab 2. Nov.)`). `og:title` is a marketing title on most pages.
//! - SKU: analytics `meta` variants (and the picker JSON where present).
//! - GTIN: the picker JSON's `barcode`, on every variant on 10 pages, on some on 8 and on none
//!   on 1 (and the add-on). 10 bundle variants on 7 pages (`DXP480T Plus + 120W DC USV`) carry a
//!   14-digit `178…` code rather than an EAN-13. Pages without a picker carry none anywhere.
//! - Price: analytics `meta` variant `price` in cents; the picker JSON's equals it.
//! - Currency: the PayPal metadata tag.
//! - Compare-at: the picker JSON's `compare_at_price` per variant; it equals the price on 7
//!   single-variant products, so only a higher one is a discount. On pages without a picker only
//!   the price block's `<s>`, for the selected (only) variant.
//! - Availability: the picker JSON's `available` per variant; on pages without a picker only
//!   the price block's `price--sold-out` and the cart form's disabled `id` input.
//! - Variants: analytics `meta`, with the picker JSON's records where present.
//! - Images: the main gallery; the picker JSON also carries one `featured_image` per variant.
//!
//! There is no Apple Pay shop-capabilities script, `xcotton_pp_variants` or `__NEXT_DATA__` on
//! any page.

use crate::ugreennasde::offer_details::v1::normalized::{
    NormalizeError, UgreenNasDeNormalizedDocument,
};
use html_extractor::io::{Architecture, Attr, collection, particle, segment, trash};
use html_extractor::serde_json;
use retailer_sauce::structuralized::StructuralizedData;
use structuralize::UgreenNasDeDocument;

pub mod normalized;
pub mod raw;
mod structuralize;

/// The main product section. Other sections render the navigation, overview, comparison and
/// review blocks; the sticky bar with its second price and cart form sits inside this one.
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
        // Four or five islands per page; see the module comment. `Attr::all()` alongside the
        // JSON read: a stray attribute appearing on the tag fails the strict raw schema instead
        // of disappearing silently.
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
        // it in `head`: the monorail `dns-prefetch` link, followed by the session-beacon,
        // Trekkie-shim and web-pixels-loader scripts. That holds on all 23 saved pages.
        // `script:has(+ script.analytics)`, which anchors on the script that follows it, only
        // parses on `Backend::Scraper`; the Kuchiki backends reject `:has` as an invalid
        // selector. Being positional, this anchor breaks if Shopify adds or drops a script
        // between the two; the strict raw schema then reports a missing `meta`. `Attr::all()`
        // adds nothing until the script gains an attribute.
        particle(
            concat!(
                r#"link[rel="dns-prefetch"][href="https://monorail-edge.shopifysvc.com"]"#,
                " + script + script + script + script"
            ),
            "analytics",
            vec![Attr::all(), Attr::javascript_vars("meta", "meta")],
        ),
        // The product title, once per section. Only the text: the tag's inline style changes
        // with the theme's layout.
        particle(&main("h1"), "title", vec![Attr::inner_text("text")]),
        // Liquid's `product.variants | json`: `sku`, `barcode`, `price`, `compare_at_price`,
        // `available`, options and image per variant. Missing on the 3 single-variant pages that
        // render no picker.
        particle(
            &main(r#"variant-selects script[type="application/json"]"#),
            "variants",
            vec![Attr::all(), Attr::json("", "data")],
        ),
        // The variant the price block describes. `disabled` (empty) is present exactly when
        // that variant is sold out. The sticky bar's form sits outside `.product__info-container`
        // and the price block's installment form is not in a `product-form`.
        particle(
            &main(
                r#".product__info-container product-form form[data-type="add-to-cart-form"] input[name="id"]"#,
            ),
            "selected_variant",
            vec![Attr::standard("value"), Attr::standard("disabled")],
        ),
        // Dawn's price snippet for the selected variant. `price--on-sale` in the class marks a
        // compare-at price above the price; without it the `<s>` is empty. `price--sold-out`
        // marks a sold-out variant. The sticky bar's copy is `#sticky-price-…`.
        segment(
            &main(r#"[id^="price-template--"] > .price"#),
            "price",
            vec![
                particle(".price", "", vec![Attr::standard("class")]),
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
        // The gallery's image slides, one image each. This build's gallery differs from the other
        // three stores': `ug-product-gallery` items typed by `data-media-type`, where the
        // others render `product-gallery` with an `.image-zoom-wrapper` per slide. Video items
        // (one each on `ugreen-nasync-dh2300-nas-speicher` and
        // `ugreen-nasync-dxp4800-pro-4-bay-nas`) render their poster thumbnail and are left out;
        // the image items' other `img` is the discount-flag badge outside
        // `.ug-product-gallery__media`. Dawn's media list (the image inside `modal-opener`;
        // `<noscript>` holds a copy) renders `worry-free-purchase`.
        collection(
            &format!(
                "{}, {}",
                main(concat!(
                    r#"media-gallery .ug-product-gallery__item[data-media-type="image"]"#,
                    " .ug-product-gallery__media img"
                )),
                main("media-gallery .product__media-list modal-opener img"),
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
    let raw: raw::UgreenNasDeRawDocument = serde_json::from_str(json).map_err(|e| e.to_string())?;

    let normalized = UgreenNasDeNormalizedDocument::from_raw(raw)
        .map_err(|error: NormalizeError| error.to_string())?;

    let document_json = serde_json::to_string(&normalized).map_err(|e| e.to_string())?;

    Ok(bodul_shared::NormalizedData::just_document_json(
        document_json,
    ))
}

pub fn structuralize(json: &str) -> Result<StructuralizedData, String> {
    let normalized: UgreenNasDeNormalizedDocument =
        serde_json::from_str(json).map_err(|e| e.to_string())?;

    let document =
        UgreenNasDeDocument::from_normalized(normalized).map_err(|error| error.to_string())?;

    let document_json = serde_json::to_string(&document).map_err(|e| e.to_string())?;

    Ok(StructuralizedData::just_document_json(document_json))
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use html_extractor::Backend;
    use html_extractor::io::Architecture;
    use html_extractor::serde_json::Value;

    use super::{architecture, normalize, structuralize};

    /// Saved product pages, kept outside the repository next to the monorepo checkout.
    const DATA_DIR: &str = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../../data/ugreennasde/offer_details/sources"
    );

    /// The shop currency, and how the rendered prices name it.
    const CURRENCY: &str = "EUR";
    const PRICE_MARK: &str = "€";

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

    fn assert_none(failures: Vec<String>) {
        assert!(
            failures.is_empty(),
            "{} failures:\n{}",
            failures.len(),
            failures.join("\n")
        );
    }

    fn non_empty(value: &Value) -> bool {
        value.as_str().is_some_and(|v| !v.trim().is_empty())
    }

    /// A rendered price in cents: its digits, since every price has two decimals whichever
    /// separators the locale uses (`$1,149.99 AUD`, `2.349,99€`, `€2.349,99`).
    fn cents(text: &Value) -> Option<u64> {
        let digits: String = text
            .as_str()?
            .chars()
            .filter(char::is_ascii_digit)
            .collect();
        digits.parse().ok()
    }

    /// Every island the architecture reads, and the facts normalization needs from it.
    fn check_page(page: &Path, arch: &Architecture) -> Vec<String> {
        let json = match Backend::Scraper.extract(&read(page), arch) {
            Ok(result) => result.json,
            Err(e) => return vec![format!("{}: extract: {e}", page.display())],
        };
        if std::env::var("DUMP").is_ok() {
            let mut j = json.clone();
            if let Some(v) = j["variants"]["data"].as_array_mut() {
                v.truncate(1);
            }
            if let Some(v) = j["analytics"]["meta"]["product"]["variants"].as_array_mut() {
                v.truncate(2);
            }
            if let Some(v) = j["images"].as_array_mut() {
                v.truncate(2);
            }
            if let Some(v) = j["ld-json"].as_array_mut() {
                for e in v.iter_mut() {
                    let t = e["data"]["@type"].clone();
                    e["data"] = t;
                }
            }
            eprintln!(
                "{}\n{}",
                page.display(),
                html_extractor::serde_json::to_string_pretty(&j).unwrap()
            );
        }
        let mut failures = Vec::new();
        macro_rules! fail {
            ($($arg:tt)*) => {
                failures.push(format!("{}: {}", page.display(), format!($($arg)*)))
            };
        }

        if json["paypal"]["data-currency"] != CURRENCY {
            fail!("paypal {}", json["paypal"]);
        }

        // ld+json: the shop's islands, and a `Product` only for the add-on, with offers.
        let ld_json = json["ld-json"].as_array().cloned().unwrap_or_default();
        if ld_json.len() < 4 {
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
            let data = &entry["data"];
            match data["@type"].as_str() {
                Some("Organization" | "WebPage" | "WebSite") => {}
                Some("Product") => {
                    let offers = data["offers"].as_array().cloned().unwrap_or_default();
                    if offers.is_empty()
                        || offers
                            .iter()
                            .any(|offer| offer["priceCurrency"] != CURRENCY)
                    {
                        fail!("ld+json product offers {}", data["offers"]);
                    }
                }
                other => fail!("ld+json type {other:?}"),
            }
        }

        // Analytics meta: the product and every variant.
        let analytics = &json["analytics"];
        if analytics.as_object().map(|a| a.len()) != Some(1) {
            fail!(
                "analytics keys {:?}",
                analytics.as_object().map(|a| a.keys().collect::<Vec<_>>())
            );
        }
        let product = &analytics["meta"]["product"];
        if !product["id"].is_u64() || !non_empty(&product["handle"]) {
            fail!("analytics product {} {}", product["id"], product["handle"]);
        }
        let variants = product["variants"].as_array().cloned().unwrap_or_default();
        if variants.is_empty() {
            fail!("no analytics variants");
        }
        for variant in &variants {
            if !variant["id"].is_u64() || !variant["price"].is_u64() || !non_empty(&variant["sku"])
            {
                fail!("analytics variant {variant}");
            }
        }

        if !non_empty(&json["title"]["text"]) {
            fail!("title {}", json["title"]);
        }

        // The picker's records: the analytics variants with compare-at, barcode and
        // availability; missing only on single-variant products.
        let records = &json["variants"];
        let records = if records.is_null() {
            if variants.len() != 1 {
                fail!("no picker JSON for {} variants", variants.len());
            }
            None
        } else {
            if records["type"] != "application/json"
                || records.as_object().map(|r| r.len()) != Some(2)
            {
                fail!(
                    "variants attributes {:?}",
                    records.as_object().map(|r| r.keys().collect::<Vec<_>>())
                );
            }
            let records = records["data"].as_array().cloned().unwrap_or_default();
            let key = |v: &Value| (v["id"].clone(), v["sku"].clone(), v["price"].clone());
            if records.iter().map(key).ne(variants.iter().map(key)) {
                fail!("picker variants differ from analytics variants");
            }
            for record in &records {
                if !record["available"].is_boolean()
                    || !(record["compare_at_price"].is_null()
                        || record["compare_at_price"].is_u64())
                {
                    fail!("picker variant {}", record["id"]);
                }
            }
            Some(records)
        };

        // The selected variant and its price block.
        let selected_id = json["selected_variant"]["value"].as_str();
        let Some(index) = variants
            .iter()
            .position(|v| v["id"].as_u64().map(|id| id.to_string()).as_deref() == selected_id)
        else {
            fail!("selected variant {selected_id:?} not among the analytics variants");
            return failures;
        };
        let disabled = json["selected_variant"].get("disabled").is_some();
        let record = records.as_ref().map(|records| &records[index]);

        let price = &json["price"];
        let class: Vec<&str> = price["class"]
            .as_str()
            .unwrap_or_default()
            .split_whitespace()
            .collect();
        if !class.contains(&"price") {
            fail!("price class {class:?}");
        }
        if class.contains(&"price--sold-out") != disabled {
            fail!("price class {class:?} with disabled {disabled}");
        }
        if let Some(record) = record
            && record["available"] == disabled
        {
            fail!("available {} with disabled {disabled}", record["available"]);
        }
        let price_cents = variants[index]["price"].as_u64();
        for key in ["regular", "sale"] {
            let text = &price[key]["text"];
            if !text.as_str().is_some_and(|t| t.contains(PRICE_MARK)) || cents(text) != price_cents
            {
                fail!("price {key} {text} for {price_cents:?}");
            }
        }
        let compare_at = &price["compare_at"]["text"];
        if class.contains(&"price--on-sale") {
            let compare_cents = cents(compare_at);
            if !compare_cents.is_some_and(|c| Some(c) > price_cents)
                || record.is_some_and(|r| r["compare_at_price"].as_u64() != compare_cents)
            {
                fail!("on sale at {price_cents:?}, compare-at {compare_at}");
            }
        } else if !(compare_at.as_str().is_some_and(|t| t.trim().is_empty())
            || cents(compare_at) == price_cents)
        {
            fail!("off sale at {price_cents:?}, compare-at {compare_at}");
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
    fn extracts_paypal_ld_json_analytics_variants_price_and_gallery_from_every_page() {
        let Some(pages) = pages() else { return };
        let arch = architecture();
        assert_none(
            pages
                .iter()
                .flat_map(|page| check_page(page, &arch))
                .collect(),
        );
    }

    #[test]
    fn backends_agree_on_every_page() {
        let Some(pages) = pages() else { return };
        let arch = architecture();
        let mut failures = Vec::new();
        for page in &pages {
            let html = read(page);
            let expected = Backend::Scraper.extract(&html, &arch).map(|r| r.json);
            for backend in Backend::ALL {
                let json = backend.extract(&html, &arch).map(|r| r.json);
                match (json, &expected) {
                    (Ok(json), Ok(expected)) if json == *expected => {}
                    (json, expected) => failures.push(format!(
                        "{}: {backend:?} {} vs Scraper {}",
                        page.display(),
                        json.map_or_else(|e| e.to_string(), |_| "differs".to_string()),
                        expected
                            .as_ref()
                            .map_or_else(|e| e.to_string(), |_| "ok".to_string()),
                    )),
                }
            }
        }
        assert_none(failures);
    }

    #[test]
    fn normalizes_and_structuralizes_every_page() {
        let Some(pages) = pages() else { return };
        let arch = architecture();
        let failures: Vec<String> = pages
            .iter()
            .filter_map(|page| {
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
                pipeline.err().map(|e| format!("{}: {e}", page.display()))
            })
            .collect();
        assert_none(failures);
    }
}
