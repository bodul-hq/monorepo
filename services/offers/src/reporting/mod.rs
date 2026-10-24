//! Read-only reports over the Offers database, matching retailer sourcing's contracts.
mod api;
mod store;

pub use api::{DEFAULT_REPORTING_TOKEN, ReportingApi};
