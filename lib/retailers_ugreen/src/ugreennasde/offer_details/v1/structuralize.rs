//! Turns normalized data into the product variations the rest of the system consumes: one
//! entry per variation, carrying its title, identifiers, images, price and URL.

use super::normalized::{UgreenNasDeNormalizedDocument, product};
use money::Money;
use retailer_sauce::identifier::{
    Gtin, IdentifierError, ShopifyHandle, ShopifyProductGid, ShopifyProductVariationGid, Sku,
};
use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Error)]
pub enum StructuralizeError {
    #[error(transparent)]
    Identifier(#[from] IdentifierError),
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct UgreenNasDeDocument {
    /// Empty for the `worry-free-purchase` add-on: purchase protection is not a product.
    pub variations: Vec<ProductVariation>,
}

impl UgreenNasDeDocument {
    pub fn from_normalized(
        input: UgreenNasDeNormalizedDocument,
    ) -> Result<UgreenNasDeDocument, StructuralizeError> {
        let product = input.product;
        if product.purchase_protection {
            return Ok(UgreenNasDeDocument {
                variations: Vec::new(),
            });
        }

        let variations_count = product.variants.len();
        let variations = product
            .variants
            .iter()
            .map(|variant| {
                let identifiers = ProductVariationIdentifier::from_normalized(&product, variant)?;
                let url = variation_url(
                    &product.url,
                    &identifiers.shopify_product_variation_gid,
                    variations_count,
                )?;

                Ok(ProductVariation {
                    title: product.title.clone(),
                    identifiers,
                    images: product.images.clone(),
                    price: variant.price.clone(),
                    url,
                })
            })
            .collect::<Result<_, StructuralizeError>>()?;

        Ok(UgreenNasDeDocument { variations })
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ProductVariation {
    pub identifiers: ProductVariationIdentifier,
    /// The product's title, shared by every variation.
    pub title: String,
    /// The variation's own price, before any discount code. `None` where Shopify renders 0,
    /// which no saved variant does.
    pub price: Option<Money>,
    pub url: String,
    /// The product's gallery images.
    pub images: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ProductVariationIdentifier {
    /// The picker's all-digit barcode. `None` where it has none, and on every page without a
    /// picker.
    pub gtin: Option<Gtin>,
    /// Never `None` on the saved pages; kept optional for an empty SKU.
    pub sku: Option<Sku>,
    pub shopify_product_gid: ShopifyProductGid,
    pub shopify_product_variation_gid: ShopifyProductVariationGid,
    pub shopify_handle: ShopifyHandle,
}

impl ProductVariationIdentifier {
    fn from_normalized(
        product: &product::Product,
        variant: &product::Variant,
    ) -> Result<ProductVariationIdentifier, StructuralizeError> {
        Ok(ProductVariationIdentifier {
            gtin: variant.barcode.clone().map(Gtin::try_from).transpose()?,
            sku: variant.sku.clone().map(Sku::try_from).transpose()?,
            shopify_product_gid: product.id.clone().try_into()?,
            shopify_product_variation_gid: variant.id.clone().try_into()?,
            shopify_handle: product.handle.clone().try_into()?,
        })
    }
}

fn variation_url(
    product_url: &str,
    gid: &ShopifyProductVariationGid,
    variations: usize,
) -> Result<String, StructuralizeError> {
    // The product URL is the handle on this store's origin, not the canonical link, which on
    // some sibling stores' pages points at `nas.ugreen.com`.
    if variations < 2 {
        return Ok(product_url.to_owned());
    }

    Ok(format!("{}?variant={}", product_url, gid.numeric_id()?))
}

#[cfg(test)]
mod tests {
    use super::{StructuralizeError, UgreenNasDeDocument};
    use crate::ugreennasde::offer_details::v1::normalized::UgreenNasDeNormalizedDocument;
    use crate::ugreennasde::offer_details::v1::raw::UgreenNasDeRawDocument;
    use crate::ugreennasde::offer_details::v1::raw::tests::document;
    use html_extractor::serde_json::{self, Value, json};
    use retailer_sauce::identifier::IdentifierError;

    /// The raw fixture, normalized, as the JSON structuralize reads.
    fn normalized_document() -> Value {
        let raw: UgreenNasDeRawDocument =
            serde_json::from_value(document()).expect("the raw fixture deserializes");
        let normalized =
            UgreenNasDeNormalizedDocument::from_raw(raw).expect("the raw fixture normalizes");
        serde_json::to_value(normalized).expect("serializes")
    }

    fn structuralize(input: Value) -> Result<UgreenNasDeDocument, StructuralizeError> {
        let normalized: UgreenNasDeNormalizedDocument =
            serde_json::from_value(input).expect("the normalized fixture deserializes");
        UgreenNasDeDocument::from_normalized(normalized)
    }

    fn structuralized(input: Value) -> Value {
        serde_json::to_value(structuralize(input).expect("the fixture structuralizes"))
            .expect("serializes")
    }

    fn single_variant(mut input: Value) -> Value {
        input["product"]["variants"]
            .as_array_mut()
            .expect("variants")
            .truncate(1);
        input
    }

    #[test]
    fn each_variation_carries_its_title_identifiers_images_price_and_url_only() {
        let images = [
            "https://ai-de.ugreen.com/cdn/shop/files/DXP4800_GT_16058.webp?v=1780565809",
            "https://ai-de.ugreen.com/cdn/shop/files/07_813b72a9-9cac-4d55-b7c0-8c1be5a945ad.jpg?v=1785465917",
        ];
        let product_url =
            "https://ai-de.ugreen.com/products/ugreen-nasync-dxp4800-gt-4-bay-nas-storage";

        assert_eq!(
            structuralized(normalized_document()),
            json!({"variations": [
                {
                    "identifiers": {
                        "gtin": "6942752010583",
                        "sku": "16058",
                        "shopify_product_gid": "gid://shopify/Product/9553994776807",
                        "shopify_product_variation_gid": "gid://shopify/ProductVariant/52118906241255",
                        "shopify_handle": "ugreen-nasync-dxp4800-gt-4-bay-nas-storage",
                    },
                    "title": "UGREEN NASync DXP4800 GT 4-Bay 144TB NAS",
                    "price": {"amount": 58999, "currency": "EUR"},
                    "url": format!("{product_url}?variant=52118906241255"),
                    "images": images,
                },
                {
                    "identifiers": {
                        "gtin": "17841947458093",
                        "sku": "16058 + 45615",
                        "shopify_product_gid": "gid://shopify/Product/9553994776807",
                        "shopify_product_variation_gid": "gid://shopify/ProductVariant/52244153860327",
                        "shopify_handle": "ugreen-nasync-dxp4800-gt-4-bay-nas-storage",
                    },
                    "title": "UGREEN NASync DXP4800 GT 4-Bay 144TB NAS",
                    "price": {"amount": 61598, "currency": "EUR"},
                    "url": format!("{product_url}?variant=52244153860327"),
                    "images": images,
                },
            ]})
        );
    }

    #[test]
    fn a_single_variation_takes_the_product_url() {
        let output = structuralized(single_variant(normalized_document()));

        assert_eq!(
            output["variations"][0]["url"],
            json!("https://ai-de.ugreen.com/products/ugreen-nasync-dxp4800-gt-4-bay-nas-storage")
        );
    }

    #[test]
    fn the_purchase_protection_add_on_has_no_variations() {
        let mut input = normalized_document();
        input["product"]["purchase_protection"] = json!(true);

        assert_eq!(structuralized(input), json!({"variations": []}));
    }

    #[test]
    fn a_variation_without_a_sku_is_kept() {
        let mut input = normalized_document();
        input["product"]["variants"][1]["sku"] = Value::Null;

        let output = structuralized(input);
        assert_eq!(output["variations"][1]["identifiers"]["sku"], Value::Null);
        assert_eq!(
            output["variations"][1]["identifiers"]["shopify_product_variation_gid"],
            json!("gid://shopify/ProductVariant/52244153860327")
        );
    }

    #[test]
    fn an_unpriced_variation_is_kept_without_a_price() {
        let mut input = single_variant(normalized_document());
        input["product"]["variants"][0]["price"] = Value::Null;

        let output = structuralized(input);
        assert_eq!(output["variations"][0]["price"], Value::Null);
    }

    #[test]
    fn an_unusable_identifier_crosses_as_the_identifier_failure() {
        let mut handle = normalized_document();
        handle["product"]["handle"] = json!("");
        assert_eq!(
            structuralize(handle),
            Err(StructuralizeError::Identifier(
                IdentifierError::EmptyShopifyHandle
            ))
        );

        let mut sku = normalized_document();
        sku["product"]["variants"][0]["sku"] = json!("");
        assert_eq!(
            structuralize(sku),
            Err(StructuralizeError::Identifier(IdentifierError::EmptySku))
        );

        let mut gtin = normalized_document();
        gtin["product"]["variants"][0]["barcode"] = json!("");
        assert_eq!(
            structuralize(gtin),
            Err(StructuralizeError::Identifier(IdentifierError::EmptyGtin))
        );
    }

    #[test]
    fn a_variant_gid_without_a_numeric_id_fails_a_multi_variant_document() {
        let mut input = normalized_document();
        input["product"]["variants"][1]["id"] = json!("gid://shopify/ProductVariant/x");

        assert_eq!(
            structuralize(input),
            Err(StructuralizeError::Identifier(
                IdentifierError::VariationGidWithoutNumericId {
                    value: "gid://shopify/ProductVariant/x".to_string()
                }
            ))
        );
    }
}
