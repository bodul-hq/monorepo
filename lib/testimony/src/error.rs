//! Testimony error types

use std::fmt;

/// Error type for event store operations.
#[derive(Debug, Clone)]
pub enum EventStoreError {
    /// Failed to load events from storage
    LoadFailed(String),
    /// Failed to append events to storage
    AppendFailed(String),
    /// Events not found for the specified aggregate
    NotFound(String),
}

impl fmt::Display for EventStoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EventStoreError::LoadFailed(msg) => write!(f, "Failed to load events: {}", msg),
            EventStoreError::AppendFailed(msg) => write!(f, "Failed to append events: {}", msg),
            EventStoreError::NotFound(msg) => write!(f, "Events not found: {}", msg),
        }
    }
}

impl std::error::Error for EventStoreError {}

/// Error type for aggregate repository operations.
pub enum AggregateRepositoryError {
    // Variants to be defined as repository functionality is implemented
}

/// Error type for event publisher operations.
#[derive(Debug, Clone)]
pub enum EventPublisherError {
    /// Failed to publish events
    PublishFailed(String),
    /// Connection to the message broker failed
    ConnectionFailed(String),
    /// Serialization of events failed
    SerializationFailed(String),
}

impl fmt::Display for EventPublisherError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EventPublisherError::PublishFailed(msg) => {
                write!(f, "Failed to publish events: {}", msg)
            }
            EventPublisherError::ConnectionFailed(msg) => {
                write!(f, "Connection to message broker failed: {}", msg)
            }
            EventPublisherError::SerializationFailed(msg) => {
                write!(f, "Failed to serialize events: {}", msg)
            }
        }
    }
}

impl std::error::Error for EventPublisherError {}
