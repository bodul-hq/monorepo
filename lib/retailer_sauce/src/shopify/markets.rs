//! Neutral context carried in durable commands and events.
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MarketContext {
    pub country: String,
    pub language: String,
    pub currency: String,
}
