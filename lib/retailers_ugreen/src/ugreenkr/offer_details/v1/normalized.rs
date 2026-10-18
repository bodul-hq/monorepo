//! Korean product facts; Shopify hundredths are converted exactly to whole won.
use serde::{Deserialize, Serialize};
use thiserror::Error;
mod offers;
pub mod product;
#[derive(Debug, Clone, PartialEq, Error)]
pub enum NormalizeError {
    #[error("{field}: {reason}")]
    InvalidOffer { field: String, reason: String },
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UgreenKrNormalizedDocument {
    pub product: product::Product,
}
impl UgreenKrNormalizedDocument {
    pub fn from_raw(raw: super::raw::UgreenKrRawDocument) -> Result<Self, NormalizeError> {
        Ok(Self {
            product: product::Product::from_raw(raw)?,
        })
    }
}
