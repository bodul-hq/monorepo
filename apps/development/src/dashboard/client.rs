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
#[derive(Debug, Deserialize)]
pub(super) struct EntryStatuses {
    pub generated_at: String,
    #[serde(default, deserialize_with = "present_window_bound")]
    pub window_start: Option<String>,
    #[serde(default, deserialize_with = "present_window_bound")]
    pub window_end: Option<String>,
    #[serde(default, deserialize_with = "present_entries")]
    pub inbox_entries: Option<Vec<StatusCount>>,
    #[serde(default, deserialize_with = "present_entries")]
    pub outbox_entries: Option<Vec<StatusCount>>,
    pub command_entries: Vec<StatusCount>,
    pub event_entries: Vec<StatusCount>,
}
fn present_window_bound<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<String>, D::Error> {
    String::deserialize(deserializer).map(Some)
}

impl EntryStatuses {
    pub(super) fn window_date(&self) -> Result<Option<chrono::NaiveDate>, ReportError> {
        let (start, end) = match (&self.window_start, &self.window_end) {
            (None, None) => return Ok(None),
            (Some(start), Some(end)) => (start, end),
            _ => return Err(ReportError::Malformed),
        };
        let parse = |value: &str| {
            chrono::DateTime::parse_from_rfc3339(value)
                .map(|time| time.with_timezone(&chrono::Utc))
                .map_err(|_| ReportError::Malformed)
        };
        let start = parse(start)?;
        let end = parse(end)?;
        if start.time() != chrono::NaiveTime::MIN || end - start != chrono::Duration::days(1) {
            return Err(ReportError::Malformed);
        }
        Ok(Some(start.date_naive()))
    }
}
// Missing fields identify older servers; present null or non-array fields are invalid.
fn present_entries<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<Vec<StatusCount>>, D::Error> {
    Vec::deserialize(deserializer).map(Some)
}

#[derive(Debug, Deserialize)]
pub(super) struct StatusCount {
    pub status: i32,
    pub status_text: String,
    pub count: u64,
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
    entries: Result<Endpoint, ReportError>,
    offers: Result<Endpoint, ReportError>,
    offers_entries: Result<Endpoint, ReportError>,
    customer_facing: Result<Endpoint, ReportError>,
    customer_facing_entries: Result<Endpoint, ReportError>,
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
        .map(|clients| {
            clients
                .with_offers(
                    &env("OFFERS_URL", "http://172.28.100.41:3000"),
                    &env("OFFERS_TOKEN", "offers-development-token"),
                )
                .with_customer_facing(
                    &env("CUSTOMER_FACING_URL", "http://172.28.100.42:3000"),
                    &env("CUSTOMER_FACING_TOKEN", "customer-facing-development-token"),
                )
        })
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
            customer_facing: Err(ReportError::Configuration),
            customer_facing_entries: Err(ReportError::Configuration),
            offers: Err(ReportError::Configuration),
            offers_entries: Err(ReportError::Configuration),
            sourcing: endpoint(sourcing_url, sourcing_token),
            storage: endpoint(storage_url, storage_token),
            entries: status_endpoint(sourcing_url, sourcing_token),
        })
    }
    pub(super) fn with_offers(mut self, url: &str, token: &str) -> Self {
        self.offers = endpoint(url, token);
        self.offers_entries = status_endpoint(url, token);
        self
    }
    pub(super) fn with_customer_facing(mut self, url: &str, token: &str) -> Self {
        self.customer_facing = endpoint(url, token);
        self.customer_facing_entries = status_endpoint(url, token);
        self
    }
    pub(super) async fn customer_facing_report(&self) -> Result<Report, ReportError> {
        self.fetch(&self.customer_facing).await
    }
    pub(super) async fn customer_facing_entry_statuses(
        &self,
    ) -> Result<EntryStatuses, ReportError> {
        self.fetch_entry_statuses(&self.customer_facing_entries)
            .await
    }
    pub(super) async fn offers_report(&self) -> Result<Report, ReportError> {
        self.fetch(&self.offers).await
    }
    pub(super) async fn offers_entry_statuses(&self) -> Result<EntryStatuses, ReportError> {
        self.fetch_entry_statuses(&self.offers_entries).await
    }
    pub(super) async fn reports(
        &self,
    ) -> (Result<Report, ReportError>, Result<Report, ReportError>) {
        tokio::join!(self.fetch(&self.sourcing), self.fetch(&self.storage))
    }
    pub(super) async fn entry_statuses(&self) -> Result<EntryStatuses, ReportError> {
        self.fetch_entry_statuses(&self.entries).await
    }
    async fn fetch_entry_statuses(
        &self,
        endpoint: &Result<Endpoint, ReportError>,
    ) -> Result<EntryStatuses, ReportError> {
        let report: EntryStatuses = self.fetch_json(endpoint).await?;
        report.window_date()?;
        if report.generated_at.is_empty()
            || report
                .command_entries
                .iter()
                .chain(&report.event_entries)
                .chain(report.inbox_entries.iter().flatten())
                .chain(report.outbox_entries.iter().flatten())
                .any(|entry| entry.status_text.is_empty())
        {
            return Err(ReportError::Malformed);
        }
        Ok(report)
    }
    async fn fetch_json<T: serde::de::DeserializeOwned>(
        &self,
        endpoint: &Result<Endpoint, ReportError>,
    ) -> Result<T, ReportError> {
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
        response.json().await.map_err(|_| ReportError::Malformed)
    }
    async fn fetch(&self, endpoint: &Result<Endpoint, ReportError>) -> Result<Report, ReportError> {
        let report: Report = self.fetch_json(endpoint).await?;
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
fn status_endpoint(base: &str, token: &str) -> Result<Endpoint, ReportError> {
    endpoint(base, token).map(|mut endpoint| {
        let path = endpoint.url.path().trim_end_matches(PATH).to_owned();
        endpoint
            .url
            .set_path(&format!("{path}/api/reporting/entry-statuses"));
        endpoint
    })
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
