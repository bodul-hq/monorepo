//! In-memory event store implementation for testing and prototyping.
//!
//! This module provides a process-local, in-memory implementation of the EventStore trait.
//! All data is stored in memory and is lost when the process terminates.

use std::cell::RefCell;
use std::collections::HashMap;
use testimony::prelude::{EventEnvelope, EventStore, EventStoreError};

/// An in-memory implementation of the EventStore trait.
///
/// Stores event envelopes grouped by aggregate ID. Events are maintained in
/// ascending sequence order per aggregate. This implementation is suitable for
/// testing, prototyping, and simple workflows that don't require persistence.
///
/// Uses interior mutability (RefCell) to allow mutations through shared references,
/// which is necessary to implement the EventStore trait.
///
/// # Characteristics
/// - Process-local storage only (no persistence)
/// - No concurrency guarantees (not thread-safe for multi-threaded use)
/// - No snapshotting support
/// - Supports duplicate sequences (later appends may override earlier ones)
#[derive(Debug, Clone)]
pub struct InMemoryEventStore<Id, E: Clone> {
    /// Mapping from aggregate ID to ordered list of event envelopes
    /// Uses RefCell for interior mutability
    streams: std::rc::Rc<RefCell<HashMap<String, Vec<EventEnvelope<E>>>>>,
    _phantom_id: std::marker::PhantomData<Id>,
}

impl<Id, E: Clone> InMemoryEventStore<Id, E> {
    /// Creates a new empty InMemoryEventStore.
    pub fn new() -> Self {
        InMemoryEventStore {
            streams: std::rc::Rc::new(RefCell::new(HashMap::new())),
            _phantom_id: std::marker::PhantomData,
        }
    }

    /// Returns the number of aggregate streams in the store.
    pub fn stream_count(&self) -> usize {
        self.streams.borrow().len()
    }

    /// Returns the total number of events across all streams.
    pub fn total_events(&self) -> usize {
        self.streams
            .borrow()
            .values()
            .map(|stream| stream.len())
            .sum()
    }
}

impl<Id, E: Clone> Default for InMemoryEventStore<Id, E> {
    fn default() -> Self {
        Self::new()
    }
}

