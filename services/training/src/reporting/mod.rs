//! Read-only reports over the Training database, matching retailer sourcing's contracts.
mod api;
mod store;

pub use api::{DEFAULT_REPORTING_TOKEN, ReportingApi};
