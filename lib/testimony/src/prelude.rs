//! Re-exports of commonly used testimony types.

pub use crate::aggregate::Aggregate;
pub use crate::error::{AggregateRepositoryError, EventPublisherError, EventStoreError};
pub use crate::event_envelope::{EventEnvelope, HasAggregateId};
pub use crate::event_publisher::{EventPublisher, NoOpEventPublisher};
pub use crate::event_sourced_repository::{EventSourcedRepository, RepositoryError};
pub use crate::event_store::EventStore;
pub use crate::metadata::Metadata;
pub use crate::query_bus::{QueryBus, QueryBusError};
pub use crate::query_handler::QueryHandler;
