//! Everything other modules may use from reporting: its registration and HTTP routes.

use crate::platform::Platform;
use crate::platform::bus::Bus;

/// Registers every reporting slice. There are none yet.
pub fn register(_bus: &mut Bus, _platform: &Platform) {}

/// Authenticated reporting routes, nested under /api.
pub fn routes() -> utoipa_axum::router::OpenApiRouter<crate::platform::state::AppState> {
    utoipa_axum::router::OpenApiRouter::new()
        .routes(utoipa_axum::routes!(super::api::database_tables))
        .route_layer(axum::middleware::from_fn(
            crate::platform::auth::require_bearer_token,
        ))
}
