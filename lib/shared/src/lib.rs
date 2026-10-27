pub mod images;

/// Liveness response shared by HTTP and GraphQL adapters.
///
/// This reports that the process is running; it does not check dependencies.
pub fn health() -> &'static str {
    "ok"
}

#[cfg(test)]
mod tests {
    use super::health;

    #[test]
    fn health_reports_ok() {
        assert_eq!(health(), "ok");
    }
}
