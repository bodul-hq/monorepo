//! Aggregate module

use crate::event_envelope::EventEnvelope;

/// Trait for aggregates that support event sourcing.
///
/// An aggregate is a domain object that:
/// - Has a stable identifier
/// - Evolves its state exclusively by applying domain events
/// - Tracks internally a list of uncommitted events since last save
pub trait Aggregate: Sized {
    /// The event type associated with this aggregate.
    type Event: Clone;
    /// The aggregate's identifier type.
    type Id: Clone;

    /// Reconstructs an aggregate from its complete event history.
    ///
    /// Called when loading an aggregate from the event store. The implementation
    /// should replay all events to reconstruct the current state.
    ///
    /// # Arguments
    /// * `id` - The aggregate's identifier
    /// * `events` - All events in the aggregate's history (in sequence order)
    ///
    /// # Returns
    /// The reconstructed aggregate with no uncommitted events
    fn from_history(id: Self::Id, events: Vec<EventEnvelope<Self::Event>>) -> Self;

    /// Returns a reference to the aggregate's uncommitted events.
    fn uncommitted_events(&self) -> &[EventEnvelope<Self::Event>];

    /// Clears the aggregate's uncommitted events after successful persistence.
    fn mark_events_committed(&mut self);
}
