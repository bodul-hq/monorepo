//! Version 1 of the offer details.

use crate::minisforumkr::offer_details::v1::normalized::{
    MinisforumKrNormalizedDocument, NormalizeError,
};
use html_extractor::io::{Architecture, Attr, collection, particle, trash};
use html_extractor::serde_json;

pub mod normalized;
pub mod raw;
mod structuralize;

// #[cfg(test)]
// mod stage_files_tests;

pub fn architecture() -> Architecture {
    Architecture::new(vec![
        trash(r#"svg"#),
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
        // Shopify's analytics `meta` sits in an anonymous inline script; the one place to anchor
        // it is the `script.analytics` that follows it.
        particle(
            "script:has(+ script.analytics)",
            "analytics",
            vec![Attr::all(), Attr::javascript_vars("meta", "meta")],
        ),
        particle(
            "variant-picker",
            "variant_picker",
            vec![
                Attr::pick_as("data-product-id", "product_id"),
                Attr::pick_as("data-product-url", "product_url"),
            ],
        ),
        collection(
            "variant-picker fieldset legend > span",
            "option_names",
            vec![particle("span", "", vec![Attr::inner_text("text")])],
        ),
        particle(
            r#"variant-picker script[type="application/json"]"#,
            "selected_variant",
            vec![Attr::all(), Attr::json("", "data")],
        ),
        // Slides only: the zoom dialog renders the same images again outside them.
        collection(
            "media-gallery slideshow-slide .product-media img",
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
    let raw: raw::MinisforumKrRawDocument =
        serde_json::from_str(json).map_err(|e| e.to_string())?;

    let normalized = MinisforumKrNormalizedDocument::from_raw(raw)
        .map_err(|error: NormalizeError| error.to_string())?;

    let document_json = serde_json::to_string(&normalized).map_err(|e| e.to_string())?;

    Ok(bodul_shared::NormalizedData::just_document_json(
        document_json,
    ))
}

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
