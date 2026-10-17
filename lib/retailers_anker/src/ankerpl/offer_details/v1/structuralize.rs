//! Turns normalized data into the product variations the rest of the system consumes: one
//! entry per variation, carrying its title, identifiers, images, price and URL.
//!
//! The storefront renders the offer twice - as a schema.org island and as its own product record -
//! and normalization keeps both. This is where they become one variation per variant, each
//! field taken from whichever rendering states it at the right scope.

use super::normalized::{self, AnkerPlNormalizedDocument};
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
pub struct AnkerPlDocument {
    pub variations: Vec<ProductVariation>,
}

impl AnkerPlDocument {
    pub fn from_normalized(
        input: AnkerPlNormalizedDocument,
    ) -> Result<AnkerPlDocument, StructuralizeError> {
        let AnkerPlNormalizedDocument {
            ld_json,
            canonical_url,
            product,
        } = input;
        let island = ld_json.product;
        let variations_count = product.variants.len();

        let variations = product
            .variants
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

        Ok(AnkerPlDocument { variations })
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ProductVariation {
    pub identifiers: ProductVariationIdentifier,
    /// The record's, verbatim and shared by every variation: some titles wrap the model number in
    /// `<b>` markup.
    pub title: String,
    /// `None` where the storefront does not sell the variant and renders a placeholder instead.
    pub price: Option<Money>,
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

// The island's `offer.url` is the URL the crawler arrived through, escaped and carrying campaign
// parameters; the page's canonical is clean.
fn variation_url(
    canonical_url: &str,
    gid: &ShopifyProductVariationGid,
    variations: usize,
) -> Result<String, StructuralizeError> {
    if variations < 2 {
        return Ok(canonical_url.to_owned());
    }

    Ok(format!("{}?variant={}", canonical_url, gid.numeric_id()?))
}

#[cfg(test)]
mod tests {
    use super::{AnkerPlDocument, StructuralizeError};
    use crate::ankerpl::offer_details::v1::normalized::AnkerPlNormalizedDocument;
    use crate::shared::IdentifierError;
    use html_extractor::serde_json::{self, Value, json};

    fn pln(amount: i64) -> Value {
        json!({"amount": amount, "currency": "PLN"})
    }

    fn variant(id: u64, sku: &str, label: &str) -> Value {
        json!({
            "id": format!("gid://shopify/ProductVariant/{id}"), "sku": sku,
            "barcode": "194644098117", "name": label, "price": pln(7998),
            "compare_at_price": null, "quantity_available": 13, "available_for_sale": true,
            "currently_not_in_stock": false, "requires_shipping": true, "weight": 0.291,
            "image": {"url": format!("https://cdn.shopify.com/{sku}.png"), "alt_text": null,
                      "width": 1, "height": 1},
            "options": [{"name": "Color", "display_name": "color",
                         "values": [{"label": label, "hex_colors": null}]}]
        })
    }

    fn normalized_document() -> Value {
        json!({
            "ld_json": {"product": {
                "description": "CTA", "images": ["https://cdn.shopify.com/one.png"],
                "sku": "A121D321", "gtin": "194644098117", "brand": "Anker",
                "offer": {
                    "availability": "in_stock", "price": pln(7998), "item_condition": "new",
                    "url": "https://www.anker.com/eu-pl/products/a121d?variant=1&amp;utm_source=google"
                }
            }},
            "canonical_url": "https://www.anker.com/eu-pl/products/a121d",
            "product": {
                "id": "gid://shopify/Product/1", "handle": "a121d",
                "title": "Anker <b>641</b> USB-C to Lightning Cable", "description": "Description",
                "key_features": [], "specifications": [], "product_type": null,
                "published_at": "2025-09-04T06:34:16Z", "price": pln(7998),
                "compare_at_price": null, "available_for_sale": true,
                "options": [{"id": "gid://shopify/ProductOption/7", "name": "Color",
                             "display_name": "color",
                             "values": [{"label": "Black", "hex_colors": null},
                                        {"label": "White", "hex_colors": null}]}],
                "variants": [variant(1, "A121D321", "Black"), variant(2, "A121D311", "White")]
            }
        })
    }

    fn structuralized(input: &Value) -> Value {
        let result = crate::ankerpl::offer_details::v1::structuralize(&input.to_string())
            .expect("the document structuralizes");

        serde_json::from_str(&result.document_json.unwrap()).unwrap()
    }

    fn error(input: Value) -> StructuralizeError {
        let normalized: AnkerPlNormalizedDocument = serde_json::from_value(input).unwrap();

        AnkerPlDocument::from_normalized(normalized).unwrap_err()
    }

    fn urls(output: &Value) -> Vec<Value> {
        output["variations"]
            .as_array()
            .unwrap()
            .iter()
            .map(|variation| variation["url"].clone())
            .collect()
    }

    #[test]
    fn a_variation_carries_its_title_identifiers_images_price_and_url_only() {
        let mut input = normalized_document();
        input["product"]["variants"]
            .as_array_mut()
            .unwrap()
            .truncate(1);

        assert_eq!(
            structuralized(&input),
            json!({"variations": [{
                "title": "Anker <b>641</b> USB-C to Lightning Cable",
                "identifiers": {
                    "gtin": "194644098117",
                    "sku": "A121D321",
                    "shopify_product_gid": "gid://shopify/Product/1",
                    "shopify_product_variation_gid": "gid://shopify/ProductVariant/1",
                    "shopify_handle": "a121d",
                },
                "images": ["https://cdn.shopify.com/one.png"],
                "price": pln(7998),
                "url": "https://www.anker.com/eu-pl/products/a121d",
            }]})
        );
    }

    #[test]
    fn each_variation_of_a_multi_variant_document_gets_its_own_url() {
        assert_eq!(
            urls(&structuralized(&normalized_document())),
            [
                "https://www.anker.com/eu-pl/products/a121d?variant=1",
                "https://www.anker.com/eu-pl/products/a121d?variant=2"
            ]
        );
    }

    #[test]
    fn a_single_variant_document_gets_the_bare_canonical() {
        let mut input = normalized_document();
        input["product"]["variants"]
            .as_array_mut()
            .unwrap()
            .truncate(1);

        assert_eq!(
            urls(&structuralized(&input)),
            ["https://www.anker.com/eu-pl/products/a121d"]
        );
    }

    #[test]
    fn the_islands_arrival_url_reaches_nothing() {
        let document =
            crate::ankerpl::offer_details::v1::structuralize(&normalized_document().to_string())
                .unwrap()
                .document_json
                .unwrap();

        for leaked in ["&amp;", "utm_source", "variant=1&"] {
            assert!(!document.contains(leaked), "{leaked} survived: {document}");
        }
    }

    #[test]
    fn the_title_crosses_with_the_records_markup() {
        assert_eq!(
            structuralized(&normalized_document())["variations"][0]["title"],
            "Anker <b>641</b> USB-C to Lightning Cable"
        );
    }

    #[test]
    fn a_variant_the_storefront_does_not_sell_has_no_price_and_its_siblings_keep_theirs() {
        let mut input = normalized_document();
        input["product"]["variants"][0]["price"] = Value::Null;

        let output = structuralized(&input);

        assert_eq!(output["variations"][0]["price"], Value::Null);
        assert_eq!(output["variations"][1]["price"], pln(7998));
    }

    #[test]
    fn an_unusable_identifier_crosses_as_the_identifier_failure() {
        let mut input = normalized_document();
        input["product"]["variants"][0]["sku"] = json!("");

        assert_eq!(
            error(input),
            StructuralizeError::Identifier(IdentifierError::EmptySku)
        );
    }
}
