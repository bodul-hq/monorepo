//! Event publisher trait for publishing domain events after persistence.

use std::rc::Rc;
use std::sync::Arc;

use crate::error::EventPublisherError;

/// EventPublisher trait defines the interface for publishing domain events
/// after they have been persisted to the event store.
///
/// This enables the outbox pattern where events are first stored reliably,
/// then published to external systems (e.g., message brokers, event buses,
/// projectors).
///
/// # Type Parameters
/// - `E`: The event type to publish
pub trait EventPublisher<E> {
    /// Publishes a slice of events to external consumers.
    ///
    /// This is called by EventSourcedRepository after events are successfully
    /// appended to the event store.
    ///
    /// # Arguments
    /// * `events` - A slice of events to publish
    ///
    /// # Returns
    /// - `Ok(())` - If events are successfully published
    /// - `Err(EventPublisherError)` - If publishing fails
    fn publish(&self, events: &[E]) -> Result<(), EventPublisherError>;
}

/// A no-op event publisher that does nothing.
///
/// Use this when you don't need event publishing (e.g., in tests or when
/// projectors are updated via replay).
pub struct NoOpEventPublisher;

impl<E> EventPublisher<E> for NoOpEventPublisher {
    fn publish(&self, _events: &[E]) -> Result<(), EventPublisherError> {
        Ok(())
    }
}

/// Implement EventPublisher for Rc<T> where T implements EventPublisher.
impl<E, T: EventPublisher<E> + ?Sized> EventPublisher<E> for Rc<T> {
    fn publish(&self, events: &[E]) -> Result<(), EventPublisherError> {
        (**self).publish(events)
    }
}

/// Implement EventPublisher for Arc<T> where T implements EventPublisher.
impl<E, T: EventPublisher<E> + ?Sized> EventPublisher<E> for Arc<T> {
    fn publish(&self, events: &[E]) -> Result<(), EventPublisherError> {
        (**self).publish(events)
    }
}

/// Implement EventPublisher for Box<T> where T implements EventPublisher.
impl<E, T: EventPublisher<E> + ?Sized> EventPublisher<E> for Box<T> {
    fn publish(&self, events: &[E]) -> Result<(), EventPublisherError> {
        (**self).publish(events)
    }
}
