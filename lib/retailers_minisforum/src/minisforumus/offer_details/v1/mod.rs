//! Version 1 of the offer details.

use crate::minisforumus::offer_details::v1::normalized::{
    MinisforumUsNormalizedDocument, NormalizeError,
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
        // The theme's islands only: review apps render their own `Product` islands, marked with
        // an `id` or a `class`, which restate the product with the apps' reviews.
        collection(
            r#"script[type="application/ld+json"]:not([id]):not([class])"#,
            "ld-json",
            vec![particle(
                "script",
                "",
                // `Attr::all()` alongside the JSON read: a stray attribute appearing on the tag
                // fails the strict raw schema instead of disappearing silently.
                vec![Attr::all(), Attr::json("", "data")],
            )],
        ),
        // An app's copy of Liquid's `product | json`, the only complete record of the product on
        // the page. Only the JSON is read: the tag carries the app's consent-tool attributes.
        particle(
            "script#xcotton_pp_variants",
            "product_json",
            vec![Attr::json("", "data")],
        ),
    ])
}

pub fn normalize(json: &str) -> Result<bodul_shared::NormalizedData, String> {
    let raw: raw::MinisforumUsRawDocument =
        serde_json::from_str(json).map_err(|e| e.to_string())?;

    let normalized = MinisforumUsNormalizedDocument::from_raw(raw)
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
