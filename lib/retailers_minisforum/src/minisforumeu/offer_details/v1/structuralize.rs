//! Turns normalized data into the product variations the rest of the system consumes: one
//! entry per variation, carrying its title, identifiers, images, price and URL.

use super::normalized::{MinisforumEuNormalizedDocument, product};
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
pub struct MinisforumEuDocument {
    pub variations: Vec<ProductVariation>,
}

impl MinisforumEuDocument {
    pub fn from_normalized(
        input: MinisforumEuNormalizedDocument,
    ) -> Result<MinisforumEuDocument, StructuralizeError> {
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

        Ok(MinisforumEuDocument { variations })
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ProductVariation {
    pub identifiers: ProductVariationIdentifier,
    /// The product's title, shared by every variation.
    pub title: String,
    /// The variation's own price, before any discount code. `None` where the product is
    /// announced but not sold (`products-minisforum-s5-all-flash-nas.html` and its German
    /// twin).
    pub price: Option<Money>,
    pub url: String,
    /// The product's images, from Liquid's record.
    pub images: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ProductVariationIdentifier {
    /// `None` on all 1386 variants of the saved pages: the storefront renders no barcode.
    pub gtin: Option<Gtin>,
    /// `None` on 20 of the 1386 saved variants; the Shopify gids still identify them.
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
    // The schema.org offer URL is missing on a page without a price, and is not read for the
    // others either: the record's handle on the storefront origin, in the page's locale, is the
    // canonical URL. `product_url` already carries the `/de` prefix and the percent-encoding.
    if variations < 2 {
        return Ok(product_url.to_owned());
    }

    Ok(format!("{}?variant={}", product_url, gid.numeric_id()?))
}

#[cfg(test)]
mod tests {
    use super::{MinisforumEuDocument, StructuralizeError};
    use crate::minisforumeu::offer_details::v1::normalized::MinisforumEuNormalizedDocument;
    use crate::minisforumeu::offer_details::v1::raw::MinisforumEuRawDocument;
    use crate::minisforumeu::offer_details::v1::raw::tests::document;
    use html_extractor::serde_json::{self, Value, json};
    use retailer_sauce::identifier::IdentifierError;

    /// The raw fixture, normalized, as the JSON structuralize reads.
    fn normalized_document() -> Value {
        let raw: MinisforumEuRawDocument =
            serde_json::from_value(document()).expect("the raw fixture deserializes");
        let normalized =
            MinisforumEuNormalizedDocument::from_raw(raw).expect("the raw fixture normalizes");
        serde_json::to_value(normalized).expect("serializes")
    }

    fn structuralize(input: Value) -> Result<MinisforumEuDocument, StructuralizeError> {
        let normalized: MinisforumEuNormalizedDocument =
            serde_json::from_value(input).expect("the normalized fixture deserializes");
        MinisforumEuDocument::from_normalized(normalized)
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
        let image = "https://minisforumpc.eu/cdn/shop/files/image13.png?v=1770796122";
        let product_url = "https://minisforumpc.eu/products/minisforum-u700-refurbished";

        assert_eq!(
            structuralized(normalized_document()),
            json!({"variations": [
                {
                    "identifiers": {
                        "gtin": null,
                        "sku": "U70082REU",
                        "shopify_product_gid": "gid://shopify/Product/14959267250542",
                        "shopify_product_variation_gid": "gid://shopify/ProductVariant/53296398172526",
                        "shopify_handle": "minisforum-u700-refurbished",
                    },
                    "title": "MINISFORUM U700 Refurbished",
                    "price": {"amount": 12900, "currency": "EUR"},
                    "url": format!("{product_url}?variant=53296398172526"),
                    "images": [image],
                },
                {
                    "identifiers": {
                        "gtin": null,
                        "sku": "U70000REU",
                        "shopify_product_gid": "gid://shopify/Product/14959267250542",
                        "shopify_product_variation_gid": "gid://shopify/ProductVariant/53753234751854",
                        "shopify_handle": "minisforum-u700-refurbished",
                    },
                    "title": "MINISFORUM U700 Refurbished",
                    "price": {"amount": 9900, "currency": "EUR"},
                    "url": format!("{product_url}?variant=53753234751854"),
                    "images": [image],
                },
            ]})
        );
    }

    #[test]
    fn a_single_variation_takes_the_product_url() {
        let output = structuralized(single_variant(normalized_document()));

        assert_eq!(
            output["variations"][0]["url"],
            json!("https://minisforumpc.eu/products/minisforum-u700-refurbished")
        );
    }

    #[test]
    fn a_german_variation_takes_the_prefixed_url() {
        let mut input = normalized_document();
        input["product"]["url"] =
            json!("https://minisforumpc.eu/de/products/minisforum-u700-refurbished");

        let output = structuralized(input);
        assert_eq!(
            output["variations"][0]["url"],
            json!(
                "https://minisforumpc.eu/de/products/minisforum-u700-refurbished?variant=53296398172526"
            )
        );
    }

    #[test]
    fn a_barcode_becomes_the_gtin() {
        let mut input = normalized_document();
        input["product"]["variants"][0]["barcode"] = json!("6971234567890");

        let output = structuralized(input);
        assert_eq!(
            output["variations"][0]["identifiers"]["gtin"],
            json!("6971234567890")
        );
    }

    /// As on `products-minisforum-ms-a1-refurbished.html`, whose second variant has no SKU.
    #[test]
    fn a_variation_without_a_sku_is_kept() {
        let mut input = normalized_document();
        input["product"]["variants"][1]["sku"] = Value::Null;

        let output = structuralized(input);
        assert_eq!(output["variations"][1]["identifiers"]["sku"], Value::Null);
        assert_eq!(
            output["variations"][1]["identifiers"]["shopify_product_variation_gid"],
            json!("gid://shopify/ProductVariant/53753234751854")
        );
    }

    /// As on `products-minisforum-s5-all-flash-nas.html`, announced at €0.
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
