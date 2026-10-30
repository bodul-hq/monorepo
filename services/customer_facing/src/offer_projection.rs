//! Incoming offers events are commands in Mulac. Apply each original inbox
//! identity once, in the same transaction as its projection change.

use crate::identifier::Identifier;
use std::sync::Arc;

use bodul_shared::retailer::RetailerCode;
use chrono::{DateTime, Utc};
use commanding::io::{CommandError, ErasedCommandHandler, NewCommandEnvelope};
use diesel::{
    Connection, PgConnection, RunQueryDsl,
    sql_types::{BigInt, Jsonb, Text, Timestamptz, Uuid as SqlUuid},
};
use kernel::{
    NewEventEnvelope,
    io::{
        CommandConsumer, CommandConsumerRepository, CommandConsumerStorage, CommandDispatcher,
        DbPool, EventGateway, EventRecorder, EventRecorderRepository, EventStoreStorage,
    },
};
use money::Money;
use retailer_sauce::{
    availability::Availability, item_condition::ItemCondition, offer_content,
    structuralized::content::Content,
};
use serde::Deserialize;
use uuid::Uuid;

mod content;

#[derive(Deserialize)]
struct Created {
    #[serde(default)]
    was_price: Option<Money>,
    #[serde(default)]
    availability: Option<Availability>,
    #[serde(default)]
    item_condition: Option<ItemCondition>,
    content: Vec<offer_content::Content>,
    #[serde(flatten)]
    identifiers: retailer_sauce::identifier::ProductIdentifiers,
    #[serde(default)]
    localization: retailer_sauce::localization::Localization,
    #[serde(default)]
    scraped_at: Option<DateTime<Utc>>,
    #[serde(default)]
    images: Vec<String>,
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

#[derive(Deserialize)]
struct Pricing {
    price: Money,
    #[serde(deserialize_with = "retailer_sauce::observation::required_nullable")]
    was_price: Option<Money>,
}

#[derive(Deserialize)]
struct PricingChanged {
    id: Uuid,
    pricing: Pricing,
}

#[derive(Deserialize)]
struct AvailabilityChanged {
    id: Uuid,
    #[serde(deserialize_with = "retailer_sauce::observation::required_nullable")]
    availability: Option<Availability>,
}

#[derive(Deserialize)]
struct ItemConditionChanged {
    id: Uuid,
    #[serde(deserialize_with = "retailer_sauce::observation::required_nullable")]
    item_condition: Option<ItemCondition>,
}

#[derive(Deserialize)]
struct ContentChanged {
    id: Uuid,
    content: Content,
}

#[derive(Deserialize)]
struct ImageAdded {
    id: Uuid,
    image_url: String,
    position: u32,
}

#[derive(Deserialize)]
struct ScrapedAtUpdated {
    id: Uuid,
    scraped_at: DateTime<Utc>,
}

#[derive(Deserialize)]
struct LocalizationChanged {
    id: Uuid,
    localization: retailer_sauce::localization::Localization,
}

#[derive(Deserialize)]
struct IdentifiersChanged {
    id: Uuid,
    #[serde(
        flatten,
        deserialize_with = "retailer_sauce::identifier::complete_identifiers"
    )]
    identifiers: retailer_sauce::identifier::ProductIdentifiers,
}

#[derive(Deserialize)]
struct ContentsChanged {
    id: Uuid,
    content: Vec<offer_content::Content>,
    #[serde(deserialize_with = "retailer_sauce::observation::required_nullable")]
    language: Option<String>,
}

enum Change {
    Contents(ContentsChanged),
    Pricing(PricingChanged),
    Availability(AvailabilityChanged),
    ItemCondition(ItemConditionChanged),
    Content(ContentChanged),
    Identifiers(IdentifiersChanged),
    Localization(LocalizationChanged),
    ScrapedAt(ScrapedAtUpdated),
    Image(ImageAdded),
    Created(Box<Created>),
    Title(TitleChanged),
    Price(PriceChanged),
}

