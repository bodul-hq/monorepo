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
use crate::platform::bus::{BusHandle, DispatchError};

#[derive(Deserialize, ToSchema)]
pub struct RetrievalRequest {
    /// Retailer codes, such as `ugreenca`.
    retailers: Vec<String>,
}

#[derive(Serialize, ToSchema)]
pub struct RetrievalResponse {
    retrievals: Vec<QueuedRetrieval>,
}

#[derive(Serialize, ToSchema)]
pub struct QueuedRetrieval {
    retailer: String,
    #[schema(value_type = String, format = "uuid")]
    retrieval_id: Uuid,
}

#[derive(Serialize, ToSchema)]
pub struct ErrorBody {
    error: String,
}

/// Queues a sitemap retrieval for each posted retailer. The retrievals run in
/// the background; each one is stored as a `RetrieveSitemap` command in
/// `command_entries`.
#[utoipa::path(
    post,
    path = "/sitemap-retrievals",
    request_body = RetrievalRequest,
    responses(
        (status = 202, description = "Retrievals queued", body = RetrievalResponse),
        (status = 400, description = "Empty list, unknown retailer or retailer without sitemaps", body = ErrorBody),
        (status = 500, description = "A retrieval couldn't be queued", body = ErrorBody),
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
        for command in &commands {
            bus.dispatcher().dispatch(command, "api")?;
        }
        Ok::<_, DispatchError>(commands)
    })
    .await;

    match queued {
        Ok(Ok(commands)) => {
            let retrievals = commands
                .into_iter()
                .map(|command| QueuedRetrieval {
                    retailer: command.retailer_code().as_str(),
                    retrieval_id: command.retrieval_id(),
                })
                .collect();
            (StatusCode::ACCEPTED, Json(RetrievalResponse { retrievals })).into_response()
        }
        Ok(Err(dispatch)) => error(StatusCode::INTERNAL_SERVER_ERROR, dispatch.to_string()),
        Err(join) => error(StatusCode::INTERNAL_SERVER_ERROR, join.to_string()),
    }
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
