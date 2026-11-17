//! Version 1 of the offer details.

use html_extractor::io::{Architecture, trash};
use html_extractor::serde_json;
use raw::AnkerKrRawDocument;
use serde_json::from_str;
use shared::NormalizedData;

pub mod raw;

// #[cfg(test)]
// mod stage_files_tests;

pub fn architecture() -> Architecture {
    Architecture::new(vec![trash(r#"svg"#)])
}

/// A valid empty extraction succeeds without producing a normalized document.
pub fn normalize(json: &str) -> Result<NormalizedData, String> {
    from_str::<AnkerKrRawDocument>(json)
        .map(|_| NormalizedData::just_document_json(None))
        .map_err(|error| error.to_string())
}
