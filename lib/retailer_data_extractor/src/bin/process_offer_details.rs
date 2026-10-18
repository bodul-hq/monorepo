//! Runs every saved offer page in `<data_dir>/<retailer>/offer_details/sources` through the
//! pipeline and writes each stage's output next to the sources, one pretty-printed JSON file per
//! page and stage:
//!
//! - `sources/<page>.html` is extracted into `extracted/<page>.json`,
//! - that is normalized into `normalized/<page>.json`,
//! - and that is structuralized into `structurized/<page>.json`.
//!
//! A page that fails a stage, or whose retailer has no such stage or yields no document, gets no
//! file for that stage or the ones after it, and any file left there by an earlier run is
//! removed. The rest of the pages carry on, and the failures are reported per retailer.
//!
//! Usage: `process_offer_details <data_dir> [retailer ...]`, where each retailer is a folder
//! name such as `ankerau`. With none given, every retailer folder in `data_dir` is processed.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::{env, fs, process, thread};

use bodul_shared::prelude::RetailerCode;
use html_extractor::serde_json::{self, Value};
use retailer_data_extractor::{extract_for, normalize_for, pretty_json, structuralize_for};

const STAGES: [&str; 3] = ["extracted", "normalized", "structurized"];

/// Error messages are cut to this many characters before they are grouped.
const ERROR_KEY_LEN: usize = 120;

/// Error groups printed per retailer.
const SHOWN_ERRORS: usize = 5;

/// The deeply nested documents some storefronts render overflow the default thread stack.
const STACK_SIZE: usize = 64 * 1024 * 1024;

fn main() {
    match run() {
        Ok(true) => {}
        Ok(false) => process::exit(1),
        Err(error) => {
            eprintln!("error: {error}");
            process::exit(2);
        }
    }
}

/// Returns whether every page of every retailer passed every stage it has.
fn run() -> Result<bool, String> {
    let mut args = env::args().skip(1);
    let data_dir = PathBuf::from(
        args.next()
            .ok_or("usage: process_offer_details <data_dir> [retailer ...]")?,
    );
    let wanted: Vec<String> = args.collect();

    let retailers = retailers(&data_dir, &wanted)?;
    if retailers.is_empty() {
        return Err(format!(
            "no offer_details/sources folders in {}",
            data_dir.display()
        ));
    }

    let reports: Vec<Report> = thread::scope(|scope| {
        let handles: Vec<_> = retailers
            .iter()
            .map(|(code, dir)| {
                thread::Builder::new()
                    .stack_size(STACK_SIZE)
                    .spawn_scoped(scope, move || process_retailer(*code, dir))
                    .expect("spawn a retailer thread")
            })
            .collect();
        handles
            .into_iter()
            .map(|handle| handle.join().expect("a retailer thread panicked"))
            .collect()
    });

    let mut all_ok = true;
    for report in &reports {
        report.print();
        all_ok &= report.errors.is_empty();
    }
    Ok(all_ok)
}

/// The retailer folders to process, with their codes, sorted by name.
fn retailers(data_dir: &Path, wanted: &[String]) -> Result<Vec<(RetailerCode, PathBuf)>, String> {
    let names: Vec<String> = if wanted.is_empty() {
        let mut names: Vec<String> = fs::read_dir(data_dir)
            .map_err(|e| format!("{}: {e}", data_dir.display()))?
            .filter_map(|entry| entry.ok())
            .filter(|entry| entry.path().join("offer_details/sources").is_dir())
            .filter_map(|entry| entry.file_name().into_string().ok())
            .collect();
        names.sort();
        names
    } else {
        wanted.to_vec()
    };

    names
        .into_iter()
        .filter_map(|name| {
            let Some(code) = RetailerCode::from_slug(&name) else {
                if wanted.is_empty() {
                    eprintln!("skipping {name}: not a retailer");
                    return None;
                }
                return Some(Err(format!("{name}: not a retailer")));
            };
            let dir = data_dir.join(&name).join("offer_details");
            if !dir.join("sources").is_dir() {
                return Some(Err(format!("{}: no sources folder", dir.display())));
            }
            Some(Ok((code, dir)))
        })
        .collect()
}

struct Report {
    name: String,
    pages: usize,
    /// Pages written per stage, in the order of [`STAGES`].
    written: [usize; 3],
    /// Pages that failed, grouped by the start of their error message.
    errors: BTreeMap<String, Vec<String>>,
}

