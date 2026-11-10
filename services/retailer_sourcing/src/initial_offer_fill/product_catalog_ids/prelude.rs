pub use super::event::ProductCatalogIdsRetrieved;
use super::{handler, reacts_to};
use crate::platform::{Platform, bus::Bus};
pub fn register(bus: &mut Bus, platform: &Platform) {
    bus.handle(handler::Handler(platform.offer_http.clone()));
    bus.on_many_keyed(reacts_to::acquired);
    bus.on_many_keyed(reacts_to::next_batch);
}
