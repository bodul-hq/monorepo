//! Everything other modules may use from this slice.

use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

use super::super::Retailers;
use super::{api, handler, store};
use crate::platform::Platform;
use crate::platform::bus::Bus;
use crate::platform::state::AppState;

pub use super::event::SitemapRetrieved;

/// Registers the slice's handler.
pub fn register(bus: &mut Bus, platform: &Platform) {
    let store = store::PgStore::new(platform.db.clone(), platform.object_storage.clone());
    bus.handle(handler::Handler::new(
        store,
        platform.http.clone(),
        Retailers,
    ));
}

/// The slice's HTTP routes, to nest under `/api`.
pub fn routes() -> OpenApiRouter<AppState> {
    OpenApiRouter::new().routes(routes!(api::request_retrievals))
}
