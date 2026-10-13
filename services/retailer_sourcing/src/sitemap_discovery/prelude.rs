//! Everything other modules may use from sitemap discovery: its registration
//! and the events other contexts may react to.

use super::{consolidate_sitemap, process_sitemap, retrieve_sitemap};
use crate::platform::Platform;
use crate::platform::bus::Bus;

#[expect(
    unused_imports,
    reason = "the pipeline's final event; no other context reacts to it yet"
)]
pub use super::consolidate_sitemap::prelude::SitemapConsolidated;

/// Registers every sitemap discovery slice.
pub fn register(bus: &mut Bus, platform: &Platform) {
    retrieve_sitemap::prelude::register(bus, platform);
    process_sitemap::prelude::register(bus, platform);
    consolidate_sitemap::prelude::register(bus, platform);
}
