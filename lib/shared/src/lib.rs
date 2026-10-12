/// Liveness response shared by HTTP and GraphQL adapters.
///
/// This reports that the process is running; it does not check dependencies.
pub fn health() -> &'static str {
    "ok"
}
