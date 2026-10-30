//! Everything other modules may use from reporting: its registration and HTTP routes.

use crate::platform::Platform;
use crate::platform::bus::Bus;

/// Registers every reporting slice. There are none yet.
pub fn register(_bus: &mut Bus, _platform: &Platform) {}

/// Authenticated reporting routes, nested under /api.
pub fn routes() -> utoipa_axum::router::OpenApiRouter<crate::platform::state::AppState> {
    routes_with_clock(chrono::Utc::now)
}

#[derive(Clone)]
pub(super) struct ReportingClock(pub fn() -> chrono::DateTime<chrono::Utc>);

pub(super) fn routes_with_clock(
    clock: fn() -> chrono::DateTime<chrono::Utc>,
) -> utoipa_axum::router::OpenApiRouter<crate::platform::state::AppState> {
    utoipa_axum::router::OpenApiRouter::new()
        .routes(utoipa_axum::routes!(super::api::database_tables))
        .routes(utoipa_axum::routes!(super::api::entry_statuses))
        .layer(axum::Extension(ReportingClock(clock)))
        .route_layer(axum::middleware::from_fn(
            crate::platform::auth::require_bearer_token,
        ))
}
