//! # Testimony Infrastructure
//!
//! Infrastructure layer implementations for the Testimony event sourcing framework.
//!
//! This crate provides concrete implementations of the `EventStore` trait defined
//! in the core `testimony` crate:
//!
//! - [`InMemoryEventStore`] - Process-local, in-memory event store for testing
//! - [`PostgresEventStore`] - PostgreSQL-backed event store for production use
//!
//! ## Usage
//!
//! ```ignore
//! use testimony_infra::prelude::*;
//!
//! // For testing
//! let store = InMemoryEventStore::<MyId, MyEvent>::new();
//!
//! // For production (with Diesel connection pool)
//! let store = PostgresEventStore::<MyId, MyEvent>::new(pool);
//! ```
//!
//! ## Supporting Types
//!
//! - [`EventStoreRecord`] / [`NewEventStoreRecord`] - Database row models for the event_store table
//! - [`MetadataRecord`] - Database row model for event metadata (JSONB)
//! - [`DbPool`] - Type alias for Diesel's PostgreSQL connection pool
//!
//! [`InMemoryEventStore`]: crate::prelude::InMemoryEventStore
//! [`PostgresEventStore`]: crate::prelude::PostgresEventStore
//! [`EventStoreRecord`]: crate::prelude::EventStoreRecord
//! [`NewEventStoreRecord`]: crate::prelude::NewEventStoreRecord
//! [`MetadataRecord`]: crate::prelude::MetadataRecord
//! [`DbPool`]: crate::prelude::DbPool

pub mod prelude;

mod event_store_record;
mod in_memory_event_store;
mod metadata_record;
mod postgres_event_store;
