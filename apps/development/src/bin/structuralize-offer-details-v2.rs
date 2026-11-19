//! Structuralizes saved v2 extracted pages using the retailer's document mapping.

use development::offer_details::{Stage, main as run_stage};
use retailer_data_extractor::structuralize_v2_for;
use shared::retailer::RetailerCode;

fn main() {
    run_stage(&Stage {
        name: "structuralize-offer-details-v2",
        directory: "offer_details_v2",
        input: "extracted",
        input_extension: "json",
        output: "structuralized",
        process: structuralize,
    });
}

fn structuralize(code: RetailerCode, json: &str) -> Result<Option<String>, String> {
    structuralize_v2_for(code, json).map(Some)
}
