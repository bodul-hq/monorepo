//! Everything other modules may use from v2.
//!
//! Exposes the v2 routes, events and step registration.

use super::application::prelude::extract_offer_details::register as register_extraction;
use super::application::prelude::offer_details::prelude::register as register_offer_details;
use super::application::prelude::product_catalog_ids::prelude::register as register_product_catalog_ids;
pub use super::ui::prelude::routes;
use crate::platform::Platform;
use crate::platform::bus::Bus;

pub(crate) use super::application::prelude::acquire_storefront_token::{
    AcquireStorefrontToken,
    StorefrontTokenAcquired,
    prelude::register as register_storefront_token, //
};

pub fn register(bus: &mut Bus, platform: &Platform) {
    register_storefront_token(bus, platform);
    register_product_catalog_ids(bus, platform);

    register_offer_details(bus, platform);
    register_extraction(bus, platform.object_storage.clone());
}

pub(crate) use super::application::prelude::ProductCatalogIdsRetrieved;

#[cfg(test)]
pub(crate) use super::application::prelude::acquire_storefront_token::{
    Fetch as StorefrontTokenFetch, Handler as AcquireStorefrontTokenHandler,
};
