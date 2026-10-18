//! Regressions for normalization failures observed in exported offer details. These tests
//! read external data only; inline schema tests continue to run when the checkout is absent.

use std::path::Path;

use bodul_shared::prelude::RetailerCode;
use retailer_data_extractor::normalize_for;

#[test]
fn reported_storefronts_normalize_the_saved_extractions() {
    let data = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../data");
    let mut failures = Vec::new();
    for slug in [
        "ankerau",
        "ankercom",
        "ankerde",
        "ankereu",
        "ankerpl",
        "ankerjapancom",
        "ugreenin",
        "ugreennaseu",
        "ugreennasfr",
        "ugreennasit",
        "ugreennasuk",
    ] {
        let directory = data.join(slug).join("offer_details/extracted");
        if !directory.is_dir() {
            eprintln!("skipping: {} not found", directory.display());
            continue;
        }
        let code = RetailerCode::from_slug(slug).unwrap();
        let mut paths: Vec<_> = std::fs::read_dir(&directory)
            .unwrap()
            .map(|entry| entry.unwrap().path())
            .filter(|path| path.extension().is_some_and(|ext| ext == "json"))
            .collect();
        paths.sort();
        assert!(!paths.is_empty(), "{} is empty", directory.display());
        let mut normalized = 0;
        let mut without_document = 0;
        for path in &paths {
            let input = std::fs::read_to_string(path).unwrap();
            match normalize_for(code, &input).expect("registered normalizer") {
                Ok(output) => match output.document_json {
                    Some(json) => {
                        assert!(html_extractor::serde_json::from_str::<html_extractor::serde_json::Value>(&json).unwrap().is_object());
                        normalized += 1;
                    }
                    None => {
                        // Only foreign pages collected by the old AU/US crawls are skipped.
                        let name = path.file_name().unwrap().to_str().unwrap();
                        assert!(
                            slug == "ankerau" || slug == "ankercom",
                            "{slug}: unexpected missing document"
                        );
                        assert!(
                            !name.starts_with("au-products-") && !name.starts_with("products-"),
                            "native page {} was skipped",
                            path.display()
                        );
                        without_document += 1;
                    }
                },
                Err(error) => failures.push(format!("{}: {error}", path.display())),
            }
        }
        eprintln!(
            "{slug}: {} pages, {normalized} normalized, {without_document} without a document",
            paths.len()
        );
    }
    assert!(
        failures.is_empty(),
        "{} failures:\n{}",
        failures.len(),
        failures.join("\n")
    );
}
