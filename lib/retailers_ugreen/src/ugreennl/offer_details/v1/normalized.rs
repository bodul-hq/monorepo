//! One product assembled from the main section, analytics and JSON-LD.

mod offers;
pub mod product;

use super::raw;
use money::{Currency, Money};
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// The storefront the pages are saved from; the page states its own address only as a path.
const ORIGIN: &str = "https://nl.ugreen.com";

#[derive(Debug, Clone, PartialEq, Error)]
pub enum NormalizeError {
    #[error("{field}: {reason}")]
    InvalidOffer { field: String, reason: String },
    #[error("shop currency `{currency}` is not supported: {reason}")]
    UnsupportedCurrency { currency: String, reason: String },

    #[error("shop currency `{currency}` has no hundredths, the unit Liquid renders amounts in")]
    CurrencyWithoutHundredths { currency: String },

    #[error("`{field}` `{value}` is not a whole number")]
    NotAWholeNumber { field: String, value: String },

    #[error("the {island} is for product `{id}`, the analytics for `{analytics}`")]
    ProductIdMismatch {
        island: String,
        id: String,
        analytics: u64,
    },

    #[error("the {island}'s URL `{url}` is not the product's `/products/{handle}`")]
    ProductUrlMismatch {
        island: String,
        url: String,
        handle: String,
    },

    #[error("the selected variant `{id}` is not among the analytics variants")]
    SelectedVariantNotListed { id: u64 },

    #[error("the variant picker renders no record of the selected variant")]
    PickerWithoutSelectedVariant,

    #[error("the picker's selected variant disagrees with the page on `{field}`")]
    SelectedVariantMismatch { field: String },

    #[error("the displayed price {displayed} is above the variant's price {listed}")]
    DisplayedPriceAboveListed { displayed: i64, listed: i64 },

    #[error(
        "the displayed price {displayed} is discounted from {listed}, but the struck-through \
         price is {compare}"
    )]
    DiscountedFromAnotherPrice {
        displayed: i64,
        listed: i64,
        compare: i64,
    },

    #[error("variant `{variant_id}` selects {selected} option values against {offered} options")]
    OptionCountMismatch {
        variant_id: u64,
        offered: usize,
        selected: usize,
    },
}

// Liquid renders every amount in hundredths of the shop currency, so only a currency whose minor
// unit is a hundredth takes them as they are.
fn shop_currency(code: &str) -> Result<Currency, NormalizeError> {
    let currency: Currency =
        code.parse()
            .map_err(|reason| NormalizeError::UnsupportedCurrency {
                currency: code.to_string(),
                reason,
            })?;

    if currency.exponent() != 2 {
        return Err(NormalizeError::CurrencyWithoutHundredths {
            currency: code.to_string(),
        });
    }

    Ok(currency)
}

fn money_from_shopify_cents(cents: i64, currency: Currency) -> Money {
    Money::new(cents, currency)
}

fn whole_number<T: std::str::FromStr>(value: &str, field: &str) -> Result<T, NormalizeError> {
    value.parse().map_err(|_| NormalizeError::NotAWholeNumber {
        field: field.to_string(),
        value: value.to_string(),
    })
}

// Liquid gives bare numeric ids; the family identifies Shopify objects by their Admin API gid.
fn variant_gid(id: u64) -> String {
    format!("gid://shopify/ProductVariant/{id}")
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UgreenNlNormalizedDocument {
    pub product: product::Product,
}

impl UgreenNlNormalizedDocument {
    pub fn from_raw(
        raw: raw::UgreenNlRawDocument,
    ) -> Result<UgreenNlNormalizedDocument, NormalizeError> {
        Ok(UgreenNlNormalizedDocument {
            product: product::Product::from_raw(raw)?,
        })
    }
}
