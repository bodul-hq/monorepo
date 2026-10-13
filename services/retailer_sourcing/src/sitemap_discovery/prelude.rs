//! Everything other modules may use from sitemap discovery: its registration,
//! its routes, and the events other contexts may react to.

use utoipa_axum::router::OpenApiRouter;

use super::{consolidate_sitemap, process_sitemap, retrieve_sitemap};
use crate::platform::Platform;
use crate::platform::bus::{Bus, BusHandle};

pub use super::consolidate_sitemap::prelude::SitemapConsolidated;

/// Registers every sitemap discovery slice.
pub fn register(bus: &mut Bus, platform: &Platform) {
    retrieve_sitemap::prelude::register(bus, platform);
    process_sitemap::prelude::register(bus, platform);
    consolidate_sitemap::prelude::register(bus, platform);
}

/// Sitemap discovery's HTTP routes, to nest under `/api`.
pub fn routes() -> OpenApiRouter<Option<BusHandle>> {
    retrieve_sitemap::prelude::routes()
}
