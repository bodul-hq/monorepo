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

/// Shopify identifiers are optional and never synthesized from URLs or other IDs.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "ExternalWire")]
pub struct ExternalIdentifiers {
    pub shopify_product_gid: Option<ShopifyProductGid>,
    pub shopify_product_variation_gid: Option<ShopifyProductVariationGid>,
    pub shopify_handle: Option<ShopifyHandle>,
}
#[derive(Deserialize)]
struct ExternalWire {
    shopify_product_gid: Option<ShopifyProductGid>,
    shopify_product_variation_gid: Option<ShopifyProductVariationGid>,
    shopify_handle: Option<ShopifyHandle>,
}
impl TryFrom<ExternalWire> for ExternalIdentifiers {
    type Error = String;
    fn try_from(value: ExternalWire) -> Result<Self, Self::Error> {
        let result = Self {
            shopify_product_gid: value.shopify_product_gid,
            shopify_product_variation_gid: value.shopify_product_variation_gid,
            shopify_handle: value.shopify_handle,
        };
        result.guard()?;
        Ok(result)
    }
}
impl ExternalIdentifiers {
    pub fn guard(&self) -> Result<(), String> {
        if let Some(value) = &self.shopify_product_gid {
            ShopifyProductGid::try_from(value.value.clone()).map_err(|error| error.to_string())?;
            guard_nul(&value.value)?;
        }
        if let Some(value) = &self.shopify_product_variation_gid {
            ShopifyProductVariationGid::try_from(value.value.clone())
                .map_err(|error| error.to_string())?;
            guard_nul(&value.value)?;
        }
        if let Some(value) = &self.shopify_handle {
            ShopifyHandle::try_from(value.value.clone()).map_err(|error| error.to_string())?;
            guard_nul(&value.value)?;
        }
        Ok(())
    }
}

/// Complete decided state. Imports enrich only fields for which they carry a value.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "IdentifiersWire")]
pub struct ProductIdentifiers {
    pub gtin: Option<Gtin>,
    pub sku: Option<Sku>,
    pub external_identifiers: ExternalIdentifiers,
}
#[derive(Deserialize)]
struct IdentifiersWire {
    gtin: Option<Gtin>,
    sku: Option<Sku>,
    #[serde(default)]
    external_identifiers: ExternalIdentifiers,
}
impl TryFrom<IdentifiersWire> for ProductIdentifiers {
    type Error = String;
    fn try_from(value: IdentifiersWire) -> Result<Self, Self::Error> {
        let result = Self {
            gtin: value.gtin,
            sku: value.sku,
            external_identifiers: value.external_identifiers,
        };
        result.guard()?;
        Ok(result)
    }
}
impl ProductIdentifiers {
    pub fn guard(&self) -> Result<(), String> {
        if let Some(value) = &self.gtin {
            Gtin::try_from(value.value.clone()).map_err(|error| error.to_string())?;
            guard_nul(&value.value)?;
        }
        if let Some(value) = &self.sku {
            Sku::try_from(value.value.clone()).map_err(|error| error.to_string())?;
            guard_nul(&value.value)?;
        }
        self.external_identifiers.guard()
    }
    pub fn enriched(&self, incoming: &Self) -> Self {
        Self {
            gtin: incoming.gtin.clone().or_else(|| self.gtin.clone()),
            sku: incoming.sku.clone().or_else(|| self.sku.clone()),
            external_identifiers: ExternalIdentifiers {
                shopify_product_gid: incoming
                    .external_identifiers
                    .shopify_product_gid
                    .clone()
                    .or_else(|| self.external_identifiers.shopify_product_gid.clone()),
                shopify_product_variation_gid: incoming
                    .external_identifiers
                    .shopify_product_variation_gid
                    .clone()
                    .or_else(|| {
                        self.external_identifiers
                            .shopify_product_variation_gid
                            .clone()
                    }),
                shopify_handle: incoming
                    .external_identifiers
                    .shopify_handle
                    .clone()
                    .or_else(|| self.external_identifiers.shopify_handle.clone()),
            },
        }
    }
}
fn guard_nul(value: &str) -> Result<(), String> {
    if value.contains('\0') {
        Err("identifiers must not contain NUL".into())
    } else {
        Ok(())
    }
}

pub fn external_object<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<ExternalIdentifiers>, D::Error> {
    ExternalIdentifiers::deserialize(deserializer).map(Some)
}

/// Change events replace the complete set, so omitted fields are not legacy defaults.
pub fn complete_identifiers<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<ProductIdentifiers, D::Error> {
    #[derive(Deserialize)]
    struct Complete {
        #[serde(deserialize_with = "Option::deserialize")]
        gtin: Option<Gtin>,
        #[serde(deserialize_with = "Option::deserialize")]
        sku: Option<Sku>,
        external_identifiers: ExternalIdentifiers,
    }
    let value = Complete::deserialize(deserializer)?;
    let identifiers = ProductIdentifiers {
        gtin: value.gtin,
        sku: value.sku,
        external_identifiers: value.external_identifiers,
    };
    identifiers.guard().map_err(serde::de::Error::custom)?;
    Ok(identifiers)
}

#[cfg(test)]
mod propagation_tests {
    use super::*;
    #[test]
    fn exact_strings_enrich_without_clearing_and_bad_present_values_fail() {
        let initial: ProductIdentifiers = serde_json::from_value(serde_json::json!({"gtin":"0012345678905","sku":"A1-BLACK","external_identifiers":{"shopify_product_gid":"gid://shopify/Product/1","shopify_product_variation_gid":"gid://shopify/ProductVariant/2","shopify_handle":"usb-charger"}})).unwrap();
        assert_eq!(initial.enriched(&Default::default()), initial);
        assert_eq!(
            serde_json::to_value(&initial).unwrap()["gtin"],
            "0012345678905"
        );
        let partial: ProductIdentifiers =
            serde_json::from_value(serde_json::json!({"sku":"A1-WHITE"})).unwrap();
        let merged = initial.enriched(&partial);
        assert_eq!(merged.sku.unwrap().value, "A1-WHITE");
        assert_eq!(merged.gtin, initial.gtin);
        assert_eq!(merged.external_identifiers, initial.external_identifiers);
        for value in [
            serde_json::json!({"gtin":""}),
            serde_json::json!({"sku":"\0"}),
            serde_json::json!({"gtin":123}),
            serde_json::json!({"external_identifiers":null}),
            serde_json::json!({"external_identifiers":{"shopify_handle":""}}),
        ] {
            assert!(serde_json::from_value::<ProductIdentifiers>(value).is_err());
        }
        // Construction bypassing transparent deserialization is still rejected by guards.
        let invalid = ProductIdentifiers {
            gtin: Some(Gtin {
                value: String::new(),
            }),
            ..Default::default()
        };
        assert!(invalid.guard().is_err());
    }
}
