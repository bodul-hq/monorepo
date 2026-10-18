use super::store::{self, DatabaseTables};
use crate::platform::db::DbPool;
use axum::{
    Json,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;

#[derive(Serialize, utoipa::ToSchema)]
pub(super) struct ErrorBody {
    error: &'static str,
}

#[utoipa::path(get, path = "/reporting/database-tables", tag = "reporting",
    security(("bearer" = [])), responses(
        (status = 200, description = "Active schema table storage and estimated live rows", body = DatabaseTables),
        (status = 401, description = "Invalid bearer token", body = ErrorBody),
        (status = 503, description = "Database unavailable", body = ErrorBody),
        (status = 500, description = "Unexpected reporting failure", body = ErrorBody)
    ))]
pub(super) async fn database_tables(State(pool): State<Option<DbPool>>) -> Response {
    let Some(pool) = pool else {
        return error(StatusCode::SERVICE_UNAVAILABLE);
    };
    let result = tokio::task::spawn_blocking(move || {
        let mut connection = pool
            .get_timeout(std::time::Duration::from_secs(3))
            .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
        store::load(&mut connection).map_err(|_| {
            use diesel::RunQueryDsl;
            if diesel::sql_query("SELECT 1")
                .execute(&mut connection)
                .is_err()
            {
                StatusCode::SERVICE_UNAVAILABLE
            } else {
                StatusCode::INTERNAL_SERVER_ERROR
            }
        })
    })
    .await;
    match result {
        Ok(Ok(report)) => ([("cache-control", "no-store")], Json(report)).into_response(),
        Ok(Err(status)) => error(status),
        Err(_) => error(StatusCode::INTERNAL_SERVER_ERROR),
    }
}
fn error(status: StatusCode) -> Response {
    let error = if status == StatusCode::SERVICE_UNAVAILABLE {
        "Database is unavailable."
    } else {
        "Unable to generate the database report."
    };
    (status, Json(ErrorBody { error })).into_response()
}

#[cfg(test)]
mod tests {
    use axum::{body::Body, http::Request};
    use tower::ServiceExt;
    #[tokio::test]
    async fn reporting_requires_auth_and_reports_missing_database() {
        let app = crate::app(Default::default());
        for (token, status) in [
            (None, 401),
            (Some("wrong"), 401),
            (Some(crate::platform::auth::BEARER_TOKEN), 503),
        ] {
            let mut request = Request::builder().uri("/api/reporting/database-tables");
            if let Some(token) = token {
                request = request.header("authorization", format!("Bearer {token}"));
            }
            let response = app
                .clone()
                .oneshot(request.body(Body::empty()).unwrap())
                .await
                .unwrap();
            assert_eq!(response.status(), status);
        }
    }
}
