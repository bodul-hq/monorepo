//! Records outgoing messages in Mulac's outbox for the publisher to deliver.
//!
//! A message's ID is its row ID and delivery identity. Recording is
//! idempotent: an existing row is never changed, so a repeated recording keeps
//! the first metadata, status, attempts and reservation. Reusing an ID for
//! different content fails instead.

use diesel::prelude::*;
use diesel::sql_types::{Array, Integer, Jsonb, Text, Uuid as SqlUuid};
use outbox::io::{OutboxEntryMetadata, OutboxStatus};
use uuid::Uuid;

use super::bus::EventContext;
use super::db;

const CONTENT_TYPE: &str = "application/json";

/// One message to record, with the event that caused it.
#[derive(Debug, Clone, PartialEq)]
pub struct OutgoingMessage {
    /// Stable delivery identity, used as both row and message ID.
    pub id: Uuid,
    /// A JSON object, stored as this exact text.
    pub payload: String,
    pub routing_key: &'static str,
    pub event_type: &'static str,
}

#[derive(Debug, thiserror::Error)]
pub enum RecordError {
    #[error("database error: {}", db::describe(.0))]
    Database(#[from] diesel::result::Error),
    #[error("outbox message {0} payload is not a JSON object")]
    InvalidPayload(Uuid),
    #[error("outbox message {0} is listed more than once")]
    Duplicate(Uuid),
    #[error("outbox message {0} already exists with different content")]
    Conflict(Uuid),
    #[error("outbox message {0} was not recorded")]
    Missing(Uuid),
}

#[derive(diesel::QueryableByName)]
struct ExistingRow {
    #[diesel(sql_type = SqlUuid)]
    id: Uuid,
    #[diesel(sql_type = Text)]
    payload: String,
    #[diesel(sql_type = Jsonb)]
    meta: serde_json::Value,
}

/// Records every message in one transaction, or none of them.
pub fn record(
    conn: &mut PgConnection,
    messages: &[OutgoingMessage],
    context: &EventContext,
) -> Result<(), RecordError> {
    if messages.is_empty() {
        return Ok(());
    }
    let mut messages: Vec<&OutgoingMessage> = messages.iter().collect();
    // A consistent insertion order keeps concurrent recordings from deadlocking.
    messages.sort_by_key(|message| message.id);
    let mut payloads = Vec::with_capacity(messages.len());
    for (index, message) in messages.iter().enumerate() {
        if index > 0 && messages[index - 1].id == message.id {
            return Err(RecordError::Duplicate(message.id));
        }
        payloads.push(parse_object(message.id, &message.payload)?);
    }
    let ids: Vec<Uuid> = messages.iter().map(|message| message.id).collect();
    let metas = messages
        .iter()
        .map(|message| serde_json::to_value(metadata(message, context)))
        .collect::<Result<Vec<_>, _>>()
        .expect("outbox metadata serializes");

    conn.transaction(|conn| {
        diesel::sql_query(
            "INSERT INTO outbox_entries (id, status, payload, meta)
             SELECT id, $2, payload, meta FROM unnest($1, $3, $4) AS m(id, payload, meta)
             ORDER BY id
             ON CONFLICT (id) DO NOTHING",
        )
        .bind::<Array<SqlUuid>, _>(&ids)
        .bind::<Integer, _>(i32::from(OutboxStatus::Received))
        .bind::<Array<Text>, _>(
            messages
                .iter()
                .map(|m| m.payload.as_str())
                .collect::<Vec<_>>(),
        )
        .bind::<Array<Jsonb>, _>(&metas)
        .execute(conn)?;

        let existing: Vec<ExistingRow> = diesel::sql_query(
            "SELECT id, payload, meta FROM outbox_entries WHERE id = ANY($1) ORDER BY id",
        )
        .bind::<Array<SqlUuid>, _>(&ids)
        .load(conn)?;
        for ((message, payload), row) in messages.iter().zip(&payloads).zip(existing.iter()) {
            if row.id != message.id {
                return Err(RecordError::Missing(message.id));
            }
            if !same_content(message, payload, row) {
                return Err(RecordError::Conflict(message.id));
            }
        }
        match ids.get(existing.len()) {
            Some(&id) => Err(RecordError::Missing(id)),
            None => Ok(()),
        }
    })
}

fn metadata(message: &OutgoingMessage, context: &EventContext) -> OutboxEntryMetadata {
    OutboxEntryMetadata {
        event_id: context.event_id,
        message_id: message.id,
        correlation_id: context.correlation_id,
        causation_id: context.causation_id,
        event_type: message.event_type.to_string(),
        routing_key: message.routing_key.to_string(),
        source: context.source.clone(),
        content_type: Some(CONTENT_TYPE.to_string()),
    }
}

fn parse_object(id: Uuid, payload: &str) -> Result<serde_json::Value, RecordError> {
    match serde_json::from_str(payload) {
        Ok(value @ serde_json::Value::Object(_)) => Ok(value),
        _ => Err(RecordError::InvalidPayload(id)),
    }
}

/// The delivery content must match; the causing event's metadata may differ,
/// since a repeated notification keeps the first one recorded.
fn same_content(message: &OutgoingMessage, payload: &serde_json::Value, row: &ExistingRow) -> bool {
    let Ok(meta) = serde_json::from_value::<OutboxEntryMetadata>(row.meta.clone()) else {
        return false;
    };
    serde_json::from_str::<serde_json::Value>(&row.payload)
        .ok()
        .as_ref()
        == Some(payload)
        && meta.message_id == message.id
        && meta.routing_key == message.routing_key
        && meta.event_type == message.event_type
        && meta.content_type.as_deref() == Some(CONTENT_TYPE)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn message(payload: &str) -> OutgoingMessage {
        OutgoingMessage {
            id: Uuid::now_v7(),
            payload: payload.to_string(),
            routing_key: "offers",
            event_type: "OfferExportWasPrepared",
        }
    }

    #[test]
    fn metadata_combines_the_message_and_its_causing_event() {
        let message = message("{}");
        let context = EventContext {
            event_id: Uuid::now_v7(),
            correlation_id: Some(Uuid::now_v7()),
            causation_id: None,
            source: Some("event:OfferExportWasPrepared".into()),
        };

        assert_eq!(
            metadata(&message, &context),
            OutboxEntryMetadata {
                event_id: context.event_id,
                message_id: message.id,
                correlation_id: context.correlation_id,
                causation_id: None,
                event_type: "OfferExportWasPrepared".into(),
                routing_key: "offers".into(),
                source: context.source.clone(),
                content_type: Some("application/json".into()),
            }
        );
    }

    #[test]
    fn payloads_must_be_json_objects() {
        for payload in ["[]", "1", "not json", ""] {
            let message = message(payload);
            assert!(matches!(
                parse_object(message.id, &message.payload),
                Err(RecordError::InvalidPayload(id)) if id == message.id
            ));
        }
        assert!(parse_object(Uuid::now_v7(), r#"{"a":1}"#).is_ok());
    }

    #[test]
    fn content_comparison_ignores_event_metadata_but_not_delivery_content() {
        let message = message(r#"{"title":"a","price":{"amount":1}}"#);
        let payload = parse_object(message.id, &message.payload).unwrap();
        let other_event = EventContext {
            event_id: Uuid::now_v7(),
            correlation_id: None,
            causation_id: None,
            source: None,
        };
        let row = |payload: &str, meta: OutboxEntryMetadata| ExistingRow {
            id: message.id,
            payload: payload.to_string(),
            meta: serde_json::to_value(meta).unwrap(),
        };
        let meta = metadata(&message, &other_event);

        assert!(same_content(
            &message,
            &payload,
            &row(r#"{"price":{"amount":1}, "title":"a"}"#, meta.clone())
        ));
        assert!(!same_content(
            &message,
            &payload,
            &row(r#"{"title":"b","price":{"amount":1}}"#, meta.clone())
        ));
        let mut rerouted = meta.clone();
        rerouted.routing_key = "elsewhere".into();
        assert!(!same_content(
            &message,
            &payload,
            &row(&message.payload, rerouted)
        ));
        let mut renamed = meta;
        renamed.message_id = Uuid::now_v7();
        assert!(!same_content(
            &message,
            &payload,
            &row(&message.payload, renamed)
        ));
    }
}
