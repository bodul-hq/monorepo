//! Extracts each offer page `dump-offer-details` wrote to
//! `<data_dir>/<retailer>/offer_details/sources/<page>.html` with its
//! retailer's architecture, into `offer_details/extracted/<page>.json`.
//!
//! Usage: `extract-offer-details <data_dir> [retailer ...]`, where each
//! retailer is a folder name such as `ankerau`. With none given, every retailer
//! folder with sources is processed.

use bodul_shared::retailer::RetailerCode;
use development::offer_details::{self, Stage};
use retailer_data_extractor::extract_for;

fn main() {
    offer_details::main(&Stage {
        name: "extract-offer-details",
        input: "sources",
        input_extension: "html",
        output: "extracted",
        process: extract,
    });
}

fn extract(code: RetailerCode, html: &str) -> Result<Option<String>, String> {
    extract_for(code, html)
        .map(Some)
        .map_err(|e| format!("extract: {e}"))
}
