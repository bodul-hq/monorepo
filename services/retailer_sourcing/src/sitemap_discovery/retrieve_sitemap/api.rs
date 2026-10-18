//! HTTP entry point: queues sitemap retrievals for the posted retailers.

use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use bodul_shared::retailer::RetailerCode;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use super::super::Retailers;
use super::command::RetrieveSitemap;
use super::handler::RootSitemaps;
use crate::platform::bus::BusHandle;

#[derive(Deserialize, ToSchema)]
pub struct RetrievalRequest {
    /// Retailer codes, such as `ugreenca`.
    retailers: Vec<String>,
}

#[derive(Serialize, ToSchema)]
pub struct RetrievalResponse {
    /// One retrieval per posted retailer, in the order posted.
    retrievals: Vec<QueuedRetrieval>,
}

#[derive(Serialize, ToSchema)]
pub struct QueuedRetrieval {
    /// The retailer's code, such as `ugreenca`.
    retailer: String,
    /// Identifies the retrieval's stored sitemap files.
    #[schema(value_type = String, format = "uuid")]
    retrieval_id: Uuid,
}

#[derive(Serialize, ToSchema)]
pub struct ErrorBody {
    /// What went wrong.
    error: String,
}

#[derive(Serialize, ToSchema)]
pub struct QueueFailure {
    /// Why a retrieval couldn't be queued.
    error: String,
    /// The retrievals queued before the failure. They stay queued and will run.
    retrievals: Vec<QueuedRetrieval>,
}

/// Queues a sitemap retrieval for each posted retailer, as a `RetrieveSitemap`
/// command in `command_entries`. Nothing runs them in the background: they run
/// when `POST /commands/run` processes the queue.
#[utoipa::path(
    post,
    path = "/sitemap-retrievals",
    request_body = RetrievalRequest,
    responses(
        (status = 202, description = "Retrievals queued", body = RetrievalResponse),
        (status = 400, description = "Empty list, unknown retailer or retailer without sitemaps", body = ErrorBody),
        (status = 500, description = "A retrieval couldn't be queued; lists those queued before it", body = QueueFailure),
        (status = 503, description = "The command bus isn't running (no DATABASE_URL)", body = ErrorBody),
    )
)]
pub async fn request_retrievals(
    State(bus): State<Option<BusHandle>>,
    Json(request): Json<RetrievalRequest>,
) -> Response {
    let retailer_codes = match parse_retailers(&request.retailers, &Retailers) {
        Ok(retailer_codes) => retailer_codes,
        Err(message) => return error(StatusCode::BAD_REQUEST, message),
    };
    let Some(bus) = bus else {
        return error(
            StatusCode::SERVICE_UNAVAILABLE,
            "the command bus is not running; start the service with DATABASE_URL".to_string(),
        );
    };

    let commands: Vec<RetrieveSitemap> = retailer_codes
        .into_iter()
        .map(RetrieveSitemap::new)
        .collect();

    // Queuing writes to Postgres through a blocking connection.
    let queued = tokio::task::spawn_blocking(move || {
        queue(commands, |command| {
            bus.dispatcher().dispatch(command, "api")
        })
    })
    .await;

    match queued {
        Ok(Ok(retrievals)) => {
            (StatusCode::ACCEPTED, Json(RetrievalResponse { retrievals })).into_response()
        }
        // Each command is its own insert, so those before the failure stay
        // queued; the caller needs them to avoid queuing them twice.
        Ok(Err((dispatch, retrievals))) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(QueueFailure {
                error: dispatch.to_string(),
                retrievals,
            }),
        )
            .into_response(),
        Err(join) => error(StatusCode::INTERNAL_SERVER_ERROR, join.to_string()),
    }
}

/// Queues `commands` in order, stopping at the first that fails. Returns the
/// retrievals queued, along with the error when one failed.
fn queue<E>(
    commands: Vec<RetrieveSitemap>,
    mut dispatch: impl FnMut(&RetrieveSitemap) -> Result<(), E>,
) -> Result<Vec<QueuedRetrieval>, (E, Vec<QueuedRetrieval>)> {
    let mut retrievals = Vec::with_capacity(commands.len());
    for command in commands {
        if let Err(error) = dispatch(&command) {
            return Err((error, retrievals));
        }
        retrievals.push(QueuedRetrieval {
            retailer: command.retailer_code().as_str(),
            retrieval_id: command.retrieval_id(),
        });
    }
    Ok(retrievals)
}

