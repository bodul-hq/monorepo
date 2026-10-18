//! Shopify Horizon offer details for Ugreen US. Reads the main product section,
//! analytics, and valid Product/ProductGroup JSON-LD, including every published offer.

use crate::ugreenus::offer_details::v1::normalized::{NormalizeError, UgreenUsNormalizedDocument};
use html_extractor::io::{Architecture, Attr, collection, particle, trash};
use html_extractor::serde_json;

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
        trash(r#"svg"#),
        collection(
            r#"script[type="application/ld+json"]"#,
            "ld-json",
            vec![particle(
                "script",
                "",
                vec![Attr::all(), Attr::json("", "data")],
            )],
        ),
        // The analytics meta is the fourth script after Shopify's monorail prefetch.
        // This forward sibling selector also works on backends without CSS :has().
        particle(
            r#"link[href="https://monorail-edge.shopifysvc.com"] + script + script + script + script"#,
            "analytics",
            vec![Attr::all(), Attr::javascript_vars("meta", "meta")],
        ),
        particle(&main("h1"), "title", vec![Attr::inner_text("text")]),
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
        particle(
            &main(FORM),
            "product_form",
            vec![
                Attr::pick_as("data-product-id", "product_id"),
                Attr::pick_as("data-product-url", "product_url"),
            ],
        ),
        particle(
            &main(&format!(r#"{FORM} input[name="id"]"#)),
            "selected_variant_id",
            vec![Attr::standard("value")],
        ),
        particle(
            &main(&format!(r#"{FORM} button[name="add"]"#)),
            "add_to_cart",
            vec![Attr::standard("disabled")],
        ),
        particle(
            &main("variant-picker"),
            "variant_picker",
            vec![
                Attr::pick_as("data-product-id", "product_id"),
                Attr::pick_as("data-product-url", "product_url"),
            ],
        ),
        // The legend's title only; its other span repeats the selected value.
        collection(
            &main("variant-picker fieldset legend > span.variant-option__legend-title"),
            "option_names",
            vec![particle("span", "", vec![Attr::inner_text("text")])],
        ),
        particle(
            &main(r#"variant-picker script[type="application/json"]"#),
            "selected_variant",
            vec![Attr::all(), Attr::json("", "data")],
        ),
        // Image slides only: a video slide renders its poster thumbnail twice.
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
    let raw: raw::UgreenUsRawDocument = serde_json::from_str(json).map_err(|e| e.to_string())?;

    let normalized = UgreenUsNormalizedDocument::from_raw(raw)
        .map_err(|error: NormalizeError| error.to_string())?;

    let document_json = serde_json::to_string(&normalized).map_err(|e| e.to_string())?;

    Ok(bodul_shared::NormalizedData::just_document_json(
        document_json,
    ))
}

#[cfg(test)]
mod tests;

/// Converts the normalized product into validated offer variations.
pub fn structuralize(
    json: &str,
) -> Result<retailer_sauce::structuralized::StructuralizedData, String> {
    let input = serde_json::from_str(json).map_err(|error| error.to_string())?;
    let document =
        structuralize::Document::from_normalized(input).map_err(|error| error.to_string())?;
    let json = serde_json::to_string(&document).map_err(|error| error.to_string())?;
    Ok(retailer_sauce::structuralized::StructuralizedData::just_document_json(json))
}
