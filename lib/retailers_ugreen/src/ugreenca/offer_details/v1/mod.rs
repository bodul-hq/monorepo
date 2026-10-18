//! Version 1 of the offer details.

use crate::ugreenca::offer_details::v1::normalized::{NormalizeError, UgreenCaNormalizedDocument};
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
        particle(
            "script#apple-pay-shop-capabilities",
            "shop",
            vec![Attr::all(), Attr::json("", "data")],
        ),
        // Shopify's analytics `meta` sits in an anonymous inline script; the one place to anchor
        // it is the `script.analytics` that follows it.
        particle(
            "script:has(+ script.analytics)",
            "analytics",
            vec![Attr::all(), Attr::javascript_vars("meta", "meta")],
        ),
        particle(
            "h1.t4s-product__title",
            "title",
            vec![Attr::inner_text("text")],
        ),
        particle(
            "[data-product-featured]",
            "featured",
            vec![Attr::json("data-product-featured", "data")],
        ),
        // The picker's islands; a `Default Title` product renders neither.
        particle(
            "script.pr_variants_json",
            "variants",
            vec![Attr::all(), Attr::json("", "data")],
        ),
        particle(
            "script.pr_options_json",
            "options",
            vec![Attr::all(), Attr::json("", "data")],
        ),
        // `data-master` is the lazy-loaded image's address without a display width; `src` is an
        // empty SVG placeholder.
        collection(
            "[data-main-slide]",
            "media",
            vec![
                particle(
                    "[data-main-slide]",
                    "",
                    vec![
                        Attr::pick_as("data-media-id", "id"),
                        Attr::pick_as("data-media-type", "media_type"),
                    ],
                ),
                particle(
                    "img[data-master]",
                    "",
                    vec![Attr::pick_as("data-master", "src")],
                ),
            ],
        ),
    ])
}

pub fn normalize(json: &str) -> Result<bodul_shared::NormalizedData, String> {
    let raw: raw::UgreenCaRawDocument = serde_json::from_str(json).map_err(|e| e.to_string())?;

    let normalized = UgreenCaNormalizedDocument::from_raw(raw)
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
