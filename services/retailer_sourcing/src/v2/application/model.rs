//! Request and response of the start-initial-fills use case.

use shared::retailer::RetailerCode;
use uuid::Uuid;

/// Retailers whose initial offer fills should be started.
pub(crate) struct InitialFillOrder {
    pub(crate) retailers: Vec<String>,
}

/// Outcome of starting initial offer fills.
pub(crate) enum InitialFillStatus {
    /// All runs were queued, in first-occurrence request order.
    Queued(Vec<FillRun>),
    /// The retailer list was empty, or held an unknown or unsupported retailer.
    InvalidRequest(PreparationFailure),
    /// Dispatching failed part-way; the failure lists confirmed runs.
    QueueFailed(StartFailure),
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum PreparationFailure {
    #[error("retailers must not be empty")]
    EmptyRetailers,
    #[error("unknown retailer code: {0}")]
    UnknownRetailer(String),
    #[error("duplicate retailer code: {0}")]
    DuplicateRetailer(String),
    #[error("initial fill is not implemented for retailer {0}")]
    UnsupportedRetailer(String),
    #[error("{message}: {retailer}")]
    InvalidInventory { retailer: String, message: String },
}

impl PreparationFailure {
    pub(crate) fn invalid_inventory(retailer_code: RetailerCode, message: String) -> Self {
        Self::InvalidInventory {
            retailer: retailer_code.into(),
            message,
        }
    }
}

#[derive(Debug)]
pub(crate) struct FillRun {
    pub(crate) retailer: String,
    pub(crate) run_id: Uuid,
}

impl FillRun {
    pub(crate) fn new(retailer: String, run_id: Uuid) -> Self {
        Self { retailer, run_id }
    }
}

/// A failed queue. `fills` holds the runs confirmed queued before the failure
/// and is empty when none were, or when the outcomes are unknown.
pub(crate) struct StartFailure {
    /// Opaque ID; internal details are logged under it.
    pub(crate) error_id: Uuid,
    pub(crate) fills: Vec<FillRun>,
}

impl StartFailure {
    pub(crate) fn new(fills: Vec<FillRun>) -> Self {
        Self {
            error_id: Uuid::now_v7(),
            fills,
        }
    }
}
