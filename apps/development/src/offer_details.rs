//! Runs one stage of the offer pipeline over the pages `dump-offer-details`
//! wrote: each file in `<data_dir>/<retailer>/offer_details/<input>/` becomes
//! `<data_dir>/<retailer>/offer_details/<output>/<page>.json`, pretty-printed
//! with its keys in the order the stage wrote them.
//!
//! A page that fails, or whose retailer has no such stage or yields no
//! document, gets no file, and any file an earlier run left for it is removed.
//! The rest of the pages carry on, and the failures are reported per retailer.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::{env, fs, process, thread};

use bodul_shared::retailer::RetailerCode;
use retailer_data_extractor::pretty_json;

/// Error messages are cut to this many characters before they are grouped.
const ERROR_KEY_LEN: usize = 120;

/// Error groups printed per retailer.
const SHOWN_ERRORS: usize = 5;

/// The deeply nested documents some storefronts render overflow the default
/// thread stack.
const STACK_SIZE: usize = 64 * 1024 * 1024;

/// One stage: where it reads from, where it writes to, and what it does to a
/// page.
pub struct Stage {
    /// The binary's name, for the usage message.
    pub name: &'static str,
    /// The folder under `offer_details` it reads.
    pub input: &'static str,
    /// The extension of the files it reads, such as `html`.
    pub input_extension: &'static str,
    /// The folder under `offer_details` it writes.
    pub output: &'static str,
    /// Turns a page's input into its output JSON text, or `None` when the
    /// retailer has no such stage or the page yields no document.
    pub process: fn(RetailerCode, &str) -> Result<Option<String>, String>,
}

/// Runs `stage` over the retailers named on the command line, or every
/// retailer with input when none are named, and exits with status 1 when any
/// page failed or 2 when the stage could not run at all.
pub fn main(stage: &Stage) {
    match run(stage) {
        Ok(true) => {}
        Ok(false) => process::exit(1),
        Err(error) => {
            eprintln!("error: {error}");
            process::exit(2);
        }
    }
}