impl Change {
    fn parse(kind: &str, payload: &str) -> Result<Self, CommandError> {
        let mut payload: serde_json::Value = serde_json::from_str(payload).map_err(invalid)?;
        if !payload.is_object() {
            return Err(invalid("offer payload must be a JSON object"));
        }
        if matches!(kind, "OfferCreated" | "OfferPriceChangedEvent")
            && payload.get("price").is_some_and(|price| !price.is_object())
        {
            return Err(invalid("price must be a JSON object"));
        }
        let pricing = if kind == "OfferPricingChanged" {
            let pricing = payload
                .get("pricing")
                .filter(|value| value.is_object())
                .ok_or_else(|| invalid("pricing must be a JSON object"))?;
            if pricing.get("price").is_none_or(|value| !value.is_object()) {
                return Err(invalid("price must be a JSON object"));
            }
            pricing
        } else {
            &payload
        };
        if matches!(kind, "OfferCreated" | "OfferPricingChanged")
            && pricing
                .get("was_price")
                .is_some_and(|value| !value.is_null() && !value.is_object())
        {
            return Err(invalid("was_price must be a JSON object or null"));
        }
        if kind == "OfferContentChanged"
            && payload
                .get("content")
                .is_some_and(|value| !value.is_object())
        {
            return Err(invalid("content must be a JSON object"));
        }
        if kind == "OfferCreated" {
            content::normalize_creation(&mut payload).map_err(invalid)?;
        }
        let change = match kind {
            "OfferContentsChanged" => {
                Self::Contents(serde_json::from_value(payload).map_err(invalid)?)
            }
            "OfferPricingChanged" => {
                Self::Pricing(serde_json::from_value(payload).map_err(invalid)?)
            }
            "OfferAvailabilityChanged" => {
                Self::Availability(serde_json::from_value(payload).map_err(invalid)?)
            }
            "OfferItemConditionChanged" => {
                Self::ItemCondition(serde_json::from_value(payload).map_err(invalid)?)
            }
            "OfferContentChanged" => {
                Self::Content(serde_json::from_value(payload).map_err(invalid)?)
            }
            "OfferIdentifiersChanged" => {
                Self::Identifiers(serde_json::from_value(payload).map_err(invalid)?)
            }
            "OfferLocalizationChanged" => {
                Self::Localization(serde_json::from_value(payload).map_err(invalid)?)
            }
            "OfferScrapedAtUpdated" => {
                Self::ScrapedAt(serde_json::from_value(payload).map_err(invalid)?)
            }
            "OfferImageAdded" => Self::Image(serde_json::from_value(payload).map_err(invalid)?),
            "OfferCreated" => Self::Created(serde_json::from_value(payload).map_err(invalid)?),
            "OfferTitleChanged" => Self::Title(serde_json::from_value(payload).map_err(invalid)?),
            "OfferPriceChangedEvent" => {
                Self::Price(serde_json::from_value(payload).map_err(invalid)?)
            }
            _ => return Err(CommandError::HandlerNotFound(kind.to_owned())),
        };
        let title = match &change {
            Self::Created(offer) => {
                guard_facts(
                    &offer.availability,
                    &offer.item_condition,
                    &Content::default(),
                )?;
                offer_content::guard(&offer.content, &offer.localization.language)
                    .map_err(invalid)?;
                if offer_content::selected(&offer.content, &offer.localization.language)
                    .map_err(invalid)?
                    .title
                    != offer.title
                {
                    return Err(invalid("selected content title must match title"));
                }
                let (filtered, skipped) = shared::images::filter(offer.images.clone());
                if skipped != 0
                    || filtered != offer.images
                    || offer.images.len() as u64 > u64::from(u32::MAX) + 1
                {
                    return Err(invalid(
                        "creation images must be valid, unique URLs within position capacity",
                    ));
                }
                if offer.url.len() > 2048 || offer.url.contains('\0') {
                    return Err(invalid("URL must be at most 2048 bytes and contain no NUL"));
                }
                Some(&offer.title)
            }
            Self::Contents(change) => {
                offer_content::guard(&change.content, &change.language).map_err(invalid)?;
                None
            }
            Self::Title(change) => Some(&change.title),
            Self::Availability(change) => {
                guard_facts(&change.availability, &None, &Content::default())?;
                None
            }
            Self::ItemCondition(change) => {
                guard_facts(&None, &change.item_condition, &Content::default())?;
                None
            }
            Self::Content(change) => {
                guard_facts(&None, &None, &change.content)?;
                None
            }
            Self::Pricing(change) => {
                if change.pricing.was_price.as_ref().is_some_and(|was| {
                    was.currency() != change.pricing.price.currency()
                        || was.amount() <= change.pricing.price.amount()
                }) {
                    return Err(invalid(
                        "was_price must use the current currency and be greater than price",
                    ));
                }
                None
            }
            Self::Price(_) | Self::ScrapedAt(_) | Self::Localization(_) | Self::Identifiers(_) => {
                None
            }
            Self::Image(change) => {
                if !shared::images::is_valid(&change.image_url) {
                    return Err(invalid("invalid image URL"));
                }
                None
            }
        };
        if title.is_some_and(|title| title.contains('\0')) {
            return Err(invalid("title must contain no NUL"));
        }
        Ok(change)
    }

