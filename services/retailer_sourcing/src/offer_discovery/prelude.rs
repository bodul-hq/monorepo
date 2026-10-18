//! Everything other modules may use from offer discovery: its registration and
//! the events other contexts may react to.

use super::{
    discover_offers, download_offer_page, extract_offer_page, normalize_offer, structuralize_offer,
};
use crate::platform::Platform;
use crate::platform::bus::Bus;

#[expect(
    unused_imports,
    reason = "offer discovery's final events; no other context reacts to them yet"
)]
pub use super::structuralize_offer::prelude::OfferWasStructuralized;

/// Registers every offer discovery slice.
pub fn register(bus: &mut Bus, platform: &Platform) {
    discover_offers::prelude::register(bus, platform);
    download_offer_page::prelude::register(bus, platform);
    extract_offer_page::prelude::register(bus, platform);
    normalize_offer::prelude::register(bus, platform);
    structuralize_offer::prelude::register(bus, platform);
}
