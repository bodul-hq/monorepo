//! Run the owning retailer's structuralizer against the pinned extracted object.

use super::extract_offer_details::OfferDetailsExtracted;
use crate::platform::bus::{Bus, Command, Event, Handle, KeyedCommand, stable_command_id};
use crate::platform::object_storage::{
    EXTRACTED_OFFERS_BUCKET, ObjectStorage, OfferStorageMetadata, STRUCTURALIZED_OFFERS_BUCKET,
};
use retailer_data_extractor::structuralize_v2_for;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct StructuralizeOfferDetails(OfferDetailsExtracted);

impl Command for StructuralizeOfferDetails {
    const NAME: &'static str = "StructuralizeOfferDetails";
}

#[derive(Debug, Serialize, Deserialize)]
pub(super) struct OfferDetailsStructuralized {
    pub(super) extracted: OfferDetailsExtracted,
    pub(super) storage: OfferStorageMetadata,
}

impl Event for OfferDetailsStructuralized {
    const NAME: &'static str = "OfferDetailsStructuralized";
}

fn structuralize(event: &OfferDetailsExtracted) -> Vec<KeyedCommand<StructuralizeOfferDetails>> {
    let command = KeyedCommand {
        id: stable_command_id(
            &event.storage.object_storage_id,
            StructuralizeOfferDetails::NAME.as_bytes(),
        ),
        run_id: event.fetched.run_id,
        command: StructuralizeOfferDetails(event.clone()),
    };
    vec![command]
}

struct Handler(ObjectStorage);

impl Handle for Handler {
    type Command = StructuralizeOfferDetails;
    type Event = OfferDetailsStructuralized;
    type Error = String;

    fn handle(
        &self,
        StructuralizeOfferDetails(extracted): StructuralizeOfferDetails,
    ) -> Result<Vec<Self::Event>, String> {
        let input = self
            .0
            .get_offer(EXTRACTED_OFFERS_BUCKET, &extracted.storage)
            .map_err(|e| e.to_string())?;
        let document = structuralize_v2_for(extracted.fetched.retailer_code, &input)?;
        let storage = self
            .0
            .create_derived_offer(STRUCTURALIZED_OFFERS_BUCKET, &extracted.storage, &document)
            .map_err(|e| e.to_string())?;
        Ok(vec![OfferDetailsStructuralized { extracted, storage }])
    }
}

pub(crate) fn register(bus: &mut Bus, storage: ObjectStorage) {
    bus.on_many_keyed(structuralize);
    bus.handle(Handler(storage));
}
