//! Incoming offers events are commands in Mulac. Apply each original inbox
//! identity once, in the same transaction as its projection change.

use std::sync::Arc;

use bodul_shared::retailer::RetailerCode;
use chrono::{DateTime, Utc};
use commanding::io::{CommandError, ErasedCommandHandler, NewCommandEnvelope};
use diesel::{
    Connection, PgConnection, RunQueryDsl,
    sql_types::{BigInt, Text, Timestamptz, Uuid as SqlUuid},
};
use kernel::{
    NewEventEnvelope,
    io::{
        CommandConsumer, CommandConsumerRepository, CommandConsumerStorage, CommandDispatcher,
        DbPool, EventGateway, EventRecorder, EventRecorderRepository, EventStoreStorage,
    },
};
use money::Money;
use serde::Deserialize;
use uuid::Uuid;

#[derive(Deserialize)]
struct Created {
    id: Uuid,
    retailer_code: RetailerCode,
    title: String,
    price: Money,
    url: String,
    created_at: DateTime<Utc>,
}

#[derive(Deserialize)]
struct TitleChanged {
    id: Uuid,
    title: String,
}

#[derive(Deserialize)]
struct PriceChanged {
    id: Uuid,
    price: Money,
}

enum Change {
    Created(Created),
    Title(TitleChanged),
    Price(PriceChanged),
}

impl Change {
    fn parse(kind: &str, payload: &str) -> Result<Self, CommandError> {
        let payload: serde_json::Value = serde_json::from_str(payload).map_err(invalid)?;
        if !payload.is_object() {
            return Err(invalid("offer payload must be a JSON object"));
        }
        if matches!(kind, "OfferCreated" | "OfferPriceChangedEvent")
            && payload.get("price").is_some_and(|price| !price.is_object())
        {
            return Err(invalid("price must be a JSON object"));
        }
        let change = match kind {
            "OfferCreated" => Self::Created(serde_json::from_value(payload).map_err(invalid)?),
            "OfferTitleChanged" => Self::Title(serde_json::from_value(payload).map_err(invalid)?),
            "OfferPriceChangedEvent" => {
                Self::Price(serde_json::from_value(payload).map_err(invalid)?)
            }
            _ => return Err(CommandError::HandlerNotFound(kind.to_owned())),
        };
        let title = match &change {
            Self::Created(offer) => {
                if offer.url.len() > 2048 || offer.url.contains('\0') {
                    return Err(invalid("URL must be at most 2048 bytes and contain no NUL"));
                }
                Some(&offer.title)
            }
            Self::Title(change) => Some(&change.title),
            Self::Price(_) => None,
        };
        if title.is_some_and(|title| title.contains('\0')) {
            return Err(invalid("title must contain no NUL"));
        }
        Ok(change)
    }

    fn apply(&self, connection: &mut PgConnection) -> diesel::QueryResult<()> {
        let changed = match self {
            Self::Created(offer) => diesel::sql_query(
                "INSERT INTO offers (id, retailer_code, title, price_amount, currency, url, created_at)
                 VALUES ($1, $2, $3, $4, $5, $6, $7)",
            )
            .bind::<SqlUuid, _>(offer.id)
            .bind::<Text, _>(offer.retailer_code.as_str())
            .bind::<Text, _>(&offer.title)
            .bind::<BigInt, _>(offer.price.amount())
            .bind::<Text, _>(offer.price.currency().as_str())
            .bind::<Text, _>(&offer.url)
            .bind::<Timestamptz, _>(offer.created_at)
            .execute(connection)?,
            Self::Title(change) => diesel::sql_query("UPDATE offers SET title = $2 WHERE id = $1")
                .bind::<SqlUuid, _>(change.id)
                .bind::<Text, _>(&change.title)
                .execute(connection)?,
            Self::Price(change) => diesel::sql_query(
                "UPDATE offers SET price_amount = $2, currency = $3 WHERE id = $1",
            )
            .bind::<SqlUuid, _>(change.id)
            .bind::<BigInt, _>(change.price.amount())
            .bind::<Text, _>(change.price.currency().as_str())
            .execute(connection)?,
        };
        if changed == 0 {
            return Err(diesel::result::Error::NotFound);
        }
        Ok(())
    }
}

fn invalid(error: impl std::fmt::Display) -> CommandError {
    CommandError::Conversion(error.to_string())
}

