//! Runs every saved page through the pipeline and writes each stage's output next to the
//! sources, one pretty-printed JSON file per page and stage:
//!
//! - `sources/<page>.html` is extracted into `extracted/<page>.json`,
//! - `extracted/<page>.json` is normalized into `normalized/<page>.json`.
//!
//! Each stage reads the files the previous stage wrote, so the files on disk are exactly what
//! the next stage was given.

use std::fs;
use std::path::{Path, PathBuf};

use html_extractor::Backend;
use html_extractor::serde_json::{self, Value};

use super::{architecture, normalize};

/// The saved pages and the stage folders, kept outside the repository next to the monorepo
/// checkout.
const OFFER_DETAILS: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../../data/minisforumfr/offer_details"
);

fn stage_dir(stage: &str) -> PathBuf {
    Path::new(OFFER_DETAILS).join(stage)
}

/// The files in `dir` with `extension`, sorted by name.
fn files(dir: &Path, extension: &str) -> Vec<PathBuf> {
    let mut files: Vec<PathBuf> = fs::read_dir(dir)
        .unwrap_or_else(|e| panic!("{}: {e}", dir.display()))
        .map(|entry| entry.expect("read dir entry").path())
        .filter(|path| path.extension().is_some_and(|ext| ext == extension))
        .collect();
    files.sort();
    files
}

fn read(path: &Path) -> String {
    fs::read_to_string(path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

/// Writes `json` pretty-printed to `<dir>/<stem of source>.json`.
fn write_pretty(dir: &Path, source: &Path, json: &str) {
    let value: Value =
        serde_json::from_str(json).unwrap_or_else(|e| panic!("{}: {e}", source.display()));
    let mut pretty = serde_json::to_string_pretty(&value).expect("a JSON value serializes");
    pretty.push('\n');

    let stem = source.file_stem().expect("a file name");
    let path = dir.join(stem).with_extension("json");
    fs::write(&path, pretty).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
}

/// Runs `stage` over every `extension` file in `from` and writes its output into `to`. Every
/// file is attempted, and the failures are reported together.
fn run_stage(
    from: &str,
    extension: &str,
    to: &str,
    stage: impl Fn(&str) -> Result<String, String>,
) {
    let (from, to) = (stage_dir(from), stage_dir(to));
    fs::create_dir_all(&to).unwrap_or_else(|e| panic!("{}: {e}", to.display()));

    let inputs = files(&from, extension);
    assert!(
        !inputs.is_empty(),
        "no .{extension} files in {}",
        from.display()
    );

    let failures: Vec<String> = inputs
        .iter()
        .filter_map(|input| match stage(&read(input)) {
            Ok(json) => {
                write_pretty(&to, input, &json);
                None
            }
            Err(e) => Some(format!("{}: {e}", input.display())),
        })
        .collect();
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn writes_every_stage_of_every_page() {
    if !stage_dir("sources").is_dir() {
        eprintln!("skipping: {} not found", stage_dir("sources").display());
        return;
    }
    let arch = architecture();

    run_stage("sources", "html", "extracted", |html| {
        Backend::Scraper
            .extract(html, &arch)
            .map(|extracted| extracted.json.to_string())
            .map_err(|e| e.to_string())
    });

    run_stage("extracted", "json", "normalized", |extracted| {
        normalize(extracted)?
            .document_json
            .ok_or_else(|| "no normalized document".to_string())
    });
}
