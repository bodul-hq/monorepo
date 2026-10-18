//! Expresses the storefront's product facts in the system's vocabulary. The schema.org island and
//! the flight payload's product record both cross, because reconciling them is structuralization's
//! job; what nothing needs stays behind as the commented-out exclusion records.

pub mod ldjson;
pub mod product;

use super::raw;
use money::{Money, MoneyError};
use serde::{Deserialize, Serialize};
use std::num::ParseFloatError;
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Error)]
pub enum NormalizeError {
    #[error("no `Product` ld+json island on the document")]
    MissingProductIsland,

    #[error("`{field}` `{value}` is not a number: {source}")]
    PriceNotANumber {
        field: String,
        value: String,
        source: ParseFloatError,
    },

    #[error("`{field}` is not money: {source:?}")]
    NotMoney { field: String, source: MoneyError },

    #[error("publishedAt `{value}` is not an RFC 3339 instant: {source}")]
    PublishedAtNotAnInstant {
        value: String,
        source: chrono::ParseError,
    },

    #[error("discount `{code}` `{field}` `{value}` is not an RFC 3339 instant: {source}")]
    DiscountInstantNotAnInstant {
        code: String,
        field: String,
        value: String,
        source: chrono::ParseError,
    },

    #[error("discount `{code}` has an unknown `value_type` `{value_type}`")]
    UnknownDiscountValueType { code: String, value_type: String },
}

fn money(amount: f64, currency: &str, field: &str) -> Result<Money, NormalizeError> {
    Money::from_raw(amount, currency).map_err(|source| NormalizeError::NotMoney {
        field: field.to_string(),
        source,
    })
}

fn decimal_money(amount: &str, currency: &str, field: &str) -> Result<Money, NormalizeError> {
    money(decimal(amount, field)?, currency, field)
}

fn decimal(raw: &str, field: &str) -> Result<f64, NormalizeError> {
    raw.parse::<f64>()
        .map_err(|source| NormalizeError::PriceNotANumber {
            field: field.to_string(),
            value: raw.to_string(),
            source,
        })
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AnkerUkNormalizedDocument {
    pub ld_json: ldjson::LdJson,
    pub product: product::ProductRecord,
}

impl AnkerUkNormalizedDocument {
    pub fn from_raw(
        raw: raw::AnkerUkRawDocument,
    ) -> Result<AnkerUkNormalizedDocument, NormalizeError> {
        Ok(AnkerUkNormalizedDocument {
            ld_json: ldjson::LdJson::from_raw(raw.ld_json)?,
            product: product::ProductRecord::from_raw(raw.flight.product)?,
        })
    }
}
