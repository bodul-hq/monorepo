//! Turns normalized data into the product variations the rest of the system consumes: one
//! entry per variation, carrying its title, identifiers, images, price and URL.

use super::normalized::{UgreenNasEsNormalizedDocument, product};
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
pub struct UgreenNasEsDocument {
    pub variations: Vec<ProductVariation>,
}

impl UgreenNasEsDocument {
    pub fn from_normalized(
        input: UgreenNasEsNormalizedDocument,
    ) -> Result<UgreenNasEsDocument, StructuralizeError> {
        let product = input.product;
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

        Ok(UgreenNasEsDocument { variations })
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
    use super::{StructuralizeError, UgreenNasEsDocument};
    use crate::ugreennases::offer_details::v1::normalized::UgreenNasEsNormalizedDocument;
    use crate::ugreennases::offer_details::v1::raw::UgreenNasEsRawDocument;
    use crate::ugreennases::offer_details::v1::raw::tests::document;
    use html_extractor::serde_json::{self, Value, json};
    use retailer_sauce::identifier::IdentifierError;

    /// The raw fixture, normalized, as the JSON structuralize reads.
    fn normalized_document() -> Value {
        let raw: UgreenNasEsRawDocument =
            serde_json::from_value(document()).expect("the raw fixture deserializes");
        let normalized =
            UgreenNasEsNormalizedDocument::from_raw(raw).expect("the raw fixture normalizes");
        serde_json::to_value(normalized).expect("serializes")
    }

    fn structuralize(input: Value) -> Result<UgreenNasEsDocument, StructuralizeError> {
        let normalized: UgreenNasEsNormalizedDocument =
            serde_json::from_value(input).expect("the normalized fixture deserializes");
        UgreenNasEsDocument::from_normalized(normalized)
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
            "https://ai-es.ugreen.com/cdn/shop/files/DXP2800_GT-16051-nuevo.webp?v=1780642511",
            "https://ai-es.ugreen.com/cdn/shop/files/01-dxp2800gt-almacene-80tb.jpg?v=1780731996",
        ];
        let product_url =
            "https://ai-es.ugreen.com/products/ugreen-nasync-dxp2800-gt-2-bay-nas-storage";

        assert_eq!(
            structuralized(normalized_document()),
            json!({"variations": [
                {
                    "identifiers": {
                        "gtin": "6942752010514",
                        "sku": "16051",
                        "shopify_product_gid": "gid://shopify/Product/10254048985370",
                        "shopify_product_variation_gid": "gid://shopify/ProductVariant/52296690008346",
                        "shopify_handle": "ugreen-nasync-dxp2800-gt-2-bay-nas-storage",
                    },
                    "title": "UGREEN NASync DXP2800 GT",
                    "price": {"amount": 45999, "currency": "EUR"},
                    "url": format!("{product_url}?variant=52296690008346"),
                    "images": images,
                },
                {
                    "identifiers": {
                        "gtin": null,
                        "sku": "16051+45615",
                        "shopify_product_gid": "gid://shopify/Product/10254048985370",
                        "shopify_product_variation_gid": "gid://shopify/ProductVariant/52640999375130",
                        "shopify_handle": "ugreen-nasync-dxp2800-gt-2-bay-nas-storage",
                    },
                    "title": "UGREEN NASync DXP2800 GT",
                    "price": {"amount": 48999, "currency": "EUR"},
                    "url": format!("{product_url}?variant=52640999375130"),
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
            json!("https://ai-es.ugreen.com/products/ugreen-nasync-dxp2800-gt-2-bay-nas-storage")
        );
    }

    #[test]
    fn a_variation_without_a_sku_is_kept() {
        let mut input = normalized_document();
        input["product"]["variants"][1]["sku"] = Value::Null;

        let output = structuralized(input);
        assert_eq!(output["variations"][1]["identifiers"]["sku"], Value::Null);
        assert_eq!(
            output["variations"][1]["identifiers"]["shopify_product_variation_gid"],
            json!("gid://shopify/ProductVariant/52640999375130")
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
