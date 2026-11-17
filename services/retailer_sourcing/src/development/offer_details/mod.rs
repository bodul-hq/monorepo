//! Offer details: inspecting the offer pages the pipeline downloaded.

mod api;

use api::{__path_list_sources, list_sources};

use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

use crate::platform::state::AppState;

/// The offer detail routes, to nest under `/api`.
pub fn routes() -> OpenApiRouter<AppState> {
    OpenApiRouter::new().routes(routes!(list_sources))
}
