//! Regression coverage for structuralization errors and implemented but unregistered stages.
//! The saved normalized documents live outside the repository and are never modified here.

use std::path::Path;

use bodul_shared::prelude::RetailerCode;
use html_extractor::serde_json::{self, Value};
use retailer_data_extractor::structuralize_for;

#[test]
fn reported_storefronts_structuralize_every_saved_normalized_document() {
    let data = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../data");
    let mut failures = Vec::new();
    for slug in [
        "minisforumca",
        "minisforumkr",
        "minisforumru",
        "minisforumuk",
        "minisforumus",
        "ugreenca",
        "ugreende",
        "ugreenkr",
        "ugreennl",
        "ugreenuk",
        "ugreenus",
        "ugreennasjp",
        "ankerca",
        "ankerde",
        "ugreeneu",
        "ugreenfr",
        "ugreenin",
        "ugreennasau",
        "ugreennasca",
        "ugreennasde",
        "ugreennases",
    ] {
        let code = RetailerCode::from_slug(slug).unwrap();
        assert!(
            structuralize_for(code, "{}").is_some(),
            "{slug} must have a registered structuralizer even without the corpus"
        );
        let directory = data.join(slug).join("offer_details/normalized");
        if !directory.is_dir() {
            eprintln!("skipping: {} not found", directory.display());
            continue;
        }
        let mut paths: Vec<_> = std::fs::read_dir(&directory)
            .unwrap()
            .map(|entry| entry.unwrap().path())
            .filter(|path| path.extension().is_some_and(|ext| ext == "json"))
            .collect();
        paths.sort();
        assert!(!paths.is_empty(), "{} is empty", directory.display());
        let mut written = 0;
        for path in &paths {
            let input = std::fs::read_to_string(path).unwrap();
            match structuralize_for(code, &input).expect("registered structuralizer") {
                Ok(output) => match output.document_json {
                    Some(json) => {
                        let json: Value = serde_json::from_str(&json).unwrap();
                        assert!(
                            json["variations"].is_array(),
                            "{}: no variations array",
                            path.display()
                        );
                        written += 1;
                    }
                    None => {
                        failures.push(format!("{}: no structuralized document", path.display()))
                    }
                },
                Err(error) => failures.push(format!("{}: {error}", path.display())),
            }
        }
        eprintln!("{slug}: {} pages, {written} structuralized", paths.len());
    }
    assert!(
        failures.is_empty(),
        "{} failures:\n{}",
        failures.len(),
        failures.join("\n")
    );
}
