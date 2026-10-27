//! Queue routes delegate validation and blocking execution to Mulac's adapters.

use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use mulac_axum::io::{
    CommandsRunResponse, EventsRunResponse, OutboxAdapter, OutboxHttpError, OutboxRunResponse,
    RunCommands, RunEvents, RunOutbox,
};
use serde::Serialize;
use utoipa::ToSchema;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

use super::bus::BusHandle;
use super::state::AppState;

#[derive(Serialize, ToSchema)]
pub struct ErrorBody {
    error: String,
}

/// The command, event and outbox runner routes, to nest under `/api`.
pub fn routes() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(run_commands))
        .routes(routes!(run_events))
        .routes(routes!(run_outbox))
}

/// Processes up to `limit` queued commands that are due, and waits for them.
#[utoipa::path(
    post,
    path = "/commands/run",
    request_body = RunCommands,
    responses(
        (status = 200, description = "Run finished, including any item failures", body = CommandsRunResponse),
        (status = 400, description = "Invalid JSON or batch limit", body = String, content_type = "text/plain"),
        (status = 413, description = "Request body too large", body = String, content_type = "text/plain"),
        (status = 415, description = "JSON content type required", body = String, content_type = "text/plain"),
        (status = 500, description = "Queue processing task failed", body = String, content_type = "text/plain"),
        (status = 503, description = "The command bus isn't running (no DATABASE_URL)", body = ErrorBody),
    )
)]
async fn run_commands(State(bus): State<Option<BusHandle>>, request: RunCommands) -> Response {
    let Some(bus) = bus else {
        return unavailable_bus();
    };
    bus.commands.run(request).await.into_response()
}

/// Processes up to `limit` queued events that are due, and waits for them.
#[utoipa::path(
    post,
    path = "/events/run",
    request_body = RunEvents,
    responses(
        (status = 200, description = "Run finished, including any item failures", body = EventsRunResponse),
        (status = 400, description = "Invalid JSON or batch limit", body = String, content_type = "text/plain"),
        (status = 413, description = "Request body too large", body = String, content_type = "text/plain"),
        (status = 415, description = "JSON content type required", body = String, content_type = "text/plain"),
        (status = 500, description = "Queue processing task failed", body = String, content_type = "text/plain"),
        (status = 503, description = "The command bus isn't running (no DATABASE_URL)", body = ErrorBody),
    )
)]
async fn run_events(State(bus): State<Option<BusHandle>>, request: RunEvents) -> Response {
    let Some(bus) = bus else {
        return unavailable_bus();
    };
    bus.events.run(request).await.into_response()
}

/// Delivers up to `limit` queued outbox messages through Mulac's adapter.
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

fn unavailable_bus() -> Response {
    let body = ErrorBody {
        error: "the command bus is not running; start the service with DATABASE_URL".to_owned(),
    };
    (StatusCode::SERVICE_UNAVAILABLE, Json(body)).into_response()
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod postgres_tests;

#[cfg(test)]
pub(crate) mod outbox_tests;

#[cfg(test)]
pub(crate) mod outbox_postgres_tests;
