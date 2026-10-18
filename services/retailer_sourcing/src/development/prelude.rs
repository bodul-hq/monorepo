//! Everything other modules may use from development: its registration and
//! its routes.

use axum::middleware;
use utoipa_axum::router::OpenApiRouter;

use super::{offer_details, sitemaps};
use crate::platform::Platform;
use crate::platform::auth::require_bearer_token;
use crate::platform::bus::Bus;
use crate::platform::state::AppState;

/// Registers every development slice. There are none yet.
pub fn register(_bus: &mut Bus, _platform: &Platform) {}

/// Development's HTTP routes, to nest under `/api`. Each needs the bearer
/// token.
pub fn routes() -> OpenApiRouter<AppState> {
    sitemaps::routes()
        .merge(offer_details::routes())
        .route_layer(middleware::from_fn(require_bearer_token))
}
