//! PostgreSQL-backed event store implementation.
//!
//! Provides a persistent EventStore implementation using PostgreSQL with Diesel ORM.
//! Events are stored in an `event_store` table and loaded/appended with proper
//! sequencing and validation.

use chrono::DateTime;
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::sql_types::{BigInt, Jsonb, Text, Timestamptz};
use serde::Serialize;
use serde::de::DeserializeOwned;
use std::cell::RefCell;
use std::marker::PhantomData;
use testimony::prelude::{EventEnvelope, EventStore, EventStoreError, HasAggregateId};

pub use crate::event_store_record::EventStoreRecord;
use crate::event_store_record::NewEventStoreRecord;

/// Type alias for a Diesel PostgreSQL connection pool.
pub type DbPool = Pool<ConnectionManager<PgConnection>>;

/// PostgreSQL-backed event store implementation.
///
/// Stores events in a PostgreSQL `event_store` table with the following schema:
/// - id: BIGSERIAL PRIMARY KEY
/// - agg_id: VARCHAR(255) NOT NULL
/// - sequence: BIGINT NOT NULL
/// - payload_type: VARCHAR(255) NOT NULL
/// - payload: JSONB NOT NULL
/// - metadata: JSONB NOT NULL
/// - recorded_on: TIMESTAMPTZ NOT NULL
///
/// # Type Parameters
/// - `Id`: The aggregate identifier type (must implement ToString)
/// - `E`: The domain event type (must implement Clone and HasAggregateId)
/// - `R`: The record type for serialization (must implement Serialize, DeserializeOwned, and conversions to/from E)
///
/// # Guarantees
/// - Events are loaded ordered by sequence
/// - Sequence ordering is validated on append (must be monotonically increasing)
/// - All events in an append batch must belong to the same aggregate
pub struct PostgresEventStore<Id, E, R> {
    pool: DbPool,
    table_name: String,
    _phantom: PhantomData<(Id, E, R)>,
}

impl<Id, E, R> PostgresEventStore<Id, E, R> {
    /// Creates a new PostgresEventStore with the given connection pool.
    ///
    /// Uses "event_store" as the default table name.
    pub fn new(pool: DbPool) -> Self {
        Self {
            pool,
            table_name: "event_store".to_string(),
            _phantom: PhantomData,
        }
    }

    /// Creates a new PostgresEventStore with a custom table name.
    pub fn with_table_name(pool: DbPool, table_name: impl Into<String>) -> Self {
        Self {
            pool,
            table_name: table_name.into(),
            _phantom: PhantomData,
        }
    }

    /// Returns the table name used by this event store.
    pub fn table_name(&self) -> &str {
        &self.table_name
    }
}

impl<Id, E, R> Clone for PostgresEventStore<Id, E, R> {
    fn clone(&self) -> Self {
        Self {
            pool: self.pool.clone(),
            table_name: self.table_name.clone(),
            _phantom: PhantomData,
        }
    }
}

impl<Id, E, R> std::fmt::Debug for PostgresEventStore<Id, E, R> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PostgresEventStore")
            .field("table_name", &self.table_name)
            .finish()
    }
}

/// Event store borrowing a caller-owned connection, including its active transaction.
///
/// The caller controls commit/rollback. The internal `RefCell` only adapts
/// `EventStore`'s shared-reference methods to Diesel's mutable connection API;
/// this scoped adapter must not be shared across threads.
pub struct ConnectionEventStore<'conn, Id, E, R> {
    conn: RefCell<&'conn mut PgConnection>,
    table_name: String,
    _phantom: PhantomData<(Id, E, R)>,
}

impl<'conn, Id, E, R> ConnectionEventStore<'conn, Id, E, R> {
    pub fn new(conn: &'conn mut PgConnection) -> Self {
        Self::with_table_name(conn, "event_store")
    }

    /// `table_name` must be a trusted SQL identifier, never user input.
    pub fn with_table_name(conn: &'conn mut PgConnection, table_name: impl Into<String>) -> Self {
        Self {
            conn: RefCell::new(conn),
            table_name: table_name.into(),
            _phantom: PhantomData,
        }
    }
}

impl<Id, E, R> EventStore<Id, E> for PostgresEventStore<Id, E, R>
where
    Id: ToString + Clone,
    E: Clone + HasAggregateId,
    R: Serialize + DeserializeOwned + HasAggregateId,
    for<'a> R: TryFrom<&'a E>,
    for<'a> <R as TryFrom<&'a E>>::Error: std::fmt::Display,
    E: TryFrom<R>,
    <E as TryFrom<R>>::Error: std::fmt::Display,
{
    fn load(&self, id: Id) -> Result<Vec<EventEnvelope<E>>, EventStoreError> {
        let mut conn = self
            .pool
            .get()
            .map_err(|e| EventStoreError::LoadFailed(e.to_string()))?;
        ConnectionEventStore::<Id, E, R>::with_table_name(&mut conn, &self.table_name).load(id)
    }

    fn append(&self, events: Vec<EventEnvelope<E>>) -> Result<(), EventStoreError> {
        if events.is_empty() {
            return Ok(());
        }
        let mut conn = self
            .pool
            .get()
            .map_err(|e| EventStoreError::AppendFailed(e.to_string()))?;
        ConnectionEventStore::<Id, E, R>::with_table_name(&mut conn, &self.table_name)
            .append(events)
    }
}

