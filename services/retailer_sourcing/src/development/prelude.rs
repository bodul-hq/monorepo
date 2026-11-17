//! Everything other modules may use from development: its registration and
//! its routes.

use middleware::from_fn;
use offer_details::routes as offer_details_routes;
use offer_details_v2::routes as offer_details_v2_routes;
use sitemaps::routes as sitemaps_routes;

use axum::middleware;
use utoipa_axum::router::OpenApiRouter;

use super::{offer_details, offer_details_v2, sitemaps};
use crate::platform::Platform;
use crate::platform::auth::require_bearer_token;
use crate::platform::bus::Bus;
use crate::platform::state::AppState;

/// Registers every development slice. There are none yet.
pub fn register(_bus: &mut Bus, _platform: &Platform) {}

/// Development's HTTP routes, to nest under `/api`. Each needs the bearer
/// token.
pub fn routes() -> OpenApiRouter<AppState> {
    sitemaps_routes()
        .merge(offer_details_routes())
        .merge(offer_details_v2_routes())
        .route_layer(from_fn(require_bearer_token))
}
