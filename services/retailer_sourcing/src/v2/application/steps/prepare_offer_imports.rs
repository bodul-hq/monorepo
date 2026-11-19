//! Record the structuralized artifact's import messages for the offers inbox.

use super::structuralize_offer_details::OfferDetailsStructuralized;
use crate::offer_discovery::prelude::shopify_imports;
use crate::platform::bus::{Bus, Event, EventContext, Listen, stable_command_id};
use crate::platform::db::DbPool;
use crate::platform::object_storage::{ObjectStorage, STRUCTURALIZED_OFFERS_BUCKET};
use crate::platform::outbox::{OutgoingMessage, record};
use serde_json::to_string;

struct Listener {
    pool: DbPool,
    storage: ObjectStorage,
}

impl Listen for Listener {
    type Event = OfferDetailsStructuralized;
    type Error = String;
    const NAME: &'static str = "PrepareOfferDetailsImports";

    fn listen(&self, event: Self::Event, context: &EventContext) -> Result<(), String> {
        let source = &event.storage;
        let body = self
            .storage
            .get_offer(STRUCTURALIZED_OFFERS_BUCKET, source)
            .map_err(|e| e.to_string())?;
        let imports = shopify_imports(
            &body,
            event.extracted.fetched.retailer_code,
            source.scraped_at,
        )
        .map_err(|e| e.to_string())?;
        let mut messages = Vec::with_capacity(imports.len());
        for (variant_id, import) in imports {
            messages.push(OutgoingMessage {
                id: stable_command_id(&source.object_storage_id, &variant_id.to_be_bytes()),
                payload: to_string(&import).map_err(|e| e.to_string())?,
                routing_key: "offers",
                event_type: OfferDetailsStructuralized::NAME,
            });
        }
        let mut conn = self.pool.get().map_err(|e| e.to_string())?;
        record(&mut conn, &messages, context).map_err(|e| e.to_string())
    }
}

pub(crate) fn register(bus: &mut Bus, pool: DbPool, storage: ObjectStorage) {
    bus.listen(Listener { pool, storage });
}
