//! Version 1 of the offer details.

use crate::minisforumuk::offer_details::v1::normalized::{
    MinisforumUkNormalizedDocument, NormalizeError,
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
        // The one statement of the shop currency on every page: Liquid's record names none, and
        // the schema.org island is missing where the product has no price.
        particle(
            "meta#in-context-paypal-metadata",
            "paypal",
            vec![Attr::all()],
        ),
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
        // Liquid's `product | json`, rendered by a storefront app on every product page,
        // including the ones whose template has no variant picker or gallery.
        particle(
            "script#xcotton_pp_variants",
            "product_json",
            vec![Attr::all(), Attr::json("", "data")],
        ),
    ])
}

pub fn normalize(json: &str) -> Result<bodul_shared::NormalizedData, String> {
    let raw: raw::MinisforumUkRawDocument =
        serde_json::from_str(json).map_err(|e| e.to_string())?;

    let normalized = MinisforumUkNormalizedDocument::from_raw(raw)
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