impl Report {
    fn print(&self) {
        let [extracted, normalized, structurized] = self.written;
        println!(
            "{}: {} pages, {extracted} extracted, {normalized} normalized, {structurized} structurized",
            self.name, self.pages
        );
        for (error, pages) in self.errors.iter().take(SHOWN_ERRORS) {
            println!("    {}x {error} (e.g. {})", pages.len(), pages[0]);
        }
        if self.errors.len() > SHOWN_ERRORS {
            println!(
                "    ... and {} more kinds of error",
                self.errors.len() - SHOWN_ERRORS
            );
        }
    }
}

fn process_retailer(code: RetailerCode, dir: &Path) -> Report {
    let mut report = Report {
        name: code.slug(),
        pages: 0,
        written: [0; 3],
        errors: BTreeMap::new(),
    };

    for stage in STAGES {
        let stage_dir = dir.join(stage);
        if let Err(e) = fs::create_dir_all(&stage_dir) {
            report
                .errors
                .entry(format!("{}: {e}", stage_dir.display()))
                .or_default()
                .push(String::new());
            return report;
        }
    }

    let sources = match html_files(&dir.join("sources")) {
        Ok(sources) => sources,
        Err(e) => {
            report.errors.entry(e).or_default().push(String::new());
            return report;
        }
    };

    for source in sources {
        report.pages += 1;
        let stem = source
            .file_stem()
            .expect("a file name")
            .to_string_lossy()
            .into_owned();

        let written = process_page(code, dir, &source, &stem);
        for stage in 0..written.stages {
            report.written[stage] += 1;
        }
        // Whatever an earlier run left for the stages this one did not reach is stale.
        for stage in &STAGES[written.stages..] {
            let _ = fs::remove_file(dir.join(stage).join(&stem).with_extension("json"));
        }
        if let Some(error) = written.error {
            report
                .errors
                .entry(error_key(&error))
                .or_default()
                .push(stem);
        }
    }

    report
}

/// The message without the position serde reports, so one cause found at different positions
/// in different pages counts once.
fn error_key(error: &str) -> String {
    let error = error
        .rfind(" at line ")
        .map_or(error, |position| &error[..position]);
    error.chars().take(ERROR_KEY_LEN).collect()
}

struct Written {
    /// How many stages, from the first, have a file for this page.
    stages: usize,
    error: Option<String>,
}

fn process_page(code: RetailerCode, dir: &Path, source: &Path, stem: &str) -> Written {
    let mut stages = 0;
    let error = (|| {
        let html = fs::read_to_string(source).map_err(|e| format!("read: {e}"))?;

        let extracted = extract_for(code, &html).map_err(|e| format!("extract: {e}"))?;
        write_pretty(&dir.join("extracted"), stem, &extracted)?;
        stages += 1;

        let Some(normalized) = normalize_for(code, &extracted) else {
            return Ok(());
        };
        let normalized = normalized.map_err(|e| format!("normalize: {e}"))?;
        let Some(normalized) = normalized.document_json else {
            return Ok(());
        };
        write_pretty(&dir.join("normalized"), stem, &normalized)?;
        stages += 1;

        let Some(structuralized) = structuralize_for(code, &normalized) else {
            return Ok(());
        };
        let structuralized = structuralized.map_err(|e| format!("structuralize: {e}"))?;
        let Some(structuralized) = structuralized.document_json else {
            return Ok(());
        };
        write_pretty(&dir.join("structurized"), stem, &structuralized)?;
        stages += 1;

        Ok(())
    })()
    .err();

    Written { stages, error }
}

/// The `.html` files in `dir`, sorted by name.
fn html_files(dir: &Path) -> Result<Vec<PathBuf>, String> {
    let mut files: Vec<PathBuf> = fs::read_dir(dir)
        .map_err(|e| format!("{}: {e}", dir.display()))?
        .filter_map(|entry| entry.ok().map(|entry| entry.path()))
        .filter(|path| path.extension().is_some_and(|ext| ext == "html"))
        .collect();
    files.sort();
    Ok(files)
}

/// Writes `json` pretty-printed to `<dir>/<stem>.json`, its keys in the order they arrive.
fn write_pretty(dir: &Path, stem: &str, json: &str) -> Result<(), String> {
    serde_json::from_str::<Value>(json).map_err(|e| format!("output: {e}"))?;
    let mut pretty = pretty_json(json);
    pretty.push('\n');

    let path = dir.join(stem).with_extension("json");
    fs::write(&path, pretty).map_err(|e| format!("{}: {e}", path.display()))
}
