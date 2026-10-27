use super::{handler, reacts_to, store};
use crate::platform::{Platform, bus::Bus};

pub use super::{entity::OfferExport, event::OfferExportWasPrepared, store::PgStore};

pub fn register(bus: &mut Bus, platform: &Platform) {
    let store = store::PgStore::new(platform.db.clone(), platform.object_storage.clone());
    bus.handle(handler::Handler::new(store));
    bus.on(reacts_to::on);
}
