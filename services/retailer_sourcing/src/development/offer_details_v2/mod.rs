//! Offer details v2: inspecting initial-fill GraphQL artifacts.

mod api;

use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

use crate::platform::state::AppState;

/// The offer detail routes, to nest under `/api`.
pub fn routes() -> OpenApiRouter<AppState> {
    OpenApiRouter::new().routes(routes!(api::list_sources))
}
