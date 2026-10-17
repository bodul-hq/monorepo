//! Version 1 of the offer details.

use crate::ankerde::offer_details::v1::normalized::{AnkerdeNormalizedDocument, NormalizeError};
use crate::ankerde::offer_details::v1::structuralize::AnkerDeDocument;
use html_extractor::io::{Architecture, Attr, collection, next_js_app_router, particle, trash};
use html_extractor::serde_json;
use retailer_sauce::structuralized::StructuralizedData;

pub mod normalized;
pub mod raw;
mod structuralize;

// #[cfg(test)]
// mod stage_files_tests;

pub fn architecture() -> Architecture {
    Architecture::new(vec![
        trash(r#"svg"#),
        next_js_app_router("flight"),
        collection(
            r#"script[type="application/ld+json"]"#,
            "ld-json",
            vec![particle("script", "", vec![Attr::json("", "data")])],
        ),
    ])
}

pub fn normalize(json: &str) -> Result<bodul_shared::NormalizedData, String> {
    let raw: raw::AnkerdeRawDocument = serde_json::from_str(json).map_err(|e| e.to_string())?;

    let normalized = AnkerdeNormalizedDocument::from_raw(raw)
        .map_err(|error: NormalizeError| error.to_string())?;

    let document_json = serde_json::to_string(&normalized).map_err(|e| e.to_string())?;

    Ok(bodul_shared::NormalizedData::just_document_json(
        document_json,
    ))
}

pub fn structuralize(json: &str) -> Result<StructuralizedData, String> {
    let normalized: AnkerdeNormalizedDocument =
        serde_json::from_str(json).map_err(|e| e.to_string())?;

    let document =
        AnkerDeDocument::from_normalized(normalized).map_err(|error| error.to_string())?;

    let document_json = serde_json::to_string(&document).map_err(|e| e.to_string())?;

    Ok(StructuralizedData::just_document_json(document_json))
}
