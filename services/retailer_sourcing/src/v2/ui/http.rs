//! Shared HTTP request/response contracts and application outcome mapping.
pub(super) mod axum;

use crate::v2::application::prelude::{FillRun, InitialFillOrder, InitialFillStatus};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Deserialize, ToSchema)]
pub(crate) struct InitialFillRequest {
    /// Nonempty retailer list. Duplicate retailers are rejected.
    retailers: Vec<String>,
}
#[derive(Serialize, ToSchema)]
pub struct QueuedInitialFill {
    retailer: String,
    #[schema(value_type=String, format="uuid")]
    run_id: Uuid,
}
#[derive(Serialize, ToSchema)]
pub struct InitialFillResponse {
    /// Queued runs, in first-occurrence request order.
    fills: Vec<QueuedInitialFill>,
}
#[derive(Serialize, ToSchema)]
pub struct InitialFillError {
    error: String,
}

impl InitialFillError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            error: message.into(),
        }
    }
}

#[derive(Serialize, ToSchema)]
pub struct InitialFillQueueFailure {
    /// Opaque error ID; internal details are logged by the service.
    error: String,
    /// Confirmed successes before dispatch failed. Empty when none were
    /// confirmed, including when the task failed unexpectedly and the enqueue
    /// outcomes are unknown.
    fills: Vec<QueuedInitialFill>,
}

impl InitialFillQueueFailure {
    fn new(error_id: Uuid, fills: Vec<QueuedInitialFill>) -> Self {
        Self {
            error: error_id.to_string(),
            fills,
        }
    }
}

impl From<FillRun> for QueuedInitialFill {
    fn from(fill: FillRun) -> Self {
        Self {
            retailer: fill.retailer,
            run_id: fill.run_id,
        }
    }
}

impl From<InitialFillRequest> for InitialFillOrder {
    fn from(request: InitialFillRequest) -> Self {
        Self {
            retailers: request.retailers,
        }
    }
}

/// HTTP outcomes rendered by each framework adapter.
pub(crate) enum InitialFillHttpResponse {
    Accepted(InitialFillResponse),
    BadRequest(InitialFillError),
    ServiceUnavailable(InitialFillError),
    InternalServerError(InitialFillQueueFailure),
}

impl From<InitialFillStatus> for InitialFillHttpResponse {
    fn from(response: InitialFillStatus) -> Self {
        match response {
            InitialFillStatus::Queued(fills) => {
                let fills = fills.into_iter().map(Into::into).collect();

                Self::Accepted(InitialFillResponse { fills })
            }
            InitialFillStatus::InvalidRequest(error) => {
                Self::BadRequest(InitialFillError::new(error.to_string()))
            }
            InitialFillStatus::QueueFailed(failure) => {
                let fills = failure.fills.into_iter().map(Into::into).collect();

                Self::InternalServerError(InitialFillQueueFailure::new(failure.error_id, fills))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use serde_json::to_value;

    fn fills(count: usize) -> Vec<QueuedInitialFill> {
        (0..count)
            .map(|_| QueuedInitialFill {
                retailer: "ankerau".into(),
                run_id: Uuid::now_v7(),
            })
            .collect()
    }

    #[test]
    fn queue_failure_lists_confirmed_fills() {
        let body = InitialFillQueueFailure::new(Uuid::nil(), fills(2));
        let body = to_value(body).unwrap();

        assert_eq!(body["fills"].as_array().unwrap().len(), 2);
        assert_eq!(body["error"], Uuid::nil().to_string());
    }

    #[test]
    fn queue_failure_lists_empty_fills() {
        let body = InitialFillQueueFailure::new(Uuid::nil(), Vec::new());
        let body = to_value(body).unwrap();

        assert_eq!(body, json!({"error": Uuid::nil().to_string(), "fills": []}));
    }
}