    fn apply(&self, connection: &mut PgConnection) -> diesel::QueryResult<()> {
        let changed = match self {
            Self::Contents(change) => content::write(connection, change.id, &change.content, &change.language)?,
            Self::Pricing(change) => diesel::sql_query(
                "UPDATE offers SET price_amount=$2, currency=$3, was_price_amount=$4, was_price_currency=$5 WHERE id=$1",
            )
            .bind::<SqlUuid, _>(change.id)
            .bind::<BigInt, _>(change.pricing.price.amount())
            .bind::<Text, _>(change.pricing.price.currency().as_str())
            .bind::<diesel::sql_types::Nullable<BigInt>, _>(change.pricing.was_price.as_ref().map(Money::amount))
            .bind::<diesel::sql_types::Nullable<Text>, _>(change.pricing.was_price.as_ref().map(|value| value.currency().as_str()))
            .execute(connection)?,
            Self::Availability(change) => diesel::sql_query("UPDATE offers SET availability=$2 WHERE id=$1")
                .bind::<SqlUuid, _>(change.id)
                .bind::<diesel::sql_types::Nullable<Jsonb>, _>(change.availability.as_ref().map(|value| serde_json::to_value(value).expect("serializable availability")))
                .execute(connection)?,
            Self::ItemCondition(change) => diesel::sql_query("UPDATE offers SET item_condition=$2 WHERE id=$1")
                .bind::<SqlUuid, _>(change.id)
                .bind::<diesel::sql_types::Nullable<Jsonb>, _>(change.item_condition.as_ref().map(|value| serde_json::to_value(value).expect("serializable condition")))
                .execute(connection)?,
            Self::Content(change) => {
                let current = content::lock(connection, change.id)?;
                let list = offer_content::legacy_description(&current.content, &current.language, change.content.description.clone()).map_err(content::error)?;
                content::write(connection, change.id, &list, &current.language)?
            }
            Self::Identifiers(change) => diesel::sql_query(
                "UPDATE offers SET gtin=$2, sku=$3, external_identifiers=$4 WHERE id=$1",
            )
            .bind::<SqlUuid, _>(change.id)
            .bind::<diesel::sql_types::Nullable<Text>, _>(
                change.identifiers.gtin.as_ref().map(|value| &value.value),
            )
            .bind::<diesel::sql_types::Nullable<Text>, _>(
                change.identifiers.sku.as_ref().map(|value| &value.value),
            )
            .bind::<Jsonb, _>(Identifier(change.identifiers.external_identifiers.clone()))
            .execute(connection)?,
            Self::Localization(change) => {
                let current = content::lock(connection, change.id)?;
                let list = offer_content::legacy_language(&current.content, &current.language, change.localization.language.clone()).map_err(content::error)?;
                content::write_language(connection, change.id, &list, &change.localization.language)?;
                diesel::sql_query("UPDATE offers SET target_market=$2 WHERE id=$1")
                    .bind::<SqlUuid, _>(change.id)
                    .bind::<diesel::sql_types::Nullable<Jsonb>, _>(change.localization.target_market.as_ref().map(|market| serde_json::to_value(market.canonicalized()).expect("serializable market")))
                    .execute(connection)?
            }
            Self::ScrapedAt(change) => diesel::sql_query(
                "UPDATE offers SET scraped_at = GREATEST(scraped_at, $2) WHERE id = $1",
            )
            .bind::<SqlUuid, _>(change.id)
            .bind::<Timestamptz, _>(change.scraped_at)
            .execute(connection)?,
            Self::Image(change) => diesel::sql_query(
                "INSERT INTO offer_images (offer_id, image_url, position) VALUES ($1,$2,$3)
                 ON CONFLICT (offer_id, position) DO UPDATE SET image_url = EXCLUDED.image_url
                 WHERE offer_images.image_url = EXCLUDED.image_url",
            )
            .bind::<SqlUuid, _>(change.id)
            .bind::<Text, _>(&change.image_url)
            .bind::<BigInt, _>(i64::from(change.position))
            .execute(connection)?,
            Self::Created(offer) => {
                let count = diesel::sql_query(
                    "INSERT INTO offers (id, retailer_code, price_amount, currency, created_at, scraped_at, target_market, gtin, sku, external_identifiers, was_price_amount, was_price_currency, availability, item_condition, content)
                     VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15)",
                )
                .bind::<SqlUuid, _>(offer.id)
                .bind::<Text, _>(offer.retailer_code.as_str())
                .bind::<BigInt, _>(offer.price.amount())
                .bind::<Text, _>(offer.price.currency().as_str())
                .bind::<Timestamptz, _>(offer.created_at)
                .bind::<diesel::sql_types::Nullable<Timestamptz>, _>(offer.scraped_at)
                .bind::<diesel::sql_types::Nullable<diesel::sql_types::Jsonb>, _>(offer.localization.target_market.as_ref().map(|market| serde_json::to_value(market.canonicalized()).expect("serializable market")))
                .bind::<diesel::sql_types::Nullable<Text>, _>(offer.identifiers.gtin.as_ref().map(|value| &value.value))
                .bind::<diesel::sql_types::Nullable<Text>, _>(offer.identifiers.sku.as_ref().map(|value| &value.value))
                .bind::<Jsonb, _>(Identifier(offer.identifiers.external_identifiers.clone()))
                .bind::<diesel::sql_types::Nullable<BigInt>, _>(offer.was_price.as_ref().map(Money::amount))
                .bind::<diesel::sql_types::Nullable<Text>, _>(offer.was_price.as_ref().map(|value| value.currency().as_str()))
                .bind::<diesel::sql_types::Nullable<Jsonb>, _>(offer.availability.as_ref().map(|value| serde_json::to_value(value).expect("serializable availability")))
                .bind::<diesel::sql_types::Nullable<Jsonb>, _>(offer.item_condition.as_ref().map(|value| serde_json::to_value(value).expect("serializable condition")))
                .bind::<Jsonb, _>(serde_json::to_value(crate::stored_content::StoredContent::new(&offer.content, &offer.localization.language, &offer.url).map_err(content::error)?).expect("serializable content"))
                .execute(connection)?;
                for (position, image_url) in offer.images.iter().enumerate() {
                    diesel::sql_query(
                        "INSERT INTO offer_images (offer_id, image_url, position) VALUES ($1,$2,$3)",
                    )
                    .bind::<SqlUuid, _>(offer.id)
                    .bind::<Text, _>(image_url)
                    .bind::<BigInt, _>(position as i64)
                    .execute(connection)?;
                }
                count
            }
            Self::Title(change) => {
                let current = content::lock(connection, change.id)?;
                let list = offer_content::legacy_title(&current.content, &current.language, change.title.clone()).map_err(content::error)?;
                content::write(connection, change.id, &list, &current.language)?
            }
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

fn guard_facts(
    availability: &Option<Availability>,
    condition: &Option<ItemCondition>,
    content: &Content,
) -> Result<(), CommandError> {
    if let Some(value) = availability {
        value.guard().map_err(invalid)?;
    }
    if let Some(value) = condition {
        value.guard().map_err(invalid)?;
    }
    content.guard().map_err(invalid)
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
    fn source_fixtures_and_historical_creation_keep_the_producer_contract() {
        let fixtures = [
            include_str!(
                "../../retailer_sourcing/tests/fixtures/offer_export/import_payload_v6.json"
            ),
            include_str!(
                "../../retailer_sourcing/tests/fixtures/offer_export/import_payload_v7.json"
            ),
            include_str!(
                "../../retailer_sourcing/tests/fixtures/offer_export/import_payload_v8.json"
            ),
        ];
        for (version, fixture) in (6..=8).zip(fixtures) {
            let mut payload: serde_json::Value = serde_json::from_str(fixture).unwrap();
            payload["id"] = json!(Uuid::now_v7());
            payload["created_at"] = json!("2026-10-27T00:00:00Z");
            let Change::Created(offer) =
                Change::parse("OfferCreated", &payload.to_string()).unwrap()
            else {
                panic!("expected creation")
            };
            assert_eq!(offer.was_price.unwrap().amount(), 2999);
            assert_eq!(offer.availability.is_some(), version >= 7);
            assert_eq!(offer.item_condition.is_some(), version >= 7);
            assert_eq!(offer.content[0].description.is_some(), version >= 8);

            // Replay preserves source facts, including an old comparison that
            // no longer agrees with current-price policy. Read mapping hides it.
            for (amount, currency) in [(1999, "AUD"), (1998, "AUD"), (2999, "EUR")] {
                payload["was_price"] = json!({"amount":amount,"currency":currency});
                let Change::Created(offer) =
                    Change::parse("OfferCreated", &payload.to_string()).unwrap()
                else {
                    panic!("expected creation")
                };
                let was = offer.was_price.unwrap();
                assert_eq!(was.amount(), amount);
                assert_eq!(was.currency().as_str(), currency);
                let new = json!({"id":payload["id"],"pricing":{"price":payload["price"],"was_price":payload["was_price"]}});
                assert!(Change::parse("OfferPricingChanged", &new.to_string()).is_err());
            }
        }
    }

    #[test]
    fn fact_changes_require_explicit_values_and_reject_malformed_shapes() {
        let id = Uuid::now_v7();
        for (kind, valid, invalid) in [
            (
                "OfferPricingChanged",
                json!({"id":id,"pricing":{"price":{"amount":10,"currency":"USD"},"was_price":null}}),
                json!({"id":id,"pricing":{"price":{"amount":10,"currency":"USD"}}}),
            ),
            (
                "OfferAvailabilityChanged",
                json!({"id":id,"availability":null}),
                json!({"id":id}),
            ),
            (
                "OfferItemConditionChanged",
                json!({"id":id,"item_condition":null}),
                json!({"id":id}),
            ),
            (
                "OfferContentChanged",
                json!({"id":id,"content":{"description":null}}),
                json!({"id":id,"content":{}}),
            ),
        ] {
            assert!(Change::parse(kind, &valid.to_string()).is_ok(), "{kind}");
            assert!(Change::parse(kind, &invalid.to_string()).is_err(), "{kind}");
        }
        for (kind, payload) in [
            (
                "OfferPricingChanged",
                json!({"id":id,"pricing":[{"amount":10,"currency":"USD"},null]}),
            ),
            (
                "OfferPricingChanged",
                json!({"id":id,"pricing":{"price":[10,"USD"],"was_price":null}}),
            ),
            (
                "OfferPricingChanged",
                json!({"id":id,"pricing":{"price":{"amount":10,"currency":"USD"},"was_price":[20,"USD"]}}),
            ),
            (
                "OfferAvailabilityChanged",
                json!({"id":id,"availability":{"other":" "}}),
            ),
            (
                "OfferItemConditionChanged",
                json!({"id":id,"item_condition":{"other":"\0"}}),
            ),
            ("OfferContentChanged", json!({"id":id,"content":[null]})),
            (
                "OfferContentChanged",
                json!({"id":id,"content":{"description":"\0"}}),
            ),
        ] {
            assert!(
                Change::parse(kind, &payload.to_string()).is_err(),
                "{kind}: {payload}"
            );
        }
    }

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
