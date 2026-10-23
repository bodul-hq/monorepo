//! Event sourced repository module

use crate::aggregate::Aggregate;
use crate::error::{EventPublisherError, EventStoreError};
use crate::event_envelope::HasAggregateId;
use crate::event_publisher::{EventPublisher, NoOpEventPublisher};
use crate::event_store::EventStore;

/// Error type for EventSourcedRepository operations.
#[derive(Debug, Clone)]
pub enum RepositoryError {
    /// Event store operation failed.
    EventStore(EventStoreError),
    /// Event publishing failed (after events were persisted).
    Publisher(EventPublisherError),
}

impl std::fmt::Display for RepositoryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RepositoryError::EventStore(e) => write!(f, "Event store error: {}", e),
            RepositoryError::Publisher(e) => write!(f, "Publisher error: {}", e),
        }
    }
}

impl std::error::Error for RepositoryError {}

impl From<EventStoreError> for RepositoryError {
    fn from(e: EventStoreError) -> Self {
        RepositoryError::EventStore(e)
    }
}

impl From<EventPublisherError> for RepositoryError {
    fn from(e: EventPublisherError) -> Self {
        RepositoryError::Publisher(e)
    }
}

/// EventSourcedRepository manages aggregate persistence using event sourcing.
///
/// Responsible for:
/// - **Loading**: Retrieving events from an EventStore and reconstructing the aggregate
/// - **Saving**: Extracting uncommitted events, persisting them to the EventStore,
///   and publishing them via EventPublisher
///
/// Generic over:
/// - `A`: The aggregate type (e.g., Retailer)
/// - `S`: The EventStore implementation
/// - `P`: The EventPublisher implementation (defaults to NoOpEventPublisher)
///
/// Uses dependency injection to accept an EventStore and EventPublisher.
/// This allows for flexible storage backends and publishing strategies.
pub struct EventSourcedRepository<
    A: Aggregate,
    S: EventStore<A::Id, A::Event>,
    P: EventPublisher<A::Event> = NoOpEventPublisher,
> where
    A::Event: HasAggregateId,
{
    event_store: S,
    event_publisher: P,
    _phantom_aggregate: std::marker::PhantomData<A>,
}

impl<A: Aggregate, S: EventStore<A::Id, A::Event>> EventSourcedRepository<A, S, NoOpEventPublisher>
where
    A::Event: HasAggregateId,
{
    /// Creates a new EventSourcedRepository with the given event store implementation
    /// and no event publisher.
    pub fn new(event_store: S) -> Self {
        EventSourcedRepository {
            event_store,
            event_publisher: NoOpEventPublisher,
            _phantom_aggregate: std::marker::PhantomData,
        }
    }
}

impl<A: Aggregate, S: EventStore<A::Id, A::Event>, P: EventPublisher<A::Event>>
    EventSourcedRepository<A, S, P>
where
    A::Event: HasAggregateId,
{
    /// Creates a new EventSourcedRepository with the given event store and event publisher.
    pub fn with_publisher(event_store: S, event_publisher: P) -> Self {
        EventSourcedRepository {
            event_store,
            event_publisher,
            _phantom_aggregate: std::marker::PhantomData,
        }
    }

    /// Loads and reconstructs an aggregate from the event store.
    ///
    /// # Arguments
    /// * `id` - The unique identifier of the aggregate
    ///
    /// # Returns
    /// - `Ok(Some(aggregate))` - The reconstructed aggregate with no uncommitted events
    /// - `Ok(None)` - If no events exist for this aggregate ID
    /// - `Err(RepositoryError)` - If the store operation fails
    pub fn load(&self, id: A::Id) -> Result<Option<A>, RepositoryError> {
        let mut events = self.event_store.load(id.clone())?;

        if events.is_empty() {
            return Ok(None);
        }

        // Sort events by sequence number to ensure correct ordering
        events.sort_by_key(|e| e.sequence());

        // Reconstruct aggregate from its complete event history
        Ok(Some(A::from_history(id, events)))
    }

    /// Saves an aggregate to the event store and publishes the events.
    ///
    /// Extracts the aggregate's uncommitted events, persists them, then publishes
    /// them via the EventPublisher. Events are only marked as committed after
    /// successful publishing.
    ///
    /// # Arguments
    /// * `aggregate` - The aggregate to save (must be mutable)
    ///
    /// # Returns
    /// - `Ok(())` - If events are successfully persisted, published, and marked committed
    /// - `Err(RepositoryError::EventStore)` - If persistence fails (uncommitted events remain)
    /// - `Err(RepositoryError::Publisher)` - If publishing fails after persistence
    ///   (events are persisted but uncommitted events remain for retry)
    pub fn save(&self, aggregate: &mut A) -> Result<(), RepositoryError> {
        let events = aggregate.uncommitted_events().to_vec();

        if events.is_empty() {
            return Ok(());
        }

        // Persist uncommitted events first
        self.event_store.append(events.clone())?;

        // Extract just the event payloads for publishing
        let event_payloads: Vec<A::Event> = events
            .into_iter()
            .map(|envelope| envelope.into_event())
            .collect();

        // Publish events to external consumers
        self.event_publisher.publish(&event_payloads)?;

        // Only mark events as committed after successful publish
        aggregate.mark_events_committed();

        Ok(())
    }
}
