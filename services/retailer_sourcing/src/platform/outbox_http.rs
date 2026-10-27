//! Explicit outbox delivery through Mulac's shared Axum adapter.

use axum::extract::State;
use mulac_axum::io::{OutboxAdapter, OutboxHttpError, OutboxRunResponse, RunOutbox};
use utoipa_axum::{router::OpenApiRouter, routes};

use super::state::AppState;

pub fn routes() -> OpenApiRouter<AppState> {
    OpenApiRouter::new().routes(routes!(run_outbox))
}

#[utoipa::path(post, path = "/outbox/run", request_body = RunOutbox,
    responses(
        (status = 200, description = "Batch result; item failures are reported in errors", body = OutboxRunResponse),
        (status = 400, description = "Invalid JSON or limit", body = String),
        (status = 413, description = "Request too large", body = String),
        (status = 415, description = "JSON content type required", body = String),
        (status = 500, description = "Queue processing task failed", body = String),
        (status = 503, description = "Database or offers publisher is not configured", body = String)
    ))]
async fn run_outbox(
    State(state): State<AppState>,
    request: RunOutbox,
) -> Result<OutboxRunResponse, OutboxHttpError> {
    state
        .outbox
        .unwrap_or_else(OutboxAdapter::unconfigured)
        .run(request)
        .await
}

#[cfg(test)]
pub(crate) mod postgres_tests;
#[cfg(test)]
pub(crate) mod tests;
