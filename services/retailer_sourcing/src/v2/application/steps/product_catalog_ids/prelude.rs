use super::{Handler, start_catalog_for_locales};
use crate::platform::{Platform, bus::Bus};

pub(crate) use super::{FetchProductCatalogIdentifiers, ProductCatalogIdsRetrieved, initial};

pub fn register(bus: &mut Bus, platform: &Platform) {
    bus.handle(Handler(platform.offer_http.clone()));
    bus.on_many_keyed(start_catalog_for_locales);
}
