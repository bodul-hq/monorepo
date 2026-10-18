//! HTTP endpoints that process queued commands and events on request.

use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

use super::bus::{BusHandle, RunReport};
use super::state::AppState;

/// The most items one request may process. A command can run for minutes (a
/// whole retailer's crawl), so keep runs small.
const MAX_LIMIT: usize = 1000;

#[derive(Deserialize, ToSchema)]
pub struct RunRequest {
    /// How many queued items to process, from 1 to 1000.
    limit: usize,
}

#[derive(Serialize, ToSchema)]
pub struct RunResponse {
    /// How many were processed, when all succeeded. `null` when some failed,
    /// since the failed ones are retried in a later run.
    processed: Option<usize>,
    /// One message per failure.
    errors: Vec<String>,
}

#[derive(Serialize, ToSchema)]
pub struct ErrorBody {
    error: String,
}

/// The bus's HTTP routes, to nest under `/api`.
pub fn routes() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(run_commands))
        .routes(routes!(run_events))
}

/// Processes up to `limit` queued commands that are due, and waits for them.
#[utoipa::path(
    post,
    path = "/commands/run",
    request_body = RunRequest,
    responses(
        (status = 200, description = "Run finished", body = RunResponse),
        (status = 400, description = "Limit out of range", body = ErrorBody),
        (status = 503, description = "The command bus isn't running (no DATABASE_URL)", body = ErrorBody),
    )
)]
async fn run_commands(
    State(bus): State<Option<BusHandle>>,
    Json(request): Json<RunRequest>,
) -> Response {
    run(bus, request, |bus, limit| bus.run_commands(limit)).await
}

/// Processes up to `limit` queued events that are due, and waits for them.
#[utoipa::path(
    post,
    path = "/events/run",
    request_body = RunRequest,
    responses(
        (status = 200, description = "Run finished", body = RunResponse),
        (status = 400, description = "Limit out of range", body = ErrorBody),
        (status = 503, description = "The command bus isn't running (no DATABASE_URL)", body = ErrorBody),
    )
)]
async fn run_events(
    State(bus): State<Option<BusHandle>>,
    Json(request): Json<RunRequest>,
) -> Response {
    run(bus, request, |bus, limit| bus.run_events(limit)).await
}

async fn run(
    bus: Option<BusHandle>,
    request: RunRequest,
    work: fn(&BusHandle, usize) -> RunReport,
) -> Response {
    if !(1..=MAX_LIMIT).contains(&request.limit) {
        let message = format!("limit must be between 1 and {MAX_LIMIT}");
        return error(StatusCode::BAD_REQUEST, message);
    }
    let Some(bus) = bus else {
        let message = "the command bus is not running; start the service with DATABASE_URL";
        return error(StatusCode::SERVICE_UNAVAILABLE, message.to_string());
    };

    // Handlers and Mulac's queries block, so run them off the async runtime.
    match tokio::task::spawn_blocking(move || work(&bus, request.limit)).await {
        Ok(report) => Json(RunResponse {
            processed: report.processed,
            errors: report.errors,
        })
        .into_response(),
        Err(join) => error(StatusCode::INTERNAL_SERVER_ERROR, join.to_string()),
    }
}

fn error(status: StatusCode, error: String) -> Response {
    (status, Json(ErrorBody { error })).into_response()
}
