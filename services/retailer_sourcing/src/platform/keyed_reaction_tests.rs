use super::*;
use commanding::prelude::CommandStorePort;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{
    Mutex,
    atomic::{AtomicBool, Ordering},
};

#[derive(Serialize, Deserialize)]
struct Batch {
    run_id: Uuid,
    ids: Vec<u32>,
}
impl Event for Batch {
    const NAME: &'static str = "TestBatch";
}
#[derive(Serialize, Deserialize)]
struct Work {
    id: u32,
}
impl Command for Work {
    const NAME: &'static str = "TestBatchWork";
}
fn commands(batch: &Batch) -> Vec<KeyedCommand<Work>> {
    batch
        .ids
        .iter()
        .map(|id| KeyedCommand {
            id: stable_command_id(&batch.run_id, &id.to_be_bytes()),
            run_id: batch.run_id,
            command: Work { id: *id },
        })
        .collect()
}
#[derive(Default)]
struct Store {
    rows: Mutex<HashMap<Uuid, NewCommandEnvelope>>,
    failed: AtomicBool,
}
impl CommandStorePort for Store {
    fn record(&self, envelope: &NewCommandEnvelope) -> Result<(), CommandError> {
        let work: Work = serde_json::from_str(&envelope.command.payload).unwrap();
        if work.id == 2 && !self.failed.swap(true, Ordering::SeqCst) {
            return Err(CommandError::Storage("transient".into()));
        }
        self.rows
            .lock()
            .unwrap()
            .entry(envelope.metadata.as_ref().unwrap().command_id)
            .or_insert_with(|| envelope.clone());
        Ok(())
    }
}
#[test]
fn partial_fanout_replays_without_duplicates_and_preserves_metadata() {
    let store = Arc::new(Store::default());
    let reaction = KeyedReaction {
        gateway: Arc::new(CommandGateway::two_phased(store.clone())),
        reaction: commands,
    };
    let run_id = Uuid::now_v7();
    let event_id = Uuid::now_v7();
    let mut envelope = NewEventEnvelope {
        event_type: Batch::NAME.into(),
        payload: serde_json::to_string(&Batch {
            run_id,
            ids: vec![1, 2, 3],
        })
        .unwrap(),
        metadata: Some(kernel::NewEventMetadata {
            sequence: None,
            event_id,
            correlation_id: Some(run_id),
            causation_id: None,
            source: Some("test".into()),
        }),
    };
    assert!(reaction.handle(&envelope).is_err());
    assert_eq!(store.rows.lock().unwrap().len(), 1);
    reaction.handle(&envelope).unwrap();
    reaction.handle(&envelope).unwrap();
    assert_eq!(store.rows.lock().unwrap().len(), 3);
    for row in store.rows.lock().unwrap().values() {
        let meta = row.metadata.as_ref().unwrap();
        assert_eq!(meta.correlation_id, Some(run_id));
        assert_eq!(meta.causation_id, Some(event_id));
    }
    envelope.metadata = None;
    assert!(reaction.handle(&envelope).is_err());
}
#[test]
fn command_identity_matches_standard_uuid_v5() {
    assert_eq!(
        stable_command_id(&Uuid::NAMESPACE_DNS, b"python.org").to_string(),
        "886313e1-3b8a-5372-9b90-0c9aee199e5d"
    );
}
