//! Everything other modules may use from this slice.

use super::super::Retailers;
use super::{handler, reacts_to, store};
use crate::platform::Platform;
use crate::platform::bus::Bus;

pub use super::event::SitemapProcessed;

/// Registers the slice's handler and the reaction that starts it.
pub fn register(bus: &mut Bus, platform: &Platform) {
    let store = store::PgStore::new(platform.db.clone(), platform.object_storage.clone());
    bus.handle(handler::Handler::new(store, Retailers));
    bus.on(reacts_to::on);
}
