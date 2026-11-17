//! Axum extraction, route registration and response rendering.
use super::{
    InitialFillError, InitialFillHttpResponse, InitialFillQueueFailure, InitialFillRequest,
    InitialFillResponse,
};
use crate::platform::{bus::BusHandle, state::AppState};
use crate::v2::application::prelude::run;
use axum::{
    Json,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use tracing::error;
use utoipa_axum::{router::OpenApiRouter, routes};

#[utoipa::path(post, path="/initial-offer-fills", request_body=InitialFillRequest, responses(
    (status=202, body=InitialFillResponse, description="Initial fills queued in request order"),
    (status=400, body=InitialFillError, description="Empty list, unknown or unsupported retailer"),
    (status=503, body=InitialFillError, description="Command bus unavailable"),
    (status=500, body=InitialFillQueueFailure, description="Queue failure (error ID); fills lists confirmed successes, or is empty when none were confirmed or outcomes are unknown")
))]
async fn start_initial_fills(
    State(bus): State<Option<BusHandle>>,
    Json(request): Json<InitialFillRequest>,
) -> Response {
    let Some(dispatcher) = bus.map(|bus| bus.dispatcher().clone()) else {
        error!(
            severity = "critical",
            "cannot start initial fills: command bus is unavailable"
        );

        return InitialFillHttpResponse::ServiceUnavailable(InitialFillError::new(
            "command bus is unavailable",
        ))
        .into_response();
    };

    let response = run(request.into(), dispatcher).await;

    InitialFillHttpResponse::from(response).into_response()
}

pub fn routes() -> OpenApiRouter<AppState> {
    OpenApiRouter::new().routes(routes!(start_initial_fills))
}

impl IntoResponse for InitialFillHttpResponse {
    fn into_response(self) -> Response {
        match self {
            Self::Accepted(body) => (StatusCode::ACCEPTED, Json(body)).into_response(),
            Self::BadRequest(body) => (StatusCode::BAD_REQUEST, Json(body)).into_response(),
            Self::ServiceUnavailable(body) => {
                (StatusCode::SERVICE_UNAVAILABLE, Json(body)).into_response()
            }
            Self::InternalServerError(body) => {
                (StatusCode::INTERNAL_SERVER_ERROR, Json(body)).into_response()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::to_bytes;
    use serde_json::Value;
    use serde_json::from_slice;
    use serde_json::json;

    #[tokio::test]
    async fn unavailable_bus_returns_503_before_retailer_validation() {
        for retailers in [vec![], vec!["unknown".into()], vec!["ankerau".into()]] {
            let response =
                start_initial_fills(State(None), Json(InitialFillRequest { retailers })).await;
            assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
            let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
            assert_eq!(
                from_slice::<Value>(&body).unwrap(),
                json!({"error": "command bus is unavailable"}),
            );
        }
    }
}
