//! Offer details.

pub mod prelude;

pub(super) mod command;
pub(super) mod event;
pub(super) mod handler;

use serde::{Deserialize, Serialize};

/// The source and input needed to fetch offer details.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SourceType {
    Html { url: String },
    GraphQl { product_id: String },
}

pub(super) mod fetch;

pub(super) mod reacts_to;
