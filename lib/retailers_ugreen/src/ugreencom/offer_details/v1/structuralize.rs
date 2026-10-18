//! Turns normalized data into the product variations the rest of the system consumes: one
//! entry per variation, carrying its title, identifiers, images, price and URL.

use super::normalized::{UgreenComNormalizedDocument, product};
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
pub struct UgreenComDocument {
    pub variations: Vec<ProductVariation>,
}

impl UgreenComDocument {
    pub fn from_normalized(
        input: UgreenComNormalizedDocument,
    ) -> Result<UgreenComDocument, StructuralizeError> {
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

        Ok(UgreenComDocument { variations })
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ProductVariation {
    pub identifiers: ProductVariationIdentifier,
    /// The product's title, shared by every variation.
    pub title: String,
    /// The analytics price, a placeholder on this brand site (USD 1,300.00 on 65 pages, USD 5.00
    /// on one). Kept as the price by the user's decision; see the normalized variant's `price`.
    pub price: Option<Money>,
    pub url: String,
    /// The product's gallery.
    pub images: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ProductVariationIdentifier {
    /// `None` on every variation: the brand site renders no barcode.
    pub gtin: Option<Gtin>,
    /// Present on all 66 saved variants.
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
            gtin: None,
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
    // Every saved product has a single variant, so the canonical product URL is the variation's.
    // Not the "Shop Now" link: that is another store's page for the product.
    if variations < 2 {
        return Ok(product_url.to_owned());
    }

    Ok(format!("{}?variant={}", product_url, gid.numeric_id()?))
}

#[cfg(test)]
mod tests {
    use super::{StructuralizeError, UgreenComDocument};
    use crate::ugreencom::offer_details::v1::normalized::UgreenComNormalizedDocument;
    use crate::ugreencom::offer_details::v1::raw::UgreenComRawDocument;
    use crate::ugreencom::offer_details::v1::raw::tests::document;
    use html_extractor::serde_json::{self, Value, json};
    use retailer_sauce::identifier::IdentifierError;

    /// The raw fixture, normalized, as the JSON structuralize reads.
    fn normalized_document() -> Value {
        let raw: UgreenComRawDocument =
            serde_json::from_value(document()).expect("the raw fixture deserializes");
        let normalized =
            UgreenComNormalizedDocument::from_raw(raw).expect("the raw fixture normalizes");
        serde_json::to_value(normalized).expect("serializes")
    }

    fn structuralize(input: Value) -> Result<UgreenComDocument, StructuralizeError> {
        let normalized: UgreenComNormalizedDocument =
            serde_json::from_value(input).expect("the normalized fixture deserializes");
        UgreenComDocument::from_normalized(normalized)
    }

    fn structuralized(input: Value) -> Value {
        serde_json::to_value(structuralize(input).expect("the fixture structuralizes"))
            .expect("serializes")
    }

    /// The fixture with a second variant, which no saved page has.
    fn two_variants(mut input: Value) -> Value {
        let mut second = input["product"]["variants"][0].clone();
        second["id"] = json!("gid://shopify/ProductVariant/47897996820700");
        second["sku"] = json!("usa-15496");
        input["product"]["variants"]
            .as_array_mut()
            .expect("variants")
            .push(second);
        input
    }

    #[test]
    fn the_variation_carries_its_title_identifiers_images_price_and_url_only() {
        assert_eq!(
            structuralized(normalized_document()),
            json!({"variations": [{
                "identifiers": {
                    "gtin": null,
                    "sku": "usa-15495",
                    "shopify_product_gid": "gid://shopify/Product/9027458236635",
                    "shopify_product_variation_gid": "gid://shopify/ProductVariant/47897996820699",
                    "shopify_handle": "usa-15495",
                },
                "title": "UGREEN Revodok 105 5-in-1 USB-C Hub",
                "price": {"amount": 130000, "currency": "USD"},
                "url": "https://www.ugreen.com/products/usa-15495",
                "images": [
                    "https://www.ugreen.com/cdn/shop/files/d0a9824bf408d4d39de70224bbafb22d.png?v=1762322082"
                ],
            }]})
        );
    }

    #[test]
    fn a_single_variation_takes_the_canonical_product_url() {
        let output = structuralized(normalized_document());

        assert_eq!(
            output["variations"][0]["url"],
            json!("https://www.ugreen.com/products/usa-15495")
        );
    }

    #[test]
    fn several_variations_take_the_variant_parameter() {
        let output = structuralized(two_variants(normalized_document()));

        assert_eq!(
            output["variations"][1]["url"],
            json!("https://www.ugreen.com/products/usa-15495?variant=47897996820700")
        );
    }

    #[test]
    fn the_sales_channels_stay_in_the_normalized_document() {
        let output = structuralized(normalized_document());

        assert!(!output.to_string().contains("us.ugreen.com"));
    }

    #[test]
    fn a_variation_without_a_sku_is_kept() {
        let mut input = normalized_document();
        input["product"]["variants"][0]["sku"] = Value::Null;

        let output = structuralized(input);
        assert_eq!(output["variations"][0]["identifiers"]["sku"], Value::Null);
        assert_eq!(
            output["variations"][0]["identifiers"]["shopify_product_variation_gid"],
            json!("gid://shopify/ProductVariant/47897996820699")
        );
    }

    #[test]
    fn an_unpriced_variation_is_kept_without_a_price() {
        let mut input = normalized_document();
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
    }

    #[test]
    fn a_variant_gid_without_a_numeric_id_fails_a_multi_variant_document() {
        let mut input = two_variants(normalized_document());
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
