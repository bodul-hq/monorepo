//! Authenticated, read-only database reports. No public reporting client.
mod store;

use crate::{error::Error, store::Store};
use axum::{
    Json, Router,
    extract::State,
    response::{IntoResponse, Response},
    routing::get,
};

pub(crate) fn routes() -> Router<Store> {
    Router::new().route("/api/reporting/database-tables", get(database_tables))
}
async fn database_tables(State(store): State<Store>) -> Result<Response, Error> {
    let report = store
        .blocking(|connection| self::store::load(connection).map_err(crate::store::database_error))
        .await?;
    Ok(([("cache-control", "no-store")], Json(report)).into_response())
}
