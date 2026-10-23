//! Event envelope module

use everyday::prelude::DateTime;

/// Trait for events that carry an aggregate identifier.
pub trait HasAggregateId {
    /// Returns the aggregate identifier for this event.
    fn aggregate_id(&self) -> String;
}

/// EventEnvelope wraps a domain event with metadata (sequence number and timestamp).
///
/// Generic over `E` - the event type.
#[derive(Clone, Debug)]
pub struct EventEnvelope<E: Clone> {
    event: E,
    sequence: u64,
    recorded_at: DateTime,
}

impl<E: Clone> EventEnvelope<E> {
    /// Creates a new EventEnvelope with the given event and sequence number.
    /// The timestamp is automatically set to the current time.
    pub fn new(event: E, sequence: u64) -> Self {
        EventEnvelope {
            event,
            sequence,
            recorded_at: DateTime::now(),
        }
    }

    /// Creates a new EventEnvelope with a specific timestamp.
    /// Used when loading events from storage where the timestamp is already known.
    pub fn with_timestamp(event: E, sequence: u64, recorded_at: DateTime) -> Self {
        EventEnvelope {
            event,
            sequence,
            recorded_at,
        }
    }

    /// Returns a reference to the event.
    pub fn event(&self) -> &E {
        &self.event
    }

    /// Consumes the envelope and returns the event.
    pub fn into_event(self) -> E {
        self.event
    }

    /// Returns the sequence number.
    pub fn sequence(&self) -> u64 {
        self.sequence
    }

    /// Returns the timestamp when the event was recorded.
    pub fn recorded_at(&self) -> DateTime {
        self.recorded_at
    }
}

impl<E: Clone + HasAggregateId> EventEnvelope<E> {
    /// Extracts the aggregate ID from the enclosed event.
    pub fn aggregate_id(&self) -> String {
        self.event.aggregate_id()
    }
}
