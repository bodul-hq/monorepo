//! Everything other modules may use from this slice.

use super::{handler, reacts_to, store};
use crate::platform::Platform;
use crate::platform::bus::Bus;

pub use super::event::{OfferPageDownloadSkipped, OfferPageWasDownloaded};

/// Registers the slice's handler and the reaction that starts it.
pub fn register(bus: &mut Bus, platform: &Platform) {
    let store = store::PgStore::new(platform.db.clone());
    bus.handle(handler::Handler::new(store, platform.http.clone()));
    bus.on(reacts_to::on);
}
