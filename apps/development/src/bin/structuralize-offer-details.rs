//! Structuralizes each page `normalize-offer-details` wrote to
//! `<data_dir>/<retailer>/offer_details/normalized/<page>.json`, into
//! `offer_details/structuralized/<page>.json`.
//!
//! Usage: `structuralize-offer-details <data_dir> [retailer ...]`, where each
//! retailer is a folder name such as `ankerau`. With none given, every retailer
//! folder with normalized pages is processed.

use bodul_shared::retailer::RetailerCode;
use development::offer_details::{self, Stage};
use retailer_data_extractor::structuralize_for;

fn main() {
    offer_details::main(&Stage {
        name: "structuralize-offer-details",
        input: "normalized",
        input_extension: "json",
        output: "structuralized",
        process: structuralize,
    });
}

/// `None` when the retailer has no structuralizer or the page no document.
fn structuralize(code: RetailerCode, json: &str) -> Result<Option<String>, String> {
    let Some(structuralized) = structuralize_for(code, json) else {
        return Ok(None);
    };
    let structuralized = structuralized.map_err(|e| format!("structuralize: {e}"))?;
    Ok(structuralized.document_json)
}
