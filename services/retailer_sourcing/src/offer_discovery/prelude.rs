//! Everything other modules may use from offer discovery: its registration and
//! the events other contexts may react to.

use super::{
    discover_offers, download_offer_page, extract_offer_page, normalize_offer,
    processing_for_export, structuralize_offer,
};
use crate::platform::Platform;
use crate::platform::bus::Bus;

#[expect(
    unused_imports,
    reason = "offer discovery's final events; no other context reacts to them yet"
)]
pub use super::structuralize_offer::prelude::OfferWasStructuralized;

#[cfg(test)]
pub(crate) use super::processing_for_export::prelude::testing as export_testing;
pub use super::processing_for_export::prelude::{
    OfferExport, OfferExportWasPrepared, PgStore as ExportStore,
    SCHEMA_VERSION as EXPORT_SCHEMA_VERSION, StoreError as ExportStoreError,
};

/// Registers every offer discovery slice.
pub fn register(bus: &mut Bus, platform: &Platform) {
    discover_offers::prelude::register(bus, platform);
    download_offer_page::prelude::register(bus, platform);
    extract_offer_page::prelude::register(bus, platform);
    normalize_offer::prelude::register(bus, platform);
    structuralize_offer::prelude::register(bus, platform);
    processing_for_export::prelude::register(bus, platform);
}
