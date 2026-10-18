//! Version 1 of the offer details.

use crate::ankeritalycom::offer_details::v1::normalized::{
    AnkerItalyComNormalizedDocument, NormalizeError,
};
use crate::ankeritalycom::offer_details::v1::structuralize::AnkerItalyComDocument;
use html_extractor::io::{Architecture, Attr, collection, particle, trash};
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

pub fn normalize(json: &str) -> Result<bodul_shared::NormalizedData, String> {
    let raw: raw::AnkerItalyComRawDocument =
        serde_json::from_str(json).map_err(|e| e.to_string())?;

    let normalized = AnkerItalyComNormalizedDocument::from_raw(raw)
        .map_err(|error: NormalizeError| error.to_string())?;

    let document_json = serde_json::to_string(&normalized).map_err(|e| e.to_string())?;

    Ok(bodul_shared::NormalizedData::just_document_json(
        document_json,
    ))
}

pub fn structuralize(json: &str) -> Result<StructuralizedData, String> {
    let normalized: AnkerItalyComNormalizedDocument =
        serde_json::from_str(json).map_err(|e| e.to_string())?;

    let document =
        AnkerItalyComDocument::from_normalized(normalized).map_err(|error| error.to_string())?;

    let document_json = serde_json::to_string(&document).map_err(|e| e.to_string())?;

    Ok(StructuralizedData::just_document_json(document_json))
}

#[cfg(test)]
mod tests {
    use super::{normalize, structuralize};
    use std::path::Path;

    #[test]
    fn normalizes_and_structuralizes_saved_extractions() {
        let directory = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../../data/ankeritalycom/offer_details/extracted");
        if !directory.is_dir() {
            eprintln!("skipping: {} not found", directory.display());
            return;
        }
        let mut pages = 0;
        let mut failures = Vec::new();
        for entry in std::fs::read_dir(&directory).unwrap() {
            let path = entry.unwrap().path();
            if !path
                .extension()
                .is_some_and(|extension| extension == "json")
            {
                continue;
            }
            pages += 1;
            let input = std::fs::read_to_string(&path).unwrap();
            let result = normalize(&input).and_then(|normalized| {
                let document = normalized.document_json.ok_or("no normalized document")?;
                let output = structuralize(&document)?;
                output
                    .document_json
                    .ok_or_else(|| "no structuralized document".to_string())
            });
            if let Err(error) = result {
                failures.push(format!("{}: {error}", path.display()));
            }
        }
        assert!(pages > 0, "{} is empty", directory.display());
        assert!(failures.is_empty(), "{}", failures.join("\n"));
        eprintln!("ankeritalycom: {pages} pages normalized and structuralized");
    }
}
