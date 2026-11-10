use super::{handler, reacts_to};
use crate::platform::{Platform, bus::Bus};
pub fn register(bus: &mut Bus, platform: &Platform) {
    bus.handle(handler::Handler {
        fetcher: platform.offer_http.clone(),
        store: platform.object_storage.clone(),
    });
    bus.on_many_keyed(reacts_to::on);
    // OfferDetailsFetched is deliberately terminal: no subscriber or outbox registration.
}
