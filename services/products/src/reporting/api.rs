use chrono::DateTime;
use chrono::Utc;
use diesel::PgConnection;
use diesel::QueryResult;
use poem::Response as PoemResponse;
use serde_json::json as serde_json_json;
use std::time::Duration;
use store::load as store_load;
use store::load_entry_statuses_at;
use tokio::task::spawn_blocking;

use poem::{Error, Result, http::StatusCode};
use poem_openapi::{
    OpenApi, SecurityScheme,
    auth::Bearer,
    payload::{Json, Response},
};
use testimony_infra::prelude::DbPool;

use super::store::{self, DatabaseTables, EntryStatuses};

/// Development default; override with PRODUCTS_REPORTING_TOKEN on the service.
pub const DEFAULT_REPORTING_TOKEN: &str = "products-development-token";

#[derive(SecurityScheme)]
#[oai(ty = "bearer")]
struct ReportingBearer(Bearer);

pub struct ReportingApi {
    pool: DbPool,
    token: String,
    clock: fn() -> DateTime<Utc>,
}

impl ReportingApi {
    pub fn new(pool: DbPool, token: String) -> Self {
        Self::with_clock(pool, token, Utc::now)
    }

    pub(crate) fn with_clock(pool: DbPool, token: String, clock: fn() -> DateTime<Utc>) -> Self {
        Self { pool, token, clock }
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
    #[oai(path = "/reporting/database-tables", method = "get")]
    async fn database_tables(
        &self,
        bearer: ReportingBearer,
    ) -> Result<Response<Json<DatabaseTables>>> {
        self.authorize(bearer)?;
        let report = report(self.pool.clone(), store_load).await?;
        Ok(Response::new(Json(report)).header("cache-control", "no-store"))
    }

    /// Exact inbox, command, event and outbox counts scheduled for today (UTC).
    #[oai(path = "/reporting/entry-statuses", method = "get")]
    async fn entry_statuses(
        &self,
        bearer: ReportingBearer,
    ) -> Result<Response<Json<EntryStatuses>>> {
        self.authorize(bearer)?;
        let clock = self.clock;
        let report = report(self.pool.clone(), move |connection| {
            load_entry_statuses_at(connection, clock())
        })
        .await?;
        Ok(Response::new(Json(report)).header("cache-control", "no-store"))
    }
}

async fn report<T: Send + 'static>(
    pool: DbPool,
    load: impl FnOnce(&mut PgConnection) -> QueryResult<T> + Send + 'static,
) -> Result<T> {
    spawn_blocking(move || {
        let mut connection = pool
            .get_timeout(Duration::from_secs(3))
            .map_err(|_| report_error(StatusCode::SERVICE_UNAVAILABLE))?;
        load(&mut connection).map_err(|_| {
            use diesel::sql_query;

            use diesel::RunQueryDsl;
            let status = if sql_query("SELECT 1").execute(&mut connection).is_err() {
                StatusCode::SERVICE_UNAVAILABLE
            } else {
                StatusCode::INTERNAL_SERVER_ERROR
            };
            report_error(status)
        })
    })
    .await
    .map_err(|_| report_error(StatusCode::INTERNAL_SERVER_ERROR))?
}

fn report_error(status: StatusCode) -> Error {
    let message = match status {
        StatusCode::UNAUTHORIZED => "Invalid bearer token.",
        StatusCode::SERVICE_UNAVAILABLE => "Database is unavailable.",
        _ => "Unable to generate the database report.",
    };
    Error::from_response(
        PoemResponse::builder()
            .status(status)
            .content_type("application/json")
            .header("cache-control", "no-store")
            .body(serde_json_json!({"error": message}).to_string()),
    )
}

#[cfg(test)]
mod tests {
    use crate::app;
    use crate::start_mulac;
    use crate::test_pool;
    use serde_json::Value;
    use serde_json::json as serde_json_json2;

    use super::*;
    use poem::test::TestClient;

    #[tokio::test]
    async fn reports_require_the_configured_token_and_database() {
        let client = TestClient::new(app(
            start_mulac(test_pool()).unwrap().1,
            "test-token".into(),
        ));
        for path in [
            "/api/reporting/database-tables",
            "/api/reporting/entry-statuses",
        ] {
            client
                .get(path)
                .send()
                .await
                .assert_status(StatusCode::UNAUTHORIZED);
            client
                .get(path)
                .header("authorization", "Bearer wrong")
                .send()
                .await
                .assert_status(StatusCode::UNAUTHORIZED);
            let response = client
                .get(path)
                .header("authorization", "Bearer test-token")
                .send()
                .await;
            response.assert_status(StatusCode::SERVICE_UNAVAILABLE);
            assert_eq!(response.0.headers()["cache-control"], "no-store");
            response
                .assert_json(serde_json_json2!({"error":"Database is unavailable."}))
                .await;
        }
        let response = client.get("/api/openapi.json").send().await;
        let spec: Value = response.json().await.value().deserialize();
        for path in ["/reporting/database-tables", "/reporting/entry-statuses"] {
            assert!(spec["paths"][path]["get"]["security"].is_array());
            assert!(spec["paths"][path]["get"]["responses"]["200"].is_object());
        }
    }
}
