//! Event store module

use crate::error::EventStoreError;
use crate::event_envelope::{EventEnvelope, HasAggregateId};

/// EventStore trait defines the interface for persisting and retrieving domain events.
///
/// Generic over:
/// - `Id`: The aggregate's unique identifier type (e.g., RetailerId)
/// - `E`: The event type (e.g., RetailerEvent) - must carry an aggregate ID
///
/// All operations return Result types to support proper error handling
/// in event sourcing scenarios.
pub trait EventStore<Id, E: Clone + HasAggregateId> {
    /// Loads a list of event envelopes for an aggregate identified by `id`.
    ///
    /// # Arguments
    /// * `id` - The unique identifier of the aggregate
    ///
    /// # Returns
    /// - `Ok(Vec<EventEnvelope<E>>)` - A vector of EventEnvelopes containing all events
    ///   for the specified aggregate. Returns an empty vector if no events are found.
    /// - `Err(EventStoreError)` - If loading fails due to storage errors or other issues
    fn load(&self, id: Id) -> Result<Vec<EventEnvelope<E>>, EventStoreError>;

    /// Appends event envelopes to the event store.
    ///
    /// # Arguments
    /// * `events` - A vector of event envelopes to persist
    ///
    /// The event envelopes should contain all necessary information about
    /// which aggregate they belong to (via their event payload).
    ///
    /// # Returns
    /// - `Ok(())` - If events are successfully persisted
    /// - `Err(EventStoreError)` - If appending fails due to storage errors or other issues
    fn append(&self, events: Vec<EventEnvelope<E>>) -> Result<(), EventStoreError>;
}
