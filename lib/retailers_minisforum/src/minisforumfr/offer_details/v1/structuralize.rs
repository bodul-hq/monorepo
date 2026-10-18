//! Turns normalized data into the product variations the rest of the system consumes: one
//! entry per variation, carrying its title, identifiers, images, price and URL.

use super::normalized::{MinisforumFrNormalizedDocument, product};
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
pub struct MinisforumFrDocument {
    pub variations: Vec<ProductVariation>,
}

impl MinisforumFrDocument {
    pub fn from_normalized(
        input: MinisforumFrNormalizedDocument,
    ) -> Result<MinisforumFrDocument, StructuralizeError> {
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

        Ok(MinisforumFrDocument { variations })
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ProductVariation {
    pub identifiers: ProductVariationIdentifier,
    /// The product's title, shared by every variation.
    pub title: String,
    /// The variation's own price, before any discount code. `None` where the variation is
    /// announced but not sold (6 of the 376 saved variants, on `products-minisforum-ai-x1.html`
    /// and `products-minisforum-s5-tout-flash-nas.html` and their English twins).
    pub price: Option<Money>,
    pub url: String,
    /// The product's images, from Liquid's record.
    pub images: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ProductVariationIdentifier {
    /// `None` on all 376 variants of the saved pages: the storefront renders no barcode.
    pub gtin: Option<Gtin>,
    /// `None` on 32 of the 376 saved variants (null on 30, empty on 2); the Shopify gids still
    /// identify them.
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
    // others either: the normalized product URL, the record's handle under the page's language
    // on the storefront origin, is the canonical URL.
    if variations < 2 {
        return Ok(product_url.to_owned());
    }

    Ok(format!("{}?variant={}", product_url, gid.numeric_id()?))
}

#[cfg(test)]
mod tests {
    use super::{MinisforumFrDocument, StructuralizeError};
    use crate::minisforumfr::offer_details::v1::normalized::MinisforumFrNormalizedDocument;
    use crate::minisforumfr::offer_details::v1::raw::MinisforumFrRawDocument;
    use crate::minisforumfr::offer_details::v1::raw::tests::document;
    use html_extractor::serde_json::{self, Value, json};
    use retailer_sauce::identifier::IdentifierError;

    /// The raw fixture, normalized, as the JSON structuralize reads.
    fn normalized_document() -> Value {
        let raw: MinisforumFrRawDocument =
            serde_json::from_value(document()).expect("the raw fixture deserializes");
        let normalized =
            MinisforumFrNormalizedDocument::from_raw(raw).expect("the raw fixture normalizes");
        serde_json::to_value(normalized).expect("serializes")
    }

    fn structuralize(input: Value) -> Result<MinisforumFrDocument, StructuralizeError> {
        let normalized: MinisforumFrNormalizedDocument =
            serde_json::from_value(input).expect("the normalized fixture deserializes");
        MinisforumFrDocument::from_normalized(normalized)
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
        let image = "https://minisforumpc.fr/cdn/shop/files/ms-03-local-ai-edge-workstation-1.png?v=1788346698";
        let product_url = "https://minisforumpc.fr/products/minisforum-ms-03-workstation";

        assert_eq!(
            structuralized(normalized_document()),
            json!({"variations": [
                {
                    "identifiers": {
                        "gtin": null,
                        "sku": "MS-03-33600EU",
                        "shopify_product_gid": "gid://shopify/Product/15885734478193",
                        "shopify_product_variation_gid": "gid://shopify/ProductVariant/64248383471985",
                        "shopify_handle": "minisforum-ms-03-workstation",
                    },
                    "title": "Minisforum MS-03 Workstation Locale IA Edge",
                    "price": {"amount": 79900, "currency": "EUR"},
                    "url": format!("{product_url}?variant=64248383471985"),
                    "images": [image],
                },
                {
                    "identifiers": {
                        "gtin": null,
                        "sku": "MS-03-33631EU",
                        "shopify_product_gid": "gid://shopify/Product/15885734478193",
                        "shopify_product_variation_gid": "gid://shopify/ProductVariant/64248383504753",
                        "shopify_handle": "minisforum-ms-03-workstation",
                    },
                    "title": "Minisforum MS-03 Workstation Locale IA Edge",
                    "price": {"amount": 139900, "currency": "EUR"},
                    "url": format!("{product_url}?variant=64248383504753"),
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
            json!("https://minisforumpc.fr/products/minisforum-ms-03-workstation")
        );
    }

    /// The French and the English page of a product are different pages of the same Shopify
    /// product.
    #[test]
    fn an_english_variation_keeps_the_en_prefix() {
        let mut input = normalized_document();
        input["product"]["url"] =
            json!("https://minisforumpc.fr/en/products/minisforum-ms-03-workstation");

        let output = structuralized(input);
        assert_eq!(
            output["variations"][1]["url"],
            json!(
                "https://minisforumpc.fr/en/products/minisforum-ms-03-workstation?variant=64248383504753"
            )
        );
        assert_eq!(
            output["variations"][1]["identifiers"]["shopify_product_gid"],
            json!("gid://shopify/Product/15885734478193")
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

    /// As on `products-minisforum-ai-x1.html`, some of whose variants have no SKU.
    #[test]
    fn a_variation_without_a_sku_is_kept() {
        let mut input = normalized_document();
        input["product"]["variants"][1]["sku"] = Value::Null;

        let output = structuralized(input);
        assert_eq!(output["variations"][1]["identifiers"]["sku"], Value::Null);
        assert_eq!(
            output["variations"][1]["identifiers"]["shopify_product_variation_gid"],
            json!("gid://shopify/ProductVariant/64248383504753")
        );
    }

    /// As on `products-minisforum-s5-tout-flash-nas.html`, announced at €0.
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
