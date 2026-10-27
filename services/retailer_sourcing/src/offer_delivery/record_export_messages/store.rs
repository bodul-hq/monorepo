use uuid::Uuid;

use super::listener::{Store, StoreError};
use crate::offer_discovery::prelude::{ExportStore, OfferExport};
use crate::platform::bus::EventContext;
use crate::platform::db::DbPool;
use crate::platform::object_storage::ObjectStorage;
use crate::platform::outbox::{self, OutgoingMessage};

pub struct PgStore {
    pool: DbPool,
    exports: ExportStore,
}

impl PgStore {
    pub fn new(pool: DbPool, objects: ObjectStorage) -> Self {
        Self {
            exports: ExportStore::new(pool.clone(), objects),
            pool,
        }
    }
}

impl Store for PgStore {
    fn load(&self, id: Uuid) -> Result<Option<OfferExport>, StoreError> {
        Ok(self.exports.load(id)?)
    }

    fn record(
        &self,
        messages: &[OutgoingMessage],
        context: &EventContext,
    ) -> Result<(), StoreError> {
        let mut conn = self.pool.get()?;
        Ok(outbox::record(&mut conn, messages, context)?)
    }
}
