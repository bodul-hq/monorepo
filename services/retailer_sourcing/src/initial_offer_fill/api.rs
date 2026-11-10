use super::{
    acquire_storefront_token::prelude::AcquireStorefrontToken,
    product_catalog_ids::command::FetchProductCatalogIdentifiers,
};
use crate::platform::bus::BusHandle;
use axum::{
    Json,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};
use shared::retailer::RetailerCode;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Deserialize, ToSchema)]
pub struct InitialFillRequest {
    /// Nonempty retailer list. Duplicates start only one run per request.
    retailers: Vec<String>,
}
#[derive(Debug, Serialize, ToSchema)]
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
#[derive(Serialize, ToSchema)]
pub struct InitialFillQueueFailure {
    /// Opaque error ID; internal details are logged by the service.
    error: String,
    /// Confirmed successes before dispatch failed. Absent if the task failed
    /// unexpectedly and the enqueue outcomes are unknown.
    #[serde(skip_serializing_if = "Option::is_none")]
    fills: Option<Vec<QueuedInitialFill>>,
}

fn error(status: StatusCode, message: impl Into<String>) -> Response {
    (
        status,
        Json(InitialFillError {
            error: message.into(),
        }),
    )
        .into_response()
}

#[utoipa::path(post, path="/initial-offer-fills", request_body=InitialFillRequest, responses(
    (status=202, body=InitialFillResponse, description="Initial fills queued in request order"),
    (status=400, body=InitialFillError, description="Empty list, unknown or unsupported retailer"),
    (status=503, body=InitialFillError, description="Command bus unavailable"),
    (status=500, body=InitialFillQueueFailure, description="Queue failure (error ID); fills lists confirmed successes, or is absent when outcomes are unknown")
))]
pub async fn start(
    State(bus): State<Option<BusHandle>>,
    Json(request): Json<InitialFillRequest>,
) -> Response {
    // Validate the entire request before accessing the bus or dispatching work.
    let prepared = match prepare(&request.retailers) {
        Ok(prepared) => prepared,
        Err(message) => return error(StatusCode::BAD_REQUEST, message),
    };
    let Some(bus) = bus else {
        return error(
            StatusCode::SERVICE_UNAVAILABLE,
            "command bus is unavailable",
        );
    };
    let queued = tokio::task::spawn_blocking(move || {
        queue(prepared, |command, run_id| match command {
            InitialCommand::Markets(command) => bus.dispatcher().dispatch_run(command, run_id),
            InitialCommand::Token(command) => bus.dispatcher().dispatch_run(command, run_id),
            InitialCommand::Catalog(command) => bus.dispatcher().dispatch_run(command, run_id),
        })
    })
    .await;
    match queued {
        Ok(Ok(fills)) => {
            (StatusCode::ACCEPTED, Json(InitialFillResponse { fills })).into_response()
        }
        Ok(Err((failure, fills))) => {
            let id = Uuid::now_v7();
            tracing::error!(error_id=%id, error=?failure, "failed to queue initial fill");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(InitialFillQueueFailure {
                    error: id.to_string(),
                    fills: Some(fills),
                }),
            )
                .into_response()
        }
        Err(failure) => {
            let id = Uuid::now_v7();
            tracing::error!(error_id=%id, error=?failure, "initial fill queue task failed; enqueue outcomes unknown");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(InitialFillQueueFailure {
                    error: id.to_string(),
                    fills: None,
                }),
            )
                .into_response()
        }
    }
}

#[derive(Debug)]
enum InitialCommand {
    Markets(super::discover_markets::DiscoverStorefrontMarkets),
    Token(AcquireStorefrontToken),
    Catalog(FetchProductCatalogIdentifiers),
}

#[derive(Debug)]
struct PreparedFill {
    fill: QueuedInitialFill,
    command: InitialCommand,
}

fn prepare(codes: &[String]) -> Result<Vec<PreparedFill>, String> {
    if codes.is_empty() {
        return Err("retailers must not be empty".into());
    }
    let mut prepared: Vec<PreparedFill> = Vec::new();
    for code in codes {
        let retailer_code =
            RetailerCode::from_str(code).map_err(|_| format!("unknown retailer code: {code}"))?;
        let config = retailer_data_extractor::initial_fill_config_for(retailer_code)
            .ok_or_else(|| format!("initial fill is not implemented for retailer {code}"))?;
        // Configuration gates scaffolds; the executable flow must match the declared process.
        let process = retailer_data_extractor::initial_fill_process(retailer_code);
        let mut expected = if retailer_data_extractor::all_market_contexts(retailer_code) {
            vec![
                retailer_data_extractor::Step::DiscoverMarkets,
                retailer_data_extractor::Step::ProductCatalogIdentifiers,
                retailer_data_extractor::Step::OfferDetails,
            ]
        } else if config.requires_token() {
            vec![
                retailer_data_extractor::Step::AcquireStorefrontToken,
                retailer_data_extractor::Step::ProductCatalogIdentifiers,
                retailer_data_extractor::Step::OfferDetails,
            ]
        } else {
            vec![
                retailer_data_extractor::Step::ProductCatalogIdentifiers,
                retailer_data_extractor::Step::OfferDetails,
            ]
        };
        if expected.first() != Some(&retailer_data_extractor::Step::AcquireStorefrontToken) {
            expected.insert(0, retailer_data_extractor::Step::AcquireStorefrontToken);
        }
        if process.steps != expected {
            return Err(format!(
                "unsupported initial fill process for retailer {code}"
            ));
        }
        let retailer = retailer_code.as_str();
        if prepared.iter().any(|p| p.fill.retailer == retailer) {
            continue;
        }
        let run_id = Uuid::now_v7();
        let command = if config.attempts_token() {
            InitialCommand::Token(AcquireStorefrontToken {
                run_id,
                retailer_code,
                url: config.storefront_url.into(),
            })
        } else if retailer_data_extractor::all_market_contexts(retailer_code) {
            InitialCommand::Markets(super::discover_markets::DiscoverStorefrontMarkets {
                storefront_token: None,
                run_id,
                retailer_code,
            })
        } else if config.requires_token() {
            InitialCommand::Token(AcquireStorefrontToken {
                run_id,
                retailer_code,
                url: config.storefront_url.into(),
            })
        } else {
            InitialCommand::Catalog(FetchProductCatalogIdentifiers {
                context: None,
                run_id,
                retailer_code,
                storefront_token: None,
                after: None,
            })
        };
        prepared.push(PreparedFill {
            fill: QueuedInitialFill { retailer, run_id },
            command,
        });
    }
    Ok(prepared)
}

