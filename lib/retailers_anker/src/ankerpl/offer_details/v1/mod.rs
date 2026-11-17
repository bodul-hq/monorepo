//! Version 1 of the offer details.

use crate::ankerpl::offer_details::v1::normalized::{AnkerPlNormalizedDocument, NormalizeError};
use crate::ankerpl::offer_details::v1::structuralize::AnkerPlDocument;
use html_extractor::io::{Architecture, Attr, collection, particle, trash};
use html_extractor::serde_json;
use raw::AnkerPlRawDocument;
use retailer_sauce::structuralized::StructuralizedData;
use serde_json::from_str;
use serde_json::to_string as serde_json_to_string;
use shared::NormalizedData;

pub mod normalized;
pub mod raw;
mod structuralize;

// #[cfg(test)]
// mod stage_files_tests;

pub fn architecture() -> Architecture {
    Architecture::new(vec![
        trash(r#"svg"#),
        collection(
            r#"script[type="application/ld+json"]"#,
            "ld-json",
            vec![particle("script", "", vec![Attr::json("", "data")])],
        ),
        particle(
            "script#__NEXT_DATA__",
            "next_data",
            vec![Attr::all(), Attr::json("", "data")],
        ),
    ])
}

pub fn normalize(json: &str) -> Result<NormalizedData, String> {
    let raw: AnkerPlRawDocument = from_str(json).map_err(|e| e.to_string())?;

    let normalized = AnkerPlNormalizedDocument::from_raw(raw)
        .map_err(|error: NormalizeError| error.to_string())?;

    let document_json = serde_json_to_string(&normalized).map_err(|e| e.to_string())?;

    Ok(NormalizedData::just_document_json(document_json))
}

pub fn structuralize(json: &str) -> Result<StructuralizedData, String> {
    let normalized: AnkerPlNormalizedDocument = from_str(json).map_err(|e| e.to_string())?;

    let document =
        AnkerPlDocument::from_normalized(normalized).map_err(|error| error.to_string())?;

    let document_json = serde_json_to_string(&document).map_err(|e| e.to_string())?;

    Ok(StructuralizedData::just_document_json(document_json))
}
