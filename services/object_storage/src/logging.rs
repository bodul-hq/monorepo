//! Stderr and daily rolling files, following the service logging convention.
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};

pub fn init() -> WorkerGuard {
    let directory = std::env::var("LOG_DIR").unwrap_or_else(|_| "logs".into());
    let (writer, guard) = tracing_appender::non_blocking(tracing_appender::rolling::daily(
        directory,
        "object_storage.log",
    ));
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    let _ = tracing_subscriber::registry()
        .with(filter)
        .with(fmt::layer().with_ansi(false).with_writer(std::io::stderr))
        .with(fmt::layer().with_ansi(false).with_writer(writer))
        .try_init();
    guard
}