/// Turns the posted codes into retailers with sitemaps, dropping duplicates.
fn parse_retailers(
    codes: &[String],
    retailers: &impl RootSitemaps,
) -> Result<Vec<RetailerCode>, String> {
    if codes.is_empty() {
        return Err("retailers must not be empty".to_string());
    }

    let mut retailer_codes = Vec::new();
    for code in codes {
        let retailer_code =
            RetailerCode::try_from(code.as_str()).map_err(|error| error.to_string())?;
        if retailers.root_sitemaps(&retailer_code).is_none() {
            return Err(format!("no sitemap configuration for retailer {code}"));
        }
        if !retailer_codes.contains(&retailer_code) {
            retailer_codes.push(retailer_code);
        }
    }

    Ok(retailer_codes)
}

fn error(status: StatusCode, error: String) -> Response {
    (status, Json(ErrorBody { error })).into_response()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn queue_failure_returns_only_prior_successes_and_stops_dispatching() {
        for fail_at in 0..3 {
            let commands: Vec<_> = [
                RetailerCode::AnkerAu,
                RetailerCode::AnkerCa,
                RetailerCode::UgreenCa,
            ]
            .into_iter()
            .map(RetrieveSitemap::new)
            .collect();
            let ids: Vec<_> = commands.iter().map(RetrieveSitemap::retrieval_id).collect();
            let mut calls = 0;
            let result = queue(commands, |_| {
                let current = calls;
                calls += 1;
                if current == fail_at {
                    Err("queue failed")
                } else {
                    Ok(())
                }
            });
            let Err((error, queued)) = result else {
                panic!("dispatch should fail")
            };
            assert_eq!(error, "queue failed");
            assert_eq!(calls, fail_at + 1);
            assert_eq!(
                queued.iter().map(|r| r.retrieval_id).collect::<Vec<_>>(),
                ids[..fail_at]
            );
            let body = serde_json::to_value(QueueFailure {
                error: error.into(),
                retrievals: queued,
            })
            .unwrap();
            assert_eq!(body["error"], "queue failed");
            assert_eq!(body["retrievals"].as_array().unwrap().len(), fail_at);
        }
    }

    #[test]
    fn successful_queue_preserves_request_order() {
        let commands: Vec<_> = [RetailerCode::AnkerAu, RetailerCode::UgreenCa]
            .into_iter()
            .map(RetrieveSitemap::new)
            .collect();
        let ids: Vec<_> = commands.iter().map(RetrieveSitemap::retrieval_id).collect();
        let Ok(queued) = queue(commands, |_| Ok::<(), ()>(())) else {
            panic!("dispatch should succeed")
        };
        assert_eq!(
            queued.iter().map(|r| r.retrieval_id).collect::<Vec<_>>(),
            ids
        );
        assert_eq!(
            queued
                .iter()
                .map(|r| r.retailer.as_str())
                .collect::<Vec<_>>(),
            ["ankerau", "ugreenca"]
        );
    }

    struct Roots(Vec<RetailerCode>);

    impl RootSitemaps for Roots {
        fn root_sitemaps(&self, retailer_code: &RetailerCode) -> Option<Vec<String>> {
            self.0
                .contains(retailer_code)
                .then(|| vec!["https://shop.test/sitemap.xml".to_string()])
        }
    }

    fn codes(codes: &[&str]) -> Vec<String> {
        codes.iter().map(|code| code.to_string()).collect()
    }

    #[test]
    fn accepts_known_retailers_once_each() {
        let roots = Roots(vec![RetailerCode::UgreenCa, RetailerCode::MiCom]);

        let parsed = parse_retailers(&codes(&["ugreenca", "micom", "ugreenca"]), &roots);

        assert_eq!(
            parsed,
            Ok(vec![RetailerCode::UgreenCa, RetailerCode::MiCom])
        );
    }

    #[test]
    fn rejects_an_empty_list() {
        assert_eq!(
            parse_retailers(&[], &Roots(Vec::new())),
            Err("retailers must not be empty".to_string())
        );
    }

    #[test]
    fn rejects_unknown_codes() {
        let parsed = parse_retailers(&codes(&["nope"]), &Roots(Vec::new()));

        assert_eq!(parsed, Err("unknown retailer code: nope".to_string()));
    }

    #[test]
    fn rejects_retailers_without_sitemaps() {
        let parsed = parse_retailers(&codes(&["ugreenca"]), &Roots(Vec::new()));

        assert_eq!(
            parsed,
            Err("no sitemap configuration for retailer ugreenca".to_string())
        );
    }
}
