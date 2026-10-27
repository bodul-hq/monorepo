use super::{handler, reacts_to, store};
use crate::platform::{Platform, bus::Bus};

pub use super::{
    entity::{OfferExport, SCHEMA_VERSION},
    event::OfferExportWasPrepared,
    store::{PgStore, StoreError},
};

pub fn register(bus: &mut Bus, platform: &Platform) {
    let store = store::PgStore::new(platform.db.clone(), platform.object_storage.clone());
    bus.handle(handler::Handler::new(store));
    bus.on(reacts_to::on);
}

/// Builds exports for other contexts' tests.
#[cfg(test)]
pub(crate) mod testing {
    pub(crate) use super::super::conversion::tests::{ANKER, source};
    pub use super::super::entity::{
        ExclusionReason, ExportEntry, ExportExclusion, ExportSource, OfferImport,
    };
    use super::super::handler::Store;
    use super::{OfferExport, OfferExportWasPrepared, PgStore};
    use crate::platform::db::DbPool;
    use crate::platform::object_storage::{ObjectStorage, STRUCTURALIZED_OFFERS_BUCKET};

    /// A source whose structuralized `document` is in object storage.
    pub fn stored_source(objects: &ObjectStorage, document: &str) -> ExportSource {
        let mut source = source();
        source.storage = objects
            .create_offer(STRUCTURALIZED_OFFERS_BUCKET, document)
            .unwrap();
        source
    }

    /// Prepares and completes the export of `source`.
    pub fn completed(db: &DbPool, objects: ObjectStorage, source: ExportSource) -> OfferExport {
        let store = PgStore::new(db.clone(), objects);
        store.complete(store.prepare(source).unwrap()).unwrap()
    }

    /// Prepares the export of `source` without completing it.
    pub fn pending(db: &DbPool, objects: ObjectStorage, source: ExportSource) -> OfferExport {
        PgStore::new(db.clone(), objects)
            .prepare(source)
            .unwrap()
            .snapshot
    }

    pub fn event(export: &OfferExport) -> OfferExportWasPrepared {
        OfferExportWasPrepared {
            export_id: export.id,
            schema_version: export.schema_version,
            source: export.source.clone(),
        }
    }
}
