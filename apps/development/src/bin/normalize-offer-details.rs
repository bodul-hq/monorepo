//! Normalizes each page `extract-offer-details` wrote to
//! `<data_dir>/<retailer>/offer_details/extracted/<page>.json`, into
//! `offer_details/normalized/<page>.json`.
//!
//! Usage: `normalize-offer-details <data_dir> [retailer ...]`, where each
//! retailer is a folder name such as `ankerau`. With none given, every retailer
//! folder with extracted pages is processed.

use bodul_shared::retailer::RetailerCode;
use development::offer_details::{self, Stage};
use retailer_data_extractor::normalize_for;

fn main() {
    offer_details::main(&Stage {
        name: "normalize-offer-details",
        input: "extracted",
        input_extension: "json",
        output: "normalized",
        process: normalize,
    });
}

/// `None` when the retailer has no normalizer or the page no document.
fn normalize(code: RetailerCode, json: &str) -> Result<Option<String>, String> {
    let Some(normalized) = normalize_for(code, json) else {
        return Ok(None);
    };
    let normalized = normalized.map_err(|e| format!("normalize: {e}"))?;
    Ok(normalized.document_json)
}
