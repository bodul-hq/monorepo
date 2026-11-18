//! Extract the retailer-owned model from a stored initial-fill artifact.

use super::offer_details::OfferDetailsFetched;
use crate::platform::bus::{Bus, Command, Event, Handle, KeyedCommand, stable_command_id};
use crate::platform::object_storage::{
    EXTRACTED_OFFERS_BUCKET, ObjectStorage, OfferStorageMetadata, RAW_OFFERS_BUCKET,
};
use retailer_data_extractor::extract_v2_for;
use serde::{Deserialize, Serialize};
use serde_json::{Value, from_str, to_value};

#[derive(Debug, Serialize, Deserialize)]
struct ExtractOfferDetails(OfferDetailsFetched);

impl Command for ExtractOfferDetails {
    const NAME: &'static str = "ExtractOfferDetails";
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(super) struct OfferDetailsExtracted {
    pub(super) fetched: OfferDetailsFetched,
    pub(super) storage: OfferStorageMetadata,
}

impl Event for OfferDetailsExtracted {
    const NAME: &'static str = "OfferDetailsExtracted";
}

fn extract(event: &OfferDetailsFetched) -> Vec<KeyedCommand<ExtractOfferDetails>> {
    vec![KeyedCommand {
        id: stable_command_id(
            &event.storage.object_storage_id,
            ExtractOfferDetails::NAME.as_bytes(),
        ),
        run_id: event.run_id,
        command: ExtractOfferDetails(event.clone()),
    }]
}

struct Handler(ObjectStorage);

impl Handle for Handler {
    type Command = ExtractOfferDetails;
    type Event = OfferDetailsExtracted;
    type Error = String;

    fn handle(
        &self,
        ExtractOfferDetails(fetched): ExtractOfferDetails,
    ) -> Result<Vec<Self::Event>, String> {
        let raw = self
            .0
            .get_offer(RAW_OFFERS_BUCKET, &fetched.storage)
            .map_err(|e| e.to_string())?;
        let mut artifact: Value = from_str(&raw).map_err(|e| e.to_string())?;
        artifact
            .as_object_mut()
            .ok_or("expected a raw offer object")?
            .entry("acquired_at")
            .or_insert(to_value(fetched.fetched_at).map_err(|e| e.to_string())?);
        let extracted = extract_v2_for(fetched.retailer_code, &artifact.to_string())?;
        let storage = self
            .0
            .create_derived_offer(EXTRACTED_OFFERS_BUCKET, &fetched.storage, &extracted)
            .map_err(|e| e.to_string())?;
        Ok(vec![OfferDetailsExtracted { fetched, storage }])
    }
}

pub(crate) fn register(bus: &mut Bus, storage: ObjectStorage) {
    bus.on_many_keyed(extract);
    bus.handle(Handler(storage));
}
