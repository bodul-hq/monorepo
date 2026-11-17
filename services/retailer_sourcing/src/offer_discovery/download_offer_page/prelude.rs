//! Everything other modules may use from this slice.

use super::{handler, reacts_to, store};
use crate::platform::Platform;
use crate::platform::bus::Bus;
use handler::Handler;
use reacts_to::on as reacts_to_on;
use store::PgStore;

pub use super::event::OfferPageWasDownloaded;

/// Registers the slice's handler and the reaction that starts it.
pub fn register(bus: &mut Bus, platform: &Platform) {
    let store = PgStore::new(platform.db.clone(), platform.object_storage.clone());
    bus.handle(Handler::new(store, platform.offer_http.clone()));
    bus.on(reacts_to_on);
}
