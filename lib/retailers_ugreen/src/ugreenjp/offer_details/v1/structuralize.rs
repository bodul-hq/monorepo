//! Turns normalized data into the product variations the rest of the system consumes: one
//! entry per variation, carrying its title, identifiers, images, price and URL.

use super::normalized::{UgreenJpNormalizedDocument, product};
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
pub struct UgreenJpDocument {
    pub variations: Vec<ProductVariation>,
}

impl UgreenJpDocument {
    pub fn from_normalized(
        input: UgreenJpNormalizedDocument,
    ) -> Result<UgreenJpDocument, StructuralizeError> {
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

        Ok(UgreenJpDocument { variations })
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ProductVariation {
    pub identifiers: ProductVariationIdentifier,
    /// The product's title, shared by every variation.
    pub title: String,
    /// The variation's own price in whole yen, before any discount code. `None` where the
    /// storefront renders ¥0; no saved variant does.
    pub price: Option<Money>,
    pub url: String,
    /// The product's gallery images.
    pub images: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ProductVariationIdentifier {
    /// `None` on all 163 saved variants: the picker's barcode is `null` throughout and a
    /// `Default Title` product renders none.
    pub gtin: Option<Gtin>,
    /// `None` on 19 of the 163 saved variants, on 11 products; the Shopify gids still identify
    /// them.
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
            gtin: variant.gtin.clone().map(Gtin::try_from).transpose()?,
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
    // The island's product URL, with the handle percent-encoded. The offer URLs are not read: the
    // variation URL is built the same way for every storefront.
    if variations < 2 {
        return Ok(product_url.to_owned());
    }

    Ok(format!("{}?variant={}", product_url, gid.numeric_id()?))
}

#[cfg(test)]
mod tests {
    use super::{StructuralizeError, UgreenJpDocument};
    use crate::ugreenjp::offer_details::v1::normalized::UgreenJpNormalizedDocument;
    use crate::ugreenjp::offer_details::v1::raw::UgreenJpRawDocument;
    use crate::ugreenjp::offer_details::v1::raw::tests::{default_title_document, document};
    use html_extractor::serde_json::{self, Value, json};
    use retailer_sauce::identifier::IdentifierError;

    /// A raw fixture, normalized, as the JSON structuralize reads.
    fn normalized_from(raw: Value) -> Value {
        let raw: UgreenJpRawDocument =
            serde_json::from_value(raw).expect("the raw fixture deserializes");
        let normalized =
            UgreenJpNormalizedDocument::from_raw(raw).expect("the raw fixture normalizes");
        serde_json::to_value(normalized).expect("serializes")
    }

    fn normalized_document() -> Value {
        normalized_from(document())
    }

    fn structuralize(input: Value) -> Result<UgreenJpDocument, StructuralizeError> {
        let normalized: UgreenJpNormalizedDocument =
            serde_json::from_value(input).expect("the normalized fixture deserializes");
        UgreenJpDocument::from_normalized(normalized)
    }

    fn structuralized(input: Value) -> Value {
        serde_json::to_value(structuralize(input).expect("the fixture structuralizes"))
            .expect("serializes")
    }

    #[test]
    fn each_variation_carries_its_title_identifiers_images_price_and_url_only() {
        let images = [
            "https://ugreen.jp/cdn/shop/files/60521-1.png?v=1726277735",
            "https://ugreen.jp/cdn/shop/files/60521-2.png?v=1726277735",
        ];
        let product_url = "https://ugreen.jp/products/lightning-usb-cable";

        assert_eq!(
            structuralized(normalized_document()),
            json!({"variations": [
                {
                    "identifiers": {
                        "gtin": null,
                        "sku": "60521",
                        "shopify_product_gid": "gid://shopify/Product/9456052437272",
                        "shopify_product_variation_gid": "gid://shopify/ProductVariant/49142586999064",
                        "shopify_handle": "lightning-usb-cable",
                    },
                    "title": "UGREENライトニング USB ケーブル",
                    "price": {"amount": 1399, "currency": "JPY"},
                    "url": format!("{product_url}?variant=49142586999064"),
                    "images": images,
                },
                {
                    "identifiers": {
                        "gtin": null,
                        "sku": "70733",
                        "shopify_product_gid": "gid://shopify/Product/9456052437272",
                        "shopify_product_variation_gid": "gid://shopify/ProductVariant/49154830631192",
                        "shopify_handle": "lightning-usb-cable",
                    },
                    "title": "UGREENライトニング USB ケーブル",
                    "price": {"amount": 1599, "currency": "JPY"},
                    "url": format!("{product_url}?variant=49154830631192"),
                    "images": images,
                },
            ]})
        );
    }

    /// As on `products-green-nasync-dh4300-plus-nas-storage.html`: one variant, no SKU.
    #[test]
    fn a_single_variation_takes_the_product_url() {
        let output = structuralized(normalized_from(default_title_document()));

        assert_eq!(
            output,
            json!({"variations": [{
                "identifiers": {
                    "gtin": null,
                    "sku": null,
                    "shopify_product_gid": "gid://shopify/Product/10414739226904",
                    "shopify_product_variation_gid": "gid://shopify/ProductVariant/52659056148760",
                    "shopify_handle": "green-nasync-dh4300-plus-nas-storage",
                },
                "title": "UGREEN NASync DH4300 Plus",
                "price": {"amount": 52700, "currency": "JPY"},
                "url": "https://ugreen.jp/products/green-nasync-dh4300-plus-nas-storage",
                "images": ["https://ugreen.jp/cdn/shop/files/DH4300.png?v=1789197630"],
            }]})
        );
    }

    #[test]
    fn a_gtin_crosses_as_the_identifier() {
        let mut input = normalized_document();
        input["product"]["variants"][0]["gtin"] = json!("6957303860521");

        let output = structuralized(input);
        assert_eq!(
            output["variations"][0]["identifiers"]["gtin"],
            json!("6957303860521")
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
            json!("gid://shopify/ProductVariant/49154830631192")
        );
    }

    #[test]
    fn an_unpriced_variation_is_kept_without_a_price() {
        let mut input = normalized_document();
        input["product"]["variants"][0]["price"] = Value::Null;

        let output = structuralized(input);
        assert_eq!(output["variations"][0]["price"], Value::Null);
        assert_eq!(output["variations"][1]["price"]["amount"], json!(1599));
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
        gtin["product"]["variants"][0]["gtin"] = json!("");
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
