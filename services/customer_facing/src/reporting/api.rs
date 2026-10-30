use poem::{Endpoint, EndpointExt, Error, Result, http::StatusCode};
use poem_openapi::{OpenApi, SecurityScheme, auth::Bearer, payload::Json};

use super::{DatabaseTables, EntryStatuses, ReportError, Reporting};

/// Override with CUSTOMER_FACING_REPORTING_TOKEN on the service.
pub const DEFAULT_REPORTING_TOKEN: &str = "customer-facing-development-token";

#[derive(SecurityScheme)]
#[oai(ty = "bearer")]
struct ReportingBearer(Bearer);

pub struct ReportingApi {
    reporting: Reporting,
    token: String,
}

impl ReportingApi {
    pub fn new(reporting: Reporting, token: String) -> Self {
        Self { reporting, token }
    }

    fn authorize(&self, bearer: ReportingBearer) -> Result<()> {
        if self.token.is_empty() || bearer.0.token != self.token {
            return Err(report_error(StatusCode::UNAUTHORIZED));
        }
        Ok(())
    }
}

#[OpenApi]
impl ReportingApi {
    /// Estimated row counts and storage for ordinary tables in the active schema.
    #[oai(
        path = "/reporting/database-tables",
        method = "get",
        transform = "no_store"
    )]
    async fn database_tables(&self, bearer: ReportingBearer) -> Result<Json<DatabaseTables>> {
        self.authorize(bearer)?;
        self.reporting
            .database_tables()
            .await
            .map(Json)
            .map_err(load_error)
    }

    /// Inbox, command, event and outbox counts scheduled for today (UTC), grouped by current status.
    #[oai(
        path = "/reporting/entry-statuses",
        method = "get",
        transform = "no_store"
    )]
    async fn entry_statuses(&self, bearer: ReportingBearer) -> Result<Json<EntryStatuses>> {
        self.authorize(bearer)?;
        self.reporting
            .entry_statuses()
            .await
            .map(Json)
            .map_err(load_error)
    }
}

// Include extractor failures (missing or malformed bearer headers) in the no-store policy.
fn no_store(endpoint: impl Endpoint) -> impl Endpoint {
    endpoint.map_to_response().after(|result| async move {
        let mut response = result.unwrap_or_else(Error::into_response);
        response
            .headers_mut()
            .insert("cache-control", "no-store".parse().unwrap());
        Ok::<_, Error>(response)
    })
}

fn load_error(error: ReportError) -> Error {
    report_error(match error {
        ReportError::Unavailable => StatusCode::SERVICE_UNAVAILABLE,
        ReportError::Query => StatusCode::INTERNAL_SERVER_ERROR,
    })
}

fn report_error(status: StatusCode) -> Error {
    let message = match status {
        StatusCode::UNAUTHORIZED => "Invalid bearer token.",
        StatusCode::SERVICE_UNAVAILABLE => "Database is unavailable.",
        _ => "Unable to generate the database report.",
    };
    Error::from_response(
        poem::Response::builder()
            .status(status)
            .content_type("application/json")
            .body(serde_json::json!({"error": message}).to_string()),
    )
}
