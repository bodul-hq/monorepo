//! Query handler trait for read-only query operations.
//!
//! Query handlers execute read-only queries and return results.
//! Each handler handles exactly one query type and must not modify domain state.

/// A handler for a specific query type.
///
/// Query handlers are responsible for:
/// - Validating incoming query parameters
/// - Fetching necessary data from read models or projections
/// - Transforming data into a result shape suitable for the caller
///
/// Query handlers must NOT:
/// - Mutate domain state
/// - Publish domain events
/// - Orchestrate other commands or queries
///
/// # Type Parameters
///
/// - `Q`: The query type this handler processes
///
/// # Associated Types
///
/// - `Result`: The successful result type returned by the handler
/// - `Error`: The error type returned when the query fails
pub trait QueryHandler<Q> {
    /// The successful result type returned by this handler.
    type Result;

    /// The error type returned when the query fails.
    type Error;

    /// Handles the query and returns a result.
    ///
    /// This method must be read-only and should not modify any domain state.
    /// It may read from databases, projections, or read models.
    ///
    /// # Arguments
    ///
    /// * `query` - The query to handle
    ///
    /// # Returns
    ///
    /// * `Ok(Self::Result)` - The query result on success
    /// * `Err(Self::Error)` - An error if the query fails (e.g., not found, validation error)
    fn handle(&self, query: Q) -> Result<Self::Result, Self::Error>;
}