/// Returns whether every page of every retailer passed.
fn run(stage: &Stage) -> Result<bool, String> {
    let mut args = env::args().skip(1);
    let data_dir = PathBuf::from(
        args.next()
            .ok_or_else(|| format!("usage: {} <data_dir> [retailer ...]", stage.name))?,
    );
    let wanted: Vec<String> = args.collect();

    let retailers = retailers(stage, &data_dir, &wanted)?;
    if retailers.is_empty() {
        return Err(format!(
            "no offer_details/{} folders in {}",
            stage.input,
            data_dir.display()
        ));
    }

    let reports: Vec<Report> = thread::scope(|scope| {
        let handles: Vec<_> = retailers
            .iter()
            .map(|(name, code, dir)| {
                thread::Builder::new()
                    .stack_size(STACK_SIZE)
                    .spawn_scoped(scope, move || process_retailer(stage, name, *code, dir))
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
        report.print(stage);
        all_ok &= report.errors.is_empty();
    }
    Ok(all_ok)
}

/// The retailer folders to process, with their codes and `offer_details`
/// folders, sorted by name.
fn retailers(
    stage: &Stage,
    data_dir: &Path,
    wanted: &[String],
) -> Result<Vec<(String, RetailerCode, PathBuf)>, String> {
    let names: Vec<String> = if wanted.is_empty() {
        let mut names: Vec<String> = fs::read_dir(data_dir)
            .map_err(|e| format!("{}: {e}", data_dir.display()))?
            .filter_map(|entry| entry.ok())
            .filter(|entry| {
                let input = entry.path().join("offer_details").join(stage.input);
                input.is_dir()
            })
            .filter_map(|entry| entry.file_name().into_string().ok())
            .collect();
        names.sort();
        names
    } else {
        wanted.to_vec()
    };

    let mut retailers = Vec::new();
    for name in names {
        let code = match RetailerCode::try_from(name.as_str()) {
            Ok(code) => code,
            Err(_) if wanted.is_empty() => {
                eprintln!("skipping {name}: not a retailer");
                continue;
            }
            Err(e) => return Err(format!("{name}: {e}")),
        };
        let dir = data_dir.join(&name).join("offer_details");
        if !dir.join(stage.input).is_dir() {
            return Err(format!("{}: no {} folder", dir.display(), stage.input));
        }
        retailers.push((name, code, dir));
    }
    Ok(retailers)
}

struct Report {
    name: String,
    pages: usize,
    written: usize,
    /// Pages for which the stage gave no document.
    without_document: usize,
    /// Pages that failed, grouped by the start of their error message.
    errors: BTreeMap<String, Vec<String>>,
}

impl Report {
    fn print(&self, stage: &Stage) {
        println!(
            "{}: {} pages, {} {}, {} without a document",
            self.name, self.pages, self.written, stage.output, self.without_document
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

fn process_retailer(stage: &Stage, name: &str, code: RetailerCode, dir: &Path) -> Report {
    let mut report = Report {
        name: name.to_string(),
        pages: 0,
        written: 0,
        without_document: 0,
        errors: BTreeMap::new(),
    };

    let output_dir = dir.join(stage.output);
    let inputs = fs::create_dir_all(&output_dir)
        .map_err(|e| format!("{}: {e}", output_dir.display()))
        .and_then(|()| files(&dir.join(stage.input), stage.input_extension));
    let inputs = match inputs {
        Ok(inputs) => inputs,
        Err(error) => {
            report.errors.entry(error).or_default().push(String::new());
            return report;
        }
    };

    for input in inputs {
        report.pages += 1;
        let stem = input
            .file_stem()
            .expect("a file name")
            .to_string_lossy()
            .into_owned();
        let output = output_dir.join(format!("{stem}.json"));

        match process_page(stage, code, &input, &output) {
            Ok(true) => {
                report.written += 1;
                continue;
            }
            Ok(false) => report.without_document += 1,
            Err(error) => report
                .errors
                .entry(error_key(&error))
                .or_default()
                .push(stem),
        }
        // Whatever an earlier run left for this page is stale.
        let _ = fs::remove_file(&output);
    }

    report
}

/// Returns whether the page has a document, written to `output`.
fn process_page(
    stage: &Stage,
    code: RetailerCode,
    input: &Path,
    output: &Path,
) -> Result<bool, String> {
    let text = fs::read_to_string(input).map_err(|e| format!("read: {e}"))?;
    let Some(json) = (stage.process)(code, &text)? else {
        return Ok(false);
    };
    serde_json::from_str::<serde_json::Value>(&json).map_err(|e| format!("output: {e}"))?;
    let mut pretty = pretty_json(&json);
    pretty.push('\n');
    fs::write(output, pretty).map_err(|e| format!("{}: {e}", output.display()))?;
    Ok(true)
}

/// The message without the position serde reports, so one cause found at
/// different positions in different pages counts once.
fn error_key(error: &str) -> String {
    let error = error
        .rfind(" at line ")
        .map_or(error, |position| &error[..position]);
    error.chars().take(ERROR_KEY_LEN).collect()
}

/// The files in `dir` with `extension`, sorted by name.
fn files(dir: &Path, extension: &str) -> Result<Vec<PathBuf>, String> {
    let mut files: Vec<PathBuf> = fs::read_dir(dir)
        .map_err(|e| format!("{}: {e}", dir.display()))?
        .filter_map(|entry| entry.ok().map(|entry| entry.path()))
        .filter(|path| path.extension().is_some_and(|ext| ext == extension))
        .collect();
    files.sort();
    Ok(files)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_dir(name: &str) -> PathBuf {
        let dir = env::temp_dir().join(format!("offer-details-{name}-{}", process::id()));
        let _ = fs::remove_dir_all(&dir);
        dir
    }

    fn echo(_: RetailerCode, text: &str) -> Result<Option<String>, String> {
        match text {
            "none" => Ok(None),
            "fail" => Err("failed".to_string()),
            text => Ok(Some(text.to_string())),
        }
    }

    const ECHO: Stage = Stage {
        name: "echo",
        input: "in",
        input_extension: "txt",
        output: "out",
        process: echo,
    };

    #[test]
    fn writes_documents_and_removes_stale_files() {
        let dir = temp_dir("stage");
        fs::create_dir_all(dir.join("in")).unwrap();
        fs::create_dir_all(dir.join("out")).unwrap();
        fs::write(dir.join("in/a.txt"), r#"{"b":1,"a":[]}"#).unwrap();
        fs::write(dir.join("in/b.txt"), "none").unwrap();
        fs::write(dir.join("in/c.txt"), "fail").unwrap();
        fs::write(dir.join("in/skip.html"), "{}").unwrap();
        fs::write(dir.join("out/b.json"), "stale").unwrap();
        fs::write(dir.join("out/c.json"), "stale").unwrap();

        let report = process_retailer(&ECHO, "ankerau", RetailerCode::AnkerAu, &dir);

        assert_eq!(report.pages, 3);
        assert_eq!(report.written, 1);
        assert_eq!(report.without_document, 1);
        assert_eq!(report.errors["failed"], ["c"]);
        assert_eq!(
            fs::read_to_string(dir.join("out/a.json")).unwrap(),
            "{\n  \"b\": 1,\n  \"a\": []\n}\n"
        );
        assert!(!dir.join("out/b.json").exists());
        assert!(!dir.join("out/c.json").exists());
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn invalid_json_is_an_error() {
        let dir = temp_dir("invalid");
        fs::create_dir_all(dir.join("in")).unwrap();
        fs::write(dir.join("in/a.txt"), "not json").unwrap();

        let report = process_retailer(&ECHO, "ankerau", RetailerCode::AnkerAu, &dir);

        assert_eq!(report.written, 0);
        assert!(report.errors.keys().all(|key| key.starts_with("output: ")));
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn groups_errors_without_their_position() {
        assert_eq!(
            error_key("normalize: missing field `x` at line 1 column 20"),
            "normalize: missing field `x`"
        );
    }
}