/// Each dispatch is independent. Stop on failure and retain confirmed successes.
fn queue<E>(
    prepared: Vec<PreparedFill>,
    mut dispatch: impl FnMut(&InitialCommand, Uuid) -> Result<(), E>,
) -> Result<Vec<QueuedInitialFill>, (E, Vec<QueuedInitialFill>)> {
    let mut fills = Vec::with_capacity(prepared.len());
    for prepared in prepared {
        if let Err(failure) = dispatch(&prepared.command, prepared.fill.run_id) {
            return Err((failure, fills));
        }
        fills.push(prepared.fill);
    }
    Ok(fills)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn prepare_codes(codes: &[&str]) -> Vec<PreparedFill> {
        prepare(
            &codes
                .iter()
                .map(|code| (*code).to_owned())
                .collect::<Vec<_>>(),
        )
        .unwrap()
    }

    #[test]
    fn queue_preserves_order_deduplicates_and_dispatches_each_start_with_its_run_id() {
        let prepared = prepare_codes(&["ankerau", "ankeritalycom", "ankerau", "ankerjapancom"]);
        let mut dispatched = Vec::new();
        let fills = queue(prepared, |command, run_id| {
            match command {
                InitialCommand::Markets(_) => panic!("Anker must retain its fixed context"),
                InitialCommand::Token(command) => {
                    assert!(matches!(
                        command.retailer_code,
                        RetailerCode::AnkerAu
                            | RetailerCode::AnkerItalyCom
                            | RetailerCode::AnkerJapanCom
                    ));
                    assert_eq!(
                        command.url,
                        retailer_data_extractor::initial_fill_config_for(command.retailer_code)
                            .unwrap()
                            .storefront_url
                    );
                    assert_eq!(command.run_id, run_id);
                }
                InitialCommand::Catalog(command) => {
                    assert!(matches!(
                        command.retailer_code,
                        RetailerCode::AnkerItalyCom | RetailerCode::AnkerJapanCom
                    ));
                    assert_eq!(command.run_id, run_id);
                    assert!(command.storefront_token.is_none());
                    assert!(command.after.is_none());
                }
            }
            dispatched.push(run_id);
            Ok::<(), ()>(())
        })
        .unwrap();
        assert_eq!(
            fills
                .iter()
                .map(|f| f.retailer.as_str())
                .collect::<Vec<_>>(),
            ["ankerau", "ankeritalycom", "ankerjapancom"]
        );
        assert_eq!(
            fills.iter().map(|f| f.run_id).collect::<Vec<_>>(),
            dispatched
        );
        assert_eq!(
            dispatched
                .iter()
                .collect::<std::collections::HashSet<_>>()
                .len(),
            3
        );
        let next = prepare_codes(&["ankerau"]);
        assert!(!dispatched.contains(&next[0].fill.run_id));
    }

    #[test]
    fn queue_failure_returns_prior_successes_and_stops_dispatching() {
        for fail_at in 0..3 {
            let prepared = prepare_codes(&["ankerau", "ankerca", "ankeritalycom"]);
            let ids: Vec<_> = prepared.iter().map(|p| p.fill.run_id).collect();
            let mut calls = 0;
            let (failure, fills) = queue(prepared, |_, _| {
                let current = calls;
                calls += 1;
                if current == fail_at {
                    Err("fixture failure")
                } else {
                    Ok(())
                }
            })
            .unwrap_err();
            assert_eq!(failure, "fixture failure");
            assert_eq!(calls, fail_at + 1);
            assert_eq!(
                fills.iter().map(|f| f.run_id).collect::<Vec<_>>(),
                ids[..fail_at]
            );
            let body = serde_json::to_value(InitialFillQueueFailure {
                error: "opaque-id".into(),
                fills: Some(fills),
            })
            .unwrap();
            assert_eq!(body["fills"].as_array().unwrap().len(), fail_at);
            assert_eq!(body["error"], "opaque-id");
        }
        let unknown = serde_json::to_value(InitialFillQueueFailure {
            error: "opaque-id".into(),
            fills: None,
        })
        .unwrap();
        assert_eq!(unknown, json!({"error":"opaque-id"}));
    }
}

#[cfg(test)]
mod market_tests {
    use super::*;
    #[test]
    fn every_new_retailer_starts_token_resolution() {
        let codes: Vec<String> = RetailerCode::ALL
            .iter()
            .copied()
            .filter(|r| retailer_data_extractor::all_market_contexts(*r))
            .map(|r| r.as_str())
            .collect();
        assert_eq!(codes.len(), 31);
        let fills = prepare(&codes).unwrap();
        assert_eq!(fills.len(), 31);
        for fill in fills {
            assert!(matches!(fill.command, InitialCommand::Token(_)));
        }
    }
}
