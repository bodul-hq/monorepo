//! Logging to stderr and to daily rolling files.
//!
//! [`init`] installs a `tracing` subscriber with three outputs:
//!
//! - stderr, for `make serve` and `docker logs`;
//! - `retailer_sourcing.log.YYYY-MM-DD` under the log folder, with the same lines;
//! - `timing.log.YYYY-MM-DD` under the log folder, with only the records on the
//!   [`TIMING_TARGET`] target: how long each fetch, command and event took.
//!
//! Timing records stay out of stderr and the main log so they don't drown it.
//!
//! The log folder is `LOG_DIR`, or `logs` in the working directory. The level
//! comes from `RUST_LOG` (for example `RUST_LOG=debug`), and defaults to `info`.

use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::filter::filter_fn;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{EnvFilter, Layer, fmt};

/// The `tracing` target for timing records, which go only to the timing log.
pub const TIMING_TARGET: &str = "timing";

const DEFAULT_LOG_DIR: &str = "logs";
const LOG_FILE_PREFIX: &str = "retailer_sourcing.log";
const TIMING_FILE_PREFIX: &str = "timing.log";
const DEFAULT_FILTER: &str = "info";

/// Keeps the file writers running. Hold it for the whole run; dropping it
/// flushes the files.
pub struct LogGuards {
    _main: WorkerGuard,
    _timing: WorkerGuard,
}

fn is_timing(metadata: &tracing::Metadata<'_>) -> bool {
    metadata.target() == TIMING_TARGET
}

/// Installs the logging subscriber. Call it once, first thing in `main`, and
/// keep the returned guards alive.
#[must_use]
pub fn init() -> LogGuards {
    let log_dir = std::env::var("LOG_DIR").unwrap_or_else(|_| DEFAULT_LOG_DIR.to_string());
    // If the folder can't be created, the file writers report it; stderr still
    // works.
    let _ = std::fs::create_dir_all(&log_dir);

    let (main_writer, main_guard) =
        tracing_appender::non_blocking(tracing_appender::rolling::daily(&log_dir, LOG_FILE_PREFIX));
    let (timing_writer, timing_guard) = tracing_appender::non_blocking(
        tracing_appender::rolling::daily(&log_dir, TIMING_FILE_PREFIX),
    );

    let filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(DEFAULT_FILTER));

    // Plain text everywhere: a span's fields are formatted once and shared by
    // every output, so colours on stderr would leak escape codes into the files.
    let stderr = fmt::layer()
        .with_ansi(false)
        .with_writer(std::io::stderr)
        .with_filter(filter_fn(|metadata| !is_timing(metadata)));
    let main_file = fmt::layer()
        .with_ansi(false)
        .with_writer(main_writer)
        .with_filter(filter_fn(|metadata| !is_timing(metadata)));
    let timing_file = fmt::layer()
        .with_ansi(false)
        .with_writer(timing_writer)
        .with_filter(filter_fn(is_timing));

    // `try_init` leaves an already installed subscriber in place instead of
    // panicking.
    let _ = tracing_subscriber::registry()
        .with(filter)
        .with(stderr)
        .with(main_file)
        .with(timing_file)
        .try_init();

    LogGuards {
        _main: main_guard,
        _timing: timing_guard,
    }
}
