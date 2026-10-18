//! Turns normalized data into the product variations the rest of the system consumes: one
//! entry per variation, carrying its title, identifiers, images, price and URL.

use super::normalized::{self, AnkerAuNormalizedDocument};
use crate::shared::{
    Gtin, IdentifierError, ShopifyHandle, ShopifyProductGid, ShopifyProductVariationGid, Sku,
};
use money::Money;
use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Error)]
pub enum StructuralizeError {
    #[error(transparent)]
    Identifier(#[from] IdentifierError),
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AnkerAuDocument {
    pub variations: Vec<ProductVariation>,
}

impl AnkerAuDocument {
    pub fn from_normalized(
        input: AnkerAuNormalizedDocument,
    ) -> Result<AnkerAuDocument, StructuralizeError> {
        let AnkerAuNormalizedDocument {
            ld_json,
            product,
            variants,
            canonical_url,
        } = input;
        let island = ld_json.product;
        let variations_count = variants.len();

        let variations = variants
            .iter()
            .map(|variant| {
                let identifiers = ProductVariationIdentifier::from_normalized(&product, variant)?;
                let url = variation_url(
                    &canonical_url,
                    &identifiers.shopify_product_variation_gid,
                    variations_count,
                )?;

                Ok(ProductVariation {
                    title: product.title.clone(),
                    identifiers,
                    images: island.images.clone(),
                    price: variant.price.clone(),
                    url,
                })
            })
            .collect::<Result<_, StructuralizeError>>()?;

        Ok(AnkerAuDocument { variations })
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ProductVariation {
    pub identifiers: ProductVariationIdentifier,
    /// The product's title, shared by every variation.
    pub title: String,
    /// The variation's own price, before any discount code.
    pub price: Money,
    pub url: String,
    /// The product's images, from the `ld+json` island.
    pub images: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ProductVariationIdentifier {
    pub gtin: Option<Gtin>,
    pub sku: Sku,
    pub shopify_product_gid: ShopifyProductGid,
    pub shopify_product_variation_gid: ShopifyProductVariationGid,
    pub shopify_handle: ShopifyHandle,
}

impl ProductVariationIdentifier {
    fn from_normalized(
        product: &normalized::ProductRecord,
        variant: &normalized::Variant,
    ) -> Result<ProductVariationIdentifier, StructuralizeError> {
        Ok(ProductVariationIdentifier {
            gtin: variant.barcode.clone().map(Gtin::try_from).transpose()?,
            sku: variant.sku.clone().try_into()?,
            shopify_product_gid: product.id.clone().try_into()?,
            shopify_product_variation_gid: variant.id.clone().try_into()?,
            shopify_handle: product.handle.clone().try_into()?,
        })
    }
}

fn variation_url(
    canonical_url: &str,
    gid: &ShopifyProductVariationGid,
    variations: usize,
) -> Result<String, StructuralizeError> {
    // The JSON-LD offer URL may retain tracking parameters and a different selected variant.
    if variations < 2 {
        return Ok(canonical_url.to_owned());
    }

    Ok(format!("{}?variant={}", canonical_url, gid.numeric_id()?))
}

#[cfg(test)]
mod tests {
    use super::{AnkerAuDocument, AnkerAuNormalizedDocument, StructuralizeError};
    use crate::shared::IdentifierError;
    use html_extractor::serde_json::{self, json};

    #[test]
    fn an_unusable_identifier_crosses_as_the_identifier_failure() {
        let mut input = normalized_document();
        input["variants"][0]["sku"] = json!("");

        let normalized: AnkerAuNormalizedDocument =
            serde_json::from_value(input).expect("an empty SKU still deserializes");

        let error = AnkerAuDocument::from_normalized(normalized)
            .expect_err("a variant with no SKU cannot structuralize");

        assert_eq!(
            error,
            StructuralizeError::Identifier(IdentifierError::EmptySku)
        );
    }

    fn normalized_document() -> serde_json::Value {
        let price = json!({"amount": 22999, "currency": "AUD"});
        let variant = |id, sku, quantity, available| {
            json!({
                "id": id, "sku": sku, "barcode": null, "name": sku,
                "price": price, "compare_at_price": null,
                "quantity_available": quantity, "available_for_sale": available,
                "currently_not_in_stock": false, "requires_shipping": true, "weight": 2.0,
                "image": {"url": "https://example.com/image.png", "width": 100, "height": 100},
                "options": [], "discounts": []
            })
        };
        json!({
            "ld_json": {"product": {
                "brand": "Anker", "description": "CTA", "images": [], "sku": "A1",
                "offer": {
                    "item_condition": "new", "price": price,
                    "url": "https://www.anker.com/au/products/test"
                }
            }},
            "canonical_url": "https://www.anker.com/au/products/test",
            "product": {
                "id": "gid://shopify/Product/1", "handle": "test", "title": "Test",
                "description": "Description", "key_features": [],
                "specifications": [],
                "published_at": "2026-09-21T00:00:00Z",
                "price": price, "compare_at_price": null, "available_for_sale": true, "options": []
            },
            "variants": [
                variant("gid://shopify/ProductVariant/1", "A1", 0, false),
                variant("gid://shopify/ProductVariant/2", "A2", 13, true)
            ]
        })
    }

    #[test]
    fn a_variation_carries_its_title_identifiers_images_price_and_url_only() {
        let mut input = normalized_document();
        input["variants"].as_array_mut().unwrap().truncate(1);
        input["variants"][0]["barcode"] = json!("194644000000");
        input["ld_json"]["product"]["images"] = json!(["https://example.com/a.png"]);

        assert_eq!(
            structuralized(&input),
            json!({"variations": [{
                "title": "Test",
                "identifiers": {
                    "gtin": "194644000000",
                    "sku": "A1",
                    "shopify_product_gid": "gid://shopify/Product/1",
                    "shopify_product_variation_gid": "gid://shopify/ProductVariant/1",
                    "shopify_handle": "test",
                },
                "images": ["https://example.com/a.png"],
                "price": {"amount": 22999, "currency": "AUD"},
                "url": "https://www.anker.com/au/products/test",
            }]})
        );
    }

    #[test]
    fn each_variation_of_a_multi_variant_document_gets_its_own_url() {
        let output = structuralized(&normalized_document());

        assert_eq!(
            urls(&output),
            [
                "https://www.anker.com/au/products/test?variant=1",
                "https://www.anker.com/au/products/test?variant=2"
            ]
        );
    }

    #[test]
    fn a_single_variant_document_gets_the_bare_canonical() {
        let mut input = normalized_document();
        input["variants"].as_array_mut().unwrap().truncate(1);

        assert_eq!(
            urls(&structuralized(&input)),
            ["https://www.anker.com/au/products/test"]
        );
    }

    #[test]
    fn the_islands_arrival_url_reaches_nothing() {
        let mut input = normalized_document();
        input["ld_json"]["product"]["offer"]["url"] = json!(
            "https://www.anker.com/au/products/test?utm_source=offline&amp;ref=chargers&amp;variant=99"
        );

        let document = crate::ankerau::offer_details::v1::structuralize(&input.to_string())
            .unwrap()
            .document_json
            .unwrap();

        for leaked in ["&amp;", "utm_source", "ref=chargers", "variant=99"] {
            assert!(!document.contains(leaked), "{leaked} survived: {document}");
        }
    }

    #[test]
    fn a_variant_id_without_a_numeric_tail_fails_the_document() {
        let mut input = normalized_document();
        input["variants"][0]["id"] = json!("gid://shopify/ProductVariant/not-a-number");

        let Err(error) = crate::ankerau::offer_details::v1::structuralize(&input.to_string())
        else {
            panic!("a malformed variant id cannot build a URL");
        };

        assert!(error.contains("does not end in a numeric id"), "{error}");
    }

    fn structuralized(input: &serde_json::Value) -> serde_json::Value {
        let result = crate::ankerau::offer_details::v1::structuralize(&input.to_string())
            .expect("the document structuralizes");

        serde_json::from_str(&result.document_json.unwrap()).unwrap()
    }

    fn urls(output: &serde_json::Value) -> Vec<String> {
        output["variations"]
            .as_array()
            .unwrap()
            .iter()
            .map(|variation| variation["url"].as_str().unwrap().to_string())
            .collect()
    }
}
