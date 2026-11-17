//! Version 1 of the offer details.

use crate::ankerjapancom::offer_details::v1::normalized::{
    AnkerJapanComNormalizedDocument, NormalizeError,
};
use crate::ankerjapancom::offer_details::v1::structuralize::AnkerJapanComDocument;
use html_extractor::io::{Architecture, Attr, collection, particle, trash};
use html_extractor::serde_json;
use raw::AnkerJapanComRawDocument;
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
            r#"script[id^="ProductJson-"]"#,
            "product_json",
            vec![Attr::json("", "data")],
        ),
    ])
}

pub fn normalize(json: &str) -> Result<NormalizedData, String> {
    let raw: AnkerJapanComRawDocument = from_str(json).map_err(|error| error.to_string())?;

    let normalized = AnkerJapanComNormalizedDocument::from_raw(raw)
        .map_err(|error: NormalizeError| error.to_string())?;

    let document_json = serde_json_to_string(&normalized).map_err(|error| error.to_string())?;

    Ok(NormalizedData::just_document_json(document_json))
}

pub fn structuralize(json: &str) -> Result<StructuralizedData, String> {
    let normalized: AnkerJapanComNormalizedDocument = from_str(json).map_err(|e| e.to_string())?;

    let document =
        AnkerJapanComDocument::from_normalized(normalized).map_err(|error| error.to_string())?;

    let document_json = serde_json_to_string(&document).map_err(|e| e.to_string())?;

    Ok(StructuralizedData::just_document_json(document_json))
}