/// EventStore implementation for InMemoryEventStore.
///
/// The event payload (E) must implement HasAggregateId to provide the aggregate ID.
/// This implementation stores events by aggregate ID and maintains sequence ordering.
impl<Id: ToString + Clone, E: Clone + testimony::prelude::HasAggregateId> EventStore<Id, E>
    for InMemoryEventStore<Id, E>
{
    fn load(&self, id: Id) -> Result<Vec<EventEnvelope<E>>, EventStoreError> {
        let id_str = id.to_string();
        let streams = self.streams.borrow();
        match streams.get(&id_str) {
            Some(stream) => Ok(stream.clone()),
            None => Ok(Vec::new()),
        }
    }

    fn append(&self, events: Vec<EventEnvelope<E>>) -> Result<(), EventStoreError> {
        if events.is_empty() {
            return Ok(());
        }

        let mut streams = self.streams.borrow_mut();

        // Group events by aggregate ID and append to their respective streams
        for event in events {
            let aggregate_id = event.aggregate_id();
            let stream = streams.entry(aggregate_id).or_default();
            stream.push(event);
        }

        // Ensure all streams maintain sequence ordering
        for stream in streams.values_mut() {
            stream.sort_by_key(|envelope| envelope.sequence());
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Debug, PartialEq)]
    struct TestEvent {
        id: String,
        data: String,
    }

    impl testimony::prelude::HasAggregateId for TestEvent {
        fn aggregate_id(&self) -> String {
            self.id.clone()
        }
    }

    fn make_event(id: &str, data: &str) -> TestEvent {
        TestEvent {
            id: id.to_string(),
            data: data.to_string(),
        }
    }

    fn make_envelope(event: TestEvent, sequence: u64) -> EventEnvelope<TestEvent> {
        EventEnvelope::new(event, sequence)
    }

    // Basic unit tests
    #[test]
    fn test_new_store_is_empty() {
        let store: InMemoryEventStore<String, TestEvent> = InMemoryEventStore::new();
        assert_eq!(store.stream_count(), 0);
        assert_eq!(store.total_events(), 0);
    }

    #[test]
    fn test_default_creates_empty_store() {
        let store: InMemoryEventStore<String, TestEvent> = InMemoryEventStore::default();
        assert_eq!(store.stream_count(), 0);
    }

    #[test]
    fn test_load_nonexistent_aggregate_returns_empty_list() {
        let store: InMemoryEventStore<String, TestEvent> = InMemoryEventStore::new();
        let result = store.load("unknown-id".to_string());
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 0);
    }

    #[test]
    fn test_append_empty_list_is_noop() {
        let store: InMemoryEventStore<String, TestEvent> = InMemoryEventStore::new();
        let result = store.append(vec![]);
        assert!(result.is_ok());
        assert_eq!(store.stream_count(), 0);
    }

    // Acceptance criteria tests for EventSourcedRepository specification
    // See docs/technical/specifications/testimony_event_sourced_repository.md

    #[test]
    fn acceptance_load_returns_empty_for_unknown_aggregate() {
        // Given an EventStore with no events for ID "A"
        let store: InMemoryEventStore<String, TestEvent> = InMemoryEventStore::new();

        // When load(A) is called
        let result = store.load("A".to_string());

        // Then the result is an empty list (no error)
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[test]
    fn acceptance_load_returns_events_for_existing_aggregate() {
        // Given an EventStore with events [e1, e2, e3] for ID "A"
        let store: InMemoryEventStore<String, TestEvent> = InMemoryEventStore::new();
        let events = vec![
            make_envelope(make_event("A", "event1"), 1),
            make_envelope(make_event("A", "event2"), 2),
            make_envelope(make_event("A", "event3"), 3),
        ];
        store.append(events).unwrap();

        // When load(A) is called
        let result = store.load("A".to_string());

        // Then the result contains 3 events in sequence order
        assert!(result.is_ok());
        let loaded = result.unwrap();
        assert_eq!(loaded.len(), 3);
        assert_eq!(loaded[0].sequence(), 1);
        assert_eq!(loaded[1].sequence(), 2);
        assert_eq!(loaded[2].sequence(), 3);
        assert_eq!(loaded[0].event().data, "event1");
        assert_eq!(loaded[1].event().data, "event2");
        assert_eq!(loaded[2].event().data, "event3");
    }

    #[test]
    fn acceptance_append_stores_events_by_aggregate_id() {
        // Given events for different aggregates
        let store: InMemoryEventStore<String, TestEvent> = InMemoryEventStore::new();
        let events = vec![
            make_envelope(make_event("A", "a-event1"), 1),
            make_envelope(make_event("B", "b-event1"), 1),
            make_envelope(make_event("A", "a-event2"), 2),
        ];

        // When append is called
        store.append(events).unwrap();

        // Then events are partitioned by aggregate ID
        let a_events = store.load("A".to_string()).unwrap();
        let b_events = store.load("B".to_string()).unwrap();

        assert_eq!(a_events.len(), 2);
        assert_eq!(b_events.len(), 1);
        assert_eq!(a_events[0].event().data, "a-event1");
        assert_eq!(a_events[1].event().data, "a-event2");
        assert_eq!(b_events[0].event().data, "b-event1");
    }

    #[test]
    fn acceptance_events_are_sorted_by_sequence() {
        // Given events appended out of order
        let store: InMemoryEventStore<String, TestEvent> = InMemoryEventStore::new();
        let events = vec![
            make_envelope(make_event("A", "event3"), 3),
            make_envelope(make_event("A", "event1"), 1),
            make_envelope(make_event("A", "event2"), 2),
        ];

        // When append is called
        store.append(events).unwrap();

        // Then events are returned in sequence order
        let loaded = store.load("A".to_string()).unwrap();
        assert_eq!(loaded[0].sequence(), 1);
        assert_eq!(loaded[1].sequence(), 2);
        assert_eq!(loaded[2].sequence(), 3);
    }

    #[test]
    fn acceptance_multiple_appends_accumulate_events() {
        // Given two separate append operations
        let store: InMemoryEventStore<String, TestEvent> = InMemoryEventStore::new();

        // First append: events 1-2
        store
            .append(vec![
                make_envelope(make_event("A", "event1"), 1),
                make_envelope(make_event("A", "event2"), 2),
            ])
            .unwrap();

        // Second append: events 3-4
        store
            .append(vec![
                make_envelope(make_event("A", "event3"), 3),
                make_envelope(make_event("A", "event4"), 4),
            ])
            .unwrap();

        // Then all events are accumulated
        let loaded = store.load("A".to_string()).unwrap();
        assert_eq!(loaded.len(), 4);
        assert_eq!(loaded[0].event().data, "event1");
        assert_eq!(loaded[3].event().data, "event4");
    }

    #[test]
    fn acceptance_store_is_cloneable_and_shares_data() {
        // Given a store with events
        let store: InMemoryEventStore<String, TestEvent> = InMemoryEventStore::new();
        store
            .append(vec![make_envelope(make_event("A", "event1"), 1)])
            .unwrap();

        // When the store is cloned
        let cloned = store.clone();

        // Then the clone shares the same data
        assert_eq!(cloned.total_events(), 1);

        // And mutations through the clone are visible in the original
        cloned
            .append(vec![make_envelope(make_event("A", "event2"), 2)])
            .unwrap();
        assert_eq!(store.total_events(), 2);
    }
}
