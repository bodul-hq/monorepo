use super::{Handler, fetch_details_for_catalog_products};
use crate::platform::{Platform, bus::Bus};

pub fn register(bus: &mut Bus, platform: &Platform) {
    bus.handle(Handler {
        fetcher: platform.offer_http.clone(),
        store: platform.object_storage.clone(),
    });
    bus.on_many_keyed(fetch_details_for_catalog_products);
    // OfferDetailsFetched is deliberately terminal: no subscriber or outbox registration.
}
