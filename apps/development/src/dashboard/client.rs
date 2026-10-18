//! Reporting access stays private to the development server, never in a public client crate.
use reqwest::{
    Client, Url,
    header::{AUTHORIZATION, HeaderValue},
};
use serde::Deserialize;
use std::time::Duration;

const PATH: &str = "/api/reporting/database-tables";
const SOURCING_TOKEN: &str = "8bcb74f24405708e90b0aa1be5679f422ea2ba199ae8ab0d0f1e80acd8c9c97d";
const STORAGE_TOKEN: &str = "6fe0382d64bba74e205af86e397d902c0f5e6f3d8e15b1a4f87c6a309d752eb1";

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub(super) enum RowCountKind {
    Estimated,
}

#[derive(Debug, Deserialize)]
pub(super) struct Report {
    pub schema_name: String,
    pub generated_at: String,
    pub row_count_kind: RowCountKind,
    pub tables: Vec<TableStats>,
}
#[derive(Debug, Deserialize)]
pub(super) struct TableStats {
    pub table_name: String,
    pub row_count: u64,
    pub table_bytes: u64,
    pub index_bytes: u64,
    pub total_bytes: u64,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ReportError {
    Unavailable,
    Authentication,
    Malformed,
    Configuration,
}

struct Endpoint {
    url: Url,
    authorization: HeaderValue,
}
pub(super) struct ReportingClients {
    http: Client,
    sourcing: Result<Endpoint, ReportError>,
    storage: Result<Endpoint, ReportError>,
}
impl ReportingClients {
    pub(super) fn from_env() -> Result<Self, reqwest::Error> {
        let env =
            |name: &str, default: &str| std::env::var(name).unwrap_or_else(|_| default.into());
        Self::new(
            &env("RETAILER_SOURCING_URL", "http://172.28.100.40:3000"),
            &env("RETAILER_SOURCING_TOKEN", SOURCING_TOKEN),
            &env("OBJECT_STORAGE_URL", "http://172.28.100.43:3000"),
            &env("OBJECT_STORAGE_TOKEN", STORAGE_TOKEN),
            Duration::from_secs(5),
        )
    }
    pub(super) fn new(
        sourcing_url: &str,
        sourcing_token: &str,
        storage_url: &str,
        storage_token: &str,
        timeout: Duration,
    ) -> Result<Self, reqwest::Error> {
        Ok(Self {
            http: Client::builder()
                .timeout(timeout)
                .redirect(reqwest::redirect::Policy::none())
                .retry(reqwest::retry::never())
                .build()?,
            sourcing: endpoint(sourcing_url, sourcing_token),
            storage: endpoint(storage_url, storage_token),
        })
    }
    pub(super) async fn reports(
        &self,
    ) -> (Result<Report, ReportError>, Result<Report, ReportError>) {
        tokio::join!(self.fetch(&self.sourcing), self.fetch(&self.storage))
    }
    async fn fetch(&self, endpoint: &Result<Endpoint, ReportError>) -> Result<Report, ReportError> {
        let endpoint = endpoint.as_ref().map_err(|error| *error)?;
        let response = self
            .http
            .get(endpoint.url.clone())
            .header(AUTHORIZATION, endpoint.authorization.clone())
            .send()
            .await
            .map_err(|_| ReportError::Unavailable)?;
        match response.status().as_u16() {
            401 | 403 => return Err(ReportError::Authentication),
            200 => {}
            _ => return Err(ReportError::Unavailable),
        }
        let report: Report = response.json().await.map_err(|_| ReportError::Malformed)?;
        if report.schema_name.is_empty()
            || report.generated_at.is_empty()
            || report.tables.iter().any(|table| {
                table.table_name.is_empty()
                    || table.table_bytes.checked_add(table.index_bytes) != Some(table.total_bytes)
            })
        {
            return Err(ReportError::Malformed);
        }
        Ok(report)
    }
}
fn endpoint(base: &str, token: &str) -> Result<Endpoint, ReportError> {
    let mut url = Url::parse(base).map_err(|_| ReportError::Configuration)?;
    if !matches!(url.scheme(), "http" | "https")
        || url.host_str().is_none()
        || !url.username().is_empty()
        || url.password().is_some()
        || url.query().is_some()
        || url.fragment().is_some()
        || token.is_empty()
    {
        return Err(ReportError::Configuration);
    }
    url.set_path(&format!("{}{PATH}", url.path().trim_end_matches('/')));
    let mut authorization = HeaderValue::from_str(&format!("Bearer {token}"))
        .map_err(|_| ReportError::Configuration)?;
    authorization.set_sensitive(true);
    Ok(Endpoint { url, authorization })
}

#[cfg(test)]
mod tests;
