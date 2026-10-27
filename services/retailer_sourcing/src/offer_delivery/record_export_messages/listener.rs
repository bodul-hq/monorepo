use diesel::r2d2::PoolError;
use uuid::Uuid;

use crate::offer_discovery::prelude::{
    EXPORT_SCHEMA_VERSION, ExportStoreError, OfferExport, OfferExportWasPrepared,
};
use crate::platform::bus::{Event, EventContext, Listen};
use crate::platform::outbox::{OutgoingMessage, RecordError};

/// The offers service's inbox destination.
const ROUTING_KEY: &str = "offers";

pub(super) trait Store: Send + Sync + 'static {
    /// The completed export, or `None` while it is missing or still pending.
    fn load(&self, id: Uuid) -> Result<Option<OfferExport>, StoreError>;
    fn record(
        &self,
        messages: &[OutgoingMessage],
        context: &EventContext,
    ) -> Result<(), StoreError>;
}

#[derive(Debug, thiserror::Error)]
pub enum StoreError {
    #[error("connection pool error: {0}")]
    Connection(#[from] PoolError),
    #[error(transparent)]
    Export(#[from] ExportStoreError),
    #[error(transparent)]
    Record(#[from] RecordError),
}

#[derive(Debug, thiserror::Error)]
pub enum ListenerError {
    #[error("offer export {0} is not completed")]
    NotCompleted(Uuid),
    #[error("offer export {export_id} has unsupported schema version {version}")]
    UnsupportedSchema { export_id: Uuid, version: u32 },
    #[error("offer export {0} does not match the event's source or schema")]
    Mismatch(Uuid),
    #[error("offer export {0} payload serialization failed: {1}")]
    Serialization(Uuid, serde_json::Error),
    #[error(transparent)]
    Store(#[from] StoreError),
}

pub(super) struct Listener<S> {
    store: S,
}

impl<S> Listener<S> {
    pub fn new(store: S) -> Self {
        Self { store }
    }
}

impl<S: Store> Listen for Listener<S> {
    type Event = OfferExportWasPrepared;
    type Error = ListenerError;
    const NAME: &'static str = "RecordOfferExportMessages";

    fn listen(&self, event: Self::Event, context: &EventContext) -> Result<(), Self::Error> {
        let export_id = event.export_id;
        if event.schema_version != EXPORT_SCHEMA_VERSION {
            return Err(ListenerError::UnsupportedSchema {
                export_id,
                version: event.schema_version,
            });
        }
        // The event names an immutable snapshot; never the latest source data.
        let export = self
            .store
            .load(export_id)?
            .ok_or(ListenerError::NotCompleted(export_id))?;
        if export.id != export_id
            || export.schema_version != event.schema_version
            || export.source != event.source
        {
            return Err(ListenerError::Mismatch(export_id));
        }
        let messages = export
            .entries
            .iter()
            .map(|entry| {
                Ok(OutgoingMessage {
                    id: entry.id,
                    payload: serde_json::to_string(&entry.payload)
                        .map_err(|error| ListenerError::Serialization(export_id, error))?,
                    routing_key: ROUTING_KEY,
                    event_type: OfferExportWasPrepared::NAME,
                })
            })
            .collect::<Result<Vec<_>, ListenerError>>()?;
        self.store.record(&messages, context)?;
        tracing::info!(
            %export_id,
            offer_id = %export.source.offer_id,
            recorded = messages.len(),
            excluded = export.exclusions.len(),
            "offer export recorded for delivery"
        );
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use money::Money;
    use serde_json::{Value, json};

    use super::*;
    use crate::offer_discovery::prelude::export_testing::{
        ExclusionReason, ExportEntry, ExportExclusion, OfferImport, source,
    };

    #[derive(Default)]
    struct FakeStore {
        export: Option<OfferExport>,
        fail_load: bool,
        fail_record: bool,
        recorded: Mutex<Vec<(Vec<OutgoingMessage>, EventContext)>>,
    }

    impl Store for FakeStore {
        fn load(&self, id: Uuid) -> Result<Option<OfferExport>, StoreError> {
            if self.fail_load {
                return Err(ExportStoreError::InvalidRecord.into());
            }
            Ok(self.export.clone().filter(|export| export.id == id))
        }

        fn record(
            &self,
            messages: &[OutgoingMessage],
            context: &EventContext,
        ) -> Result<(), StoreError> {
            if self.fail_record {
                return Err(RecordError::Conflict(messages[0].id).into());
            }
            self.recorded
                .lock()
                .unwrap()
                .push((messages.to_vec(), context.clone()));
            Ok(())
        }
    }

    fn entry(variation_index: usize, amount: i64) -> ExportEntry {
        ExportEntry {
            id: Uuid::now_v7(),
            variation_index,
            payload: OfferImport {
                retailer_code: bodul_shared::retailer::RetailerCode::AnkerAu,
                title: "USB charger".into(),
                price: serde_json::from_value::<Money>(
                    json!({"amount": amount, "currency": "AUD"}),
                )
                .unwrap(),
                url: format!("https://shop.example/Charger?variant={variation_index}#details"),
            },
        }
    }

    fn export(entries: Vec<ExportEntry>, exclusions: Vec<ExportExclusion>) -> OfferExport {
        OfferExport {
            id: Uuid::now_v7(),
            schema_version: EXPORT_SCHEMA_VERSION,
            source: source(),
            entries,
            exclusions,
        }
    }

    fn event(export: &OfferExport) -> OfferExportWasPrepared {
        OfferExportWasPrepared {
            export_id: export.id,
            schema_version: export.schema_version,
            source: export.source.clone(),
        }
    }

    fn context() -> EventContext {
        EventContext {
            event_id: Uuid::now_v7(),
            correlation_id: Some(Uuid::now_v7()),
            causation_id: Some(Uuid::now_v7()),
            source: Some("event:OfferWasStructuralized".into()),
        }
    }

    fn listener(export: OfferExport) -> Listener<FakeStore> {
        Listener::new(FakeStore {
            export: Some(export),
            ..Default::default()
        })
    }

    #[test]
    fn records_one_message_per_entry_and_none_for_exclusions() {
        let exclusion = ExportExclusion {
            variation_index: 1,
            url: "https://shop.example/Charger?variant=1".into(),
            reason: ExclusionReason::MissingPrice,
        };
        for (entries, exclusions) in [
            (vec![], vec![]),
            (vec![], vec![exclusion.clone()]),
            (vec![entry(0, 1999)], vec![]),
            (vec![entry(0, 1999), entry(2, 0)], vec![exclusion]),
        ] {
            let export = export(entries, exclusions);
            let listener = listener(export.clone());
            let context = context();

            listener.listen(event(&export), &context).unwrap();

            let recorded = listener.store.recorded.lock().unwrap();
            assert_eq!(recorded.len(), 1);
            let (messages, recorded_context) = &recorded[0];
            assert_eq!(recorded_context, &context);
            assert_eq!(messages.len(), export.entries.len());
            for (message, entry) in messages.iter().zip(&export.entries) {
                assert_eq!(message.id, entry.id);
                assert_eq!(message.routing_key, "offers");
                assert_eq!(message.event_type, "OfferExportWasPrepared");
                assert_eq!(
                    serde_json::from_str::<OfferImport>(&message.payload).unwrap(),
                    entry.payload
                );
            }
        }
    }

    #[test]
    fn payload_is_the_bare_import_command() {
        let export = export(vec![entry(0, 1999)], vec![]);
        let listener = listener(export.clone());

        listener.listen(event(&export), &context()).unwrap();

        let recorded = listener.store.recorded.lock().unwrap();
        let payload: Value = serde_json::from_str(&recorded[0].0[0].payload).unwrap();
        assert_eq!(
            payload,
            json!({
                "retailer_code": "AnkerAu",
                "title": "USB charger",
                "price": {"amount": 1999, "currency": "AUD"},
                "url": "https://shop.example/Charger?variant=0#details"
            })
        );
    }

    #[test]
    fn a_missing_or_pending_export_fails() {
        let export = export(vec![entry(0, 1999)], vec![]);
        let listener = Listener::new(FakeStore::default());

        let error = listener.listen(event(&export), &context()).unwrap_err();

        assert!(matches!(error, ListenerError::NotCompleted(id) if id == export.id));
        assert!(listener.store.recorded.lock().unwrap().is_empty());
    }

    #[test]
    fn store_failures_fail_the_event() {
        let export = export(vec![entry(0, 1999)], vec![]);
        for (fail_load, fail_record) in [(true, false), (false, true)] {
            let listener = Listener::new(FakeStore {
                export: Some(export.clone()),
                fail_load,
                fail_record,
                ..Default::default()
            });

            let error = listener.listen(event(&export), &context()).unwrap_err();

            assert!(matches!(error, ListenerError::Store(_)), "{error}");
            assert!(listener.store.recorded.lock().unwrap().is_empty());
        }
    }

    #[test]
    fn an_unsupported_schema_or_different_source_records_nothing() {
        let export = export(vec![entry(0, 1999)], vec![]);
        let mut unsupported = event(&export);
        unsupported.schema_version = EXPORT_SCHEMA_VERSION + 1;
        let mut other_source = event(&export);
        other_source.source.offer_id = Uuid::now_v7();
        let mut other_snapshot = export.clone();
        other_snapshot.schema_version = EXPORT_SCHEMA_VERSION + 1;

        let listener = listener(export.clone());
        assert!(matches!(
            listener.listen(unsupported, &context()).unwrap_err(),
            ListenerError::UnsupportedSchema { .. }
        ));
        assert!(matches!(
            listener.listen(other_source, &context()).unwrap_err(),
            ListenerError::Mismatch(_)
        ));
        let mismatched = self::listener(other_snapshot);
        assert!(matches!(
            mismatched.listen(event(&export), &context()).unwrap_err(),
            ListenerError::Mismatch(_)
        ));
        assert!(listener.store.recorded.lock().unwrap().is_empty());
        assert!(mismatched.store.recorded.lock().unwrap().is_empty());
    }
}
