//! Read-only survey; never writes the user's retailer corpus.
use bodul_shared::prelude::RetailerCode;
use retailer_data_extractor::{extract_for, normalize_for, structuralize_for};
use std::{collections::BTreeMap, path::PathBuf};

#[test]
fn survey_saved_source_evidence_and_facade_pipeline() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../data");
    if !root.is_dir() {
        eprintln!("saved source corpus unavailable; portable fixtures still run");
        return;
    }
    let mut report = BTreeMap::new();
    for folder in std::fs::read_dir(root).unwrap().flatten() {
        let slug = folder.file_name().to_string_lossy().into_owned();
        let Ok(code) = RetailerCode::try_from(slug.as_str()) else {
            continue;
        };
        let sources = folder.path().join("offer_details/sources");
        let Ok(files) = std::fs::read_dir(sources) else {
            continue;
        };
        let counts = report.entry(slug).or_insert([0usize; 5]);
        for file in files
            .flatten()
            .filter(|file| file.path().extension().is_some_and(|ext| ext == "html"))
        {
            counts[0] += 1;
            let html = std::fs::read_to_string(file.path()).unwrap();
            let Ok(extracted) = extract_for(code, &html) else {
                counts[4] += 1;
                continue;
            };
            let raw: serde_json::Value = serde_json::from_str(&extracted).unwrap();
            let selected = &raw["_localization_evidence"]["selected"];
            counts[1] += usize::from(!selected["target_market"].is_null());
            counts[2] += usize::from(!selected["language"].is_null());
            let Some(Ok(normalized)) = normalize_for(code, &extracted) else {
                counts[4] += 1;
                continue;
            };
            let Some(document) = normalized.document_json else {
                continue;
            };
            let Some(Ok(structuralized)) = structuralize_for(code, &document) else {
                counts[4] += 1;
                continue;
            };
            if let Some(document) = structuralized.document_json {
                let output: serde_json::Value = serde_json::from_str(&document).unwrap();
                assert_eq!(&output["_localization_evidence"]["selected"], selected);
                for variation in output["variations"].as_array().into_iter().flatten() {
                    assert_eq!(&variation["localization"], selected);
                }
                counts[3] += 1;
            }
        }
    }
    for (slug, counts) in report {
        println!(
            "SURVEY|{slug}|{}|{}|{}|{}|{}",
            counts[0], counts[1], counts[2], counts[3], counts[4]
        );
    }
}
