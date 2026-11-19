//! Everything other modules may use from offer discovery: its registration and
//! the events other contexts may react to.

use discover_offers::prelude::register as discover_offers_register;
use download_offer_page::prelude::register as download_offer_page_register;
use extract_offer_page::prelude::register as extract_offer_page_register;
use normalize_offer::prelude::register as normalize_offer_register;
use processing_for_export::prelude::register as processing_for_export_register;
use structuralize_offer::prelude::register as structuralize_offer_register;

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

pub(crate) use super::processing_for_export::prelude::shopify_imports;
#[cfg(test)]
pub(crate) use super::processing_for_export::prelude::testing as export_testing;
pub use super::processing_for_export::prelude::{
    OfferExport, OfferExportWasPrepared, PgStore as ExportStore, StoreError as ExportStoreError,
};

/// Registers every offer discovery slice.
pub fn register(bus: &mut Bus, platform: &Platform) {
    discover_offers_register(bus, platform);
    download_offer_page_register(bus, platform);
    extract_offer_page_register(bus, platform);
    normalize_offer_register(bus, platform);
    structuralize_offer_register(bus, platform);
    processing_for_export_register(bus, platform);
}
