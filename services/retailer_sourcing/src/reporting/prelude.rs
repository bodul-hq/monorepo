//! Everything other modules may use from reporting: its registration and HTTP routes.

use super::api::{__path_database_tables, database_tables};
use super::api::{__path_entry_statuses, entry_statuses};
use crate::platform::auth::require_bearer_token;
use crate::platform::state::AppState;
use axum::Extension;
use axum::middleware::from_fn;
use chrono::DateTime;
use chrono::Utc;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes as utoipa_axum_routes;

use crate::platform::Platform;
use crate::platform::bus::Bus;

/// Registers every reporting slice. There are none yet.
pub fn register(_bus: &mut Bus, _platform: &Platform) {}

/// Authenticated reporting routes, nested under /api.
pub fn routes() -> OpenApiRouter<AppState> {
    routes_with_clock(Utc::now)
}

#[derive(Clone)]
pub(super) struct ReportingClock(pub fn() -> DateTime<Utc>);

pub(super) fn routes_with_clock(clock: fn() -> DateTime<Utc>) -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(utoipa_axum_routes!(database_tables))
        .routes(utoipa_axum_routes!(entry_statuses))
        .layer(Extension(ReportingClock(clock)))
        .route_layer(from_fn(require_bearer_token))
}
