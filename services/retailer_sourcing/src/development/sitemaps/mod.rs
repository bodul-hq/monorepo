//! Sitemaps: inspecting the sitemap files the pipeline fetched.

mod api;

use api::{__path_list_sources, list_sources};

use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

use crate::platform::state::AppState;

/// The sitemap routes, to nest under `/api`.
pub fn routes() -> OpenApiRouter<AppState> {
    OpenApiRouter::new().routes(routes!(list_sources))
}
