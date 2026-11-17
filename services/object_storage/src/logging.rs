//! Stderr and daily rolling files, following the service logging convention.
use fmt::layer as fmt_layer;
use std::env::var;
use std::io::stderr;
use tracing_appender::non_blocking;
use tracing_appender::rolling::daily as tracing_appender_rolling_daily;
use tracing_subscriber::registry;

use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};

pub fn init() -> WorkerGuard {
    let directory = var("LOG_DIR").unwrap_or_else(|_| "logs".into());
    let (writer, guard) = non_blocking(tracing_appender_rolling_daily(
        directory,
        "object_storage.log",
    ));
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    let _ = registry()
        .with(filter)
        .with(fmt_layer().with_ansi(false).with_writer(stderr))
        .with(fmt_layer().with_ansi(false).with_writer(writer))
        .try_init();
    guard
}