pub(super) struct Projection(pub DbPool);

impl ErasedCommandHandler for Projection {
    fn execute(
        &self,
        envelope: &NewCommandEnvelope,
    ) -> Result<Vec<NewEventEnvelope>, CommandError> {
        let change = Change::parse(&envelope.command.command_type, &envelope.command.payload)?;
        let metadata = envelope
            .metadata
            .as_ref()
            .ok_or_else(|| invalid("missing inbox metadata"))?;
        let message_id = metadata
            .causation_id
            .ok_or_else(|| invalid("missing original message ID"))?;
        if metadata
            .source
            .as_ref()
            .is_none_or(|source| source.trim().is_empty())
        {
            return Err(invalid("missing or blank message source"));
        }
        let mut connection = self
            .0
            .get()
            .map_err(|_| CommandError::Storage("could not acquire projection connection".into()))?;
        connection
            .transaction::<_, diesel::result::Error, _>(|connection| {
                // The unique insert waits for a concurrent application of this ID.
                // A rolled-back application leaves no receipt and remains retryable.
                let claimed = diesel::sql_query(
                    "INSERT INTO applied_offer_messages (message_id) VALUES ($1)
                 ON CONFLICT (message_id) DO NOTHING",
                )
                .bind::<SqlUuid, _>(message_id)
                .execute(connection)?;
                if claimed != 0 {
                    change.apply(connection)?;
                }
                Ok(())
            })
            .map_err(|error| match error {
                diesel::result::Error::NotFound => {
                    CommandError::HandlerExecution("offer has not been created".into())
                }
                _ => CommandError::Storage(format!("offer projection failed: {error}")),
            })?;
        // Projection is complete at the command stage; no synthetic event needed.
        Ok(Vec::new())
    }
}

pub fn consumer(pool: DbPool) -> Arc<CommandConsumer> {
    let storage = Arc::new(CommandConsumerStorage::new(pool.clone()));
    let events = EventGateway::two_phased(Arc::new(EventRecorder::new(Arc::new(
        EventRecorderRepository::new(Arc::new(EventStoreStorage::new(pool.clone()))),
    ))));
    Arc::new(CommandConsumer::new(
        CommandConsumerRepository::new(storage.clone(), storage),
        Arc::new(CommandDispatcher::new(
            Arc::new(Projection(pool)),
            Arc::new(events),
        )),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn wire_validation_preserves_values_and_enforces_storage_boundaries() {
        let mut payload = json!({
            "id":Uuid::now_v7(),"retailer_code":"AnkerAu","title":" \tČaj 🔌\n",
            "price":{"amount":i64::MAX,"currency":"AUD"},
            "url":"é".repeat(1024),"created_at":"2026-10-24T14:00:00.123456+02:00"
        });
        let Change::Created(offer) = Change::parse("OfferCreated", &payload.to_string()).unwrap()
        else {
            panic!("expected creation")
        };
        assert_eq!(offer.price.amount(), i64::MAX);
        assert_eq!(offer.retailer_code.as_str(), "ankerau");
        assert_eq!(offer.title, " \tČaj 🔌\n");
        assert_eq!(offer.url.len(), 2048);
        assert_eq!(
            offer.created_at.to_rfc3339(),
            "2026-10-24T12:00:00.123456+00:00"
        );
        for (field, value) in [
            ("url", json!("é".repeat(1025))),
            ("url", json!("https://example.com/\0")),
            ("title", json!("\0")),
            ("retailer_code", json!("unknown")),
            ("created_at", json!("not a timestamp")),
            ("id", json!("not a UUID")),
        ] {
            let original = payload[field].clone();
            payload[field] = value;
            assert!(Change::parse("OfferCreated", &payload.to_string()).is_err());
            payload[field] = original;
        }
        for amount in [i64::MIN, 0, i64::MAX] {
            payload["price"]["amount"] = json!(amount);
            assert!(Change::parse("OfferCreated", &payload.to_string()).is_ok());
        }
        assert!(
            Change::parse(
                "OfferTitleChanged",
                &json!([Uuid::now_v7(), "title"]).to_string()
            )
            .is_err()
        );
        assert!(Change::parse("OfferCreated", "{").is_err());
        assert!(Change::parse("OfferPriceChanged", &payload.to_string()).is_err());
    }
}
