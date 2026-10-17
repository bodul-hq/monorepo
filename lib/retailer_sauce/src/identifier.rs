//! Identifier types shared by the retailer crates.
//!
//! Each one names a single way a storefront identifies a product or one of its variations, and
//! carries that name in a `value` string. They exist so a signature says which identifier it
//! takes: five `String` fields in a row are five chances to pass the handle where the SKU
//! belongs, and the compiler cannot see the mistake. The wrapper makes each one its own type
//! without changing what is stored, and each is built through `TryFrom<String>` so an empty
//! name fails the document instead of travelling on as a valid-looking identifier.
//!
//! Each serializes as its bare string, not as an object holding `value`.

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Why a string could not be taken as one of the identifiers below.
///
/// One enum for all five: they are constructed together, one per field, by the stage that
/// builds an identifier set, and a caller that wants to tell them apart matches the variant.
#[derive(Debug, Clone, PartialEq, Error)]
pub enum IdentifierError {
    #[error("`Gtin` is empty")]
    EmptyGtin,

    #[error("`Sku` is empty")]
    EmptySku,

    #[error("`ShopifyProductGid` is empty")]
    EmptyShopifyProductGid,

    #[error("`ShopifyProductVariationGid` is empty")]
    EmptyShopifyProductVariationGid,

    #[error("`ShopifyHandle` is empty")]
    EmptyShopifyHandle,

    /// Carries the whole gid: the tail that failed is only readable against the value it came
    /// from.
    #[error("`ShopifyProductVariationGid` `{value}` does not end in a numeric id")]
    VariationGidWithoutNumericId { value: String },
}

/// The barcode, as the Global Trade Item Number the storefront renders.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Gtin {
    pub value: String,
}

impl TryFrom<String> for Gtin {
    type Error = IdentifierError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        if value.is_empty() {
            return Err(IdentifierError::EmptyGtin);
        }

        Ok(Gtin { value })
    }
}

/// The retailer's own stock-keeping unit.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Sku {
    pub value: String,
}

impl TryFrom<String> for Sku {
    type Error = IdentifierError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        if value.is_empty() {
            return Err(IdentifierError::EmptySku);
        }

        Ok(Sku { value })
    }
}

/// Shopify's global id for the product, `gid://shopify/Product/<n>`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ShopifyProductGid {
    pub value: String,
}

impl TryFrom<String> for ShopifyProductGid {
    type Error = IdentifierError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        if value.is_empty() {
            return Err(IdentifierError::EmptyShopifyProductGid);
        }

        Ok(ShopifyProductGid { value })
    }
}

/// Shopify's global id for one variation of it, `gid://shopify/ProductVariant/<n>`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ShopifyProductVariationGid {
    pub value: String,
}

impl TryFrom<String> for ShopifyProductVariationGid {
    type Error = IdentifierError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        if value.is_empty() {
            return Err(IdentifierError::EmptyShopifyProductVariationGid);
        }

        Ok(ShopifyProductVariationGid { value })
    }
}

impl ShopifyProductVariationGid {
    /// The numeric id at the tail - `gid://shopify/ProductVariant/56758478831919` yields
    /// `56758478831919` - which is what a storefront URL passes as `?variant=`.
    ///
    /// Fallible rather than checked in [`TryFrom<String>`], which rejects only the empty string:
    /// tightening construction would change a type all fourteen retailers build. All 94 variant
    /// ids of the `ankerau` subset match `gid://shopify/ProductVariant/<digits>`, so one that does
    /// not is a finding - and a URL built from a malformed id deep-links to nothing while looking
    /// perfectly well-formed.
    pub fn numeric_id(&self) -> Result<&str, IdentifierError> {
        self.value
            .rsplit('/')
            .next()
            .filter(|tail| !tail.is_empty() && tail.bytes().all(|byte| byte.is_ascii_digit()))
            .ok_or_else(|| IdentifierError::VariationGidWithoutNumericId {
                value: self.value.clone(),
            })
    }
}

/// The product's URL slug on the storefront.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ShopifyHandle {
    pub value: String,
}

impl TryFrom<String> for ShopifyHandle {
    type Error = IdentifierError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        if value.is_empty() {
            return Err(IdentifierError::EmptyShopifyHandle);
        }

        Ok(ShopifyHandle { value })
    }
}

#[cfg(test)]
mod tests {
    use super::{IdentifierError, ShopifyProductVariationGid};

    #[test]
    fn the_numeric_id_is_the_tail_of_a_well_formed_gid() {
        let gid = ShopifyProductVariationGid::try_from(
            "gid://shopify/ProductVariant/56758478831919".to_string(),
        )
        .expect("a well-formed gid");

        assert_eq!(gid.numeric_id().expect("a numeric tail"), "56758478831919");
    }

    /// A tail that is not a number fails rather than travelling on: it would build a URL that
    /// looks right and resolves to nothing.
    #[test]
    fn a_tail_that_is_not_a_number_fails() {
        for value in [
            "gid://shopify/ProductVariant/not-a-number",
            "gid://shopify/ProductVariant/",
            "56758478831919x",
        ] {
            let gid = ShopifyProductVariationGid::try_from(value.to_string())
                .expect("construction only rejects the empty string");

            let error = gid
                .numeric_id()
                .expect_err("a non-numeric tail has no variant id");

            assert_eq!(
                error,
                IdentifierError::VariationGidWithoutNumericId {
                    value: value.to_string()
                }
            );
        }
    }

    /// A bare number is its own tail - `rsplit` yields the whole string when there is no `/`.
    #[test]
    fn a_bare_number_is_its_own_tail() {
        let gid = ShopifyProductVariationGid::try_from("56758478831919".to_string()).unwrap();

        assert_eq!(gid.numeric_id().unwrap(), "56758478831919");
    }
}
