//! Extracts each offer page `dump-offer-details-v2` wrote to
//! `<data_dir>/<retailer>/offer_details_v2/sources/<page>.json` with its
//! retailer's raw-to-extracted conversion, into `offer_details_v2/extracted/<page>.json`.
//!
//! Usage: `extract-offer-details-v2 <data_dir> [retailer ...]`, where each
//! retailer is a folder name such as `ankerau`. With none given, every retailer
//! folder with sources is processed.

use offer_details::main as offer_details_main;

use development::offer_details::{self, Stage};
use retailer_data_extractor::extract_v2_for;
use shared::retailer::RetailerCode;

fn main() {
    offer_details_main(&Stage {
        name: "extract-offer-details-v2",
        directory: "offer_details_v2",
        input: "sources",
        input_extension: "json",
        output: "extracted",
        process: extract,
    });
}

fn extract(code: RetailerCode, json: &str) -> Result<Option<String>, String> {
    extract_v2_for(code, json)
        .map(Some)
        .map_err(|e| format!("extract: {e}"))
}