impl<Id, E, R> EventStore<Id, E> for ConnectionEventStore<'_, Id, E, R>
where
    Id: ToString + Clone,
    E: Clone + HasAggregateId,
    R: Serialize + DeserializeOwned + HasAggregateId,
    for<'a> R: TryFrom<&'a E>,
    for<'a> <R as TryFrom<&'a E>>::Error: std::fmt::Display,
    E: TryFrom<R>,
    <E as TryFrom<R>>::Error: std::fmt::Display,
{
    fn load(&self, id: Id) -> Result<Vec<EventEnvelope<E>>, EventStoreError> {
        let id_str = id.to_string();

        let mut conn = self.conn.borrow_mut();

        // Load events ordered by sequence
        let query = format!(
            "SELECT id, agg_id, sequence, payload_type, payload, metadata, recorded_on \
             FROM {} WHERE agg_id = $1 ORDER BY sequence ASC",
            self.table_name
        );

        let records: Vec<EventStoreRecord> = diesel::sql_query(query)
            .bind::<Text, _>(&id_str)
            .load(&mut **conn)
            .map_err(|e| EventStoreError::LoadFailed(format!("Query failed: {}", e)))?;

        let mut envelopes = Vec::with_capacity(records.len());

        for record in records {
            let event_record: R = serde_json::from_value(record.payload.clone()).map_err(|e| {
                EventStoreError::LoadFailed(format!(
                    "Failed to deserialize event {}: {}",
                    record.payload_type, e
                ))
            })?;

            let event: E = E::try_from(event_record).map_err(|e| {
                EventStoreError::LoadFailed(format!(
                    "Failed to convert event {}: {}",
                    record.payload_type, e
                ))
            })?;

            let envelope = EventEnvelope::with_timestamp(
                event,
                u64::try_from(record.sequence)
                    .map_err(|_| EventStoreError::LoadFailed("negative event sequence".into()))?,
                everyday::prelude::DateTime::from_timestamp(record.recorded_on.timestamp()),
            );

            envelopes.push(envelope);
        }

        Ok(envelopes)
    }

    fn append(&self, events: Vec<EventEnvelope<E>>) -> Result<(), EventStoreError> {
        if events.is_empty() {
            return Ok(());
        }

        // Validate events belong to the same aggregate
        if events.len() > 1 {
            let first_agg_id = events[0].aggregate_id();
            for event in &events[1..] {
                if event.aggregate_id() != first_agg_id {
                    return Err(EventStoreError::AppendFailed(
                        "All events must belong to the same aggregate".to_string(),
                    ));
                }
            }
        }

        // Validate sequences are monotonically increasing
        if events.len() > 1 {
            for i in 1..events.len() {
                if events[i].sequence() <= events[i - 1].sequence() {
                    return Err(EventStoreError::AppendFailed(format!(
                        "Event sequences must be monotonically increasing: {} followed by {}",
                        events[i - 1].sequence(),
                        events[i].sequence()
                    )));
                }
            }
        }

        let mut conn = self.conn.borrow_mut();

        // Get the type name for payload_type column
        let payload_type = std::any::type_name::<R>();

        for envelope in events {
            let event = envelope.event();
            let event_record: R = event.try_into().map_err(|e| {
                EventStoreError::AppendFailed(format!("Failed to convert event to record: {}", e))
            })?;
            let payload = serde_json::to_value(&event_record).map_err(|e| {
                EventStoreError::AppendFailed(format!("Failed to serialize event: {}", e))
            })?;

            let recorded_on =
                DateTime::from_timestamp(envelope.recorded_at().timestamp(), 0).unwrap_or_default();

            // Serialize envelope metadata for the metadata column
            let metadata = serde_json::json!({
                "sequence": envelope.sequence(),
                "recorded_at": envelope.recorded_at().timestamp(),
            });

            let record = NewEventStoreRecord::new(
                envelope.aggregate_id(),
                i64::try_from(envelope.sequence()).map_err(|_| {
                    EventStoreError::AppendFailed("event sequence exceeds BIGINT".into())
                })?,
                payload_type,
                payload,
                metadata,
                recorded_on,
            );

            let insert_query = format!(
                "INSERT INTO {} (agg_id, sequence, payload_type, payload, metadata, recorded_on) \
                 VALUES ($1, $2, $3, $4, $5, $6)",
                self.table_name
            );

            diesel::sql_query(insert_query)
                .bind::<Text, _>(&record.agg_id)
                .bind::<BigInt, _>(record.sequence)
                .bind::<Text, _>(&record.payload_type)
                .bind::<Jsonb, _>(&record.payload)
                .bind::<Jsonb, _>(&record.metadata)
                .bind::<Timestamptz, _>(record.recorded_on)
                .execute(&mut **conn)
                .map_err(|e| {
                    EventStoreError::AppendFailed(format!("Failed to insert event: {}", e))
                })?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use serde::{Deserialize, Serialize};
    use serde_json::json;

    // Test event type for unit tests
    #[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
    struct TestEvent {
        id: String,
        data: String,
    }

    impl HasAggregateId for TestEvent {
        fn aggregate_id(&self) -> String {
            self.id.clone()
        }
    }

    #[test]
    fn test_serde_roundtrip() {
        let event = TestEvent {
            id: "agg-1".to_string(),
            data: "test data".to_string(),
        };

        let payload = serde_json::to_value(&event).unwrap();
        let restored: TestEvent = serde_json::from_value(payload).unwrap();

        assert_eq!(event, restored);
    }

    #[test]
    fn test_event_store_record_to_event() {
        let record = EventStoreRecord::new(
            1,
            "agg-1",
            0,
            "TestEvent",
            json!({"id": "agg-1", "data": "hello"}),
            json!({}),
            Utc::now(),
        );

        let event: TestEvent = serde_json::from_value(record.payload).unwrap();

        assert_eq!(event.id, "agg-1");
        assert_eq!(event.data, "hello");
    }

    #[test]
    fn test_deserialize_wrong_structure_fails() {
        let result: Result<TestEvent, _> = serde_json::from_value(json!({"wrong": "structure"}));
        assert!(result.is_err());
    }
}
