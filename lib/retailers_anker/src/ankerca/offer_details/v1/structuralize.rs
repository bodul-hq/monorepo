//! Turns normalized data into the product variations the rest of the system consumes: one
//! entry per variation, carrying its title, identifiers, images, price and URL.
//!
//! The storefront renders the same offer twice - as a schema.org island and as its own product
//! record - and normalization keeps both. Each field is taken from whichever rendering states it
//! cleanly.

use super::normalized::{self, AnkerCaNormalizedDocument};
use crate::shared::{
    IdentifierError, ShopifyHandle, ShopifyProductGid, ShopifyProductVariationGid, Sku,
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
pub struct AnkerCaDocument {
    pub variations: Vec<ProductVariation>,
}

impl AnkerCaDocument {
    pub fn from_normalized(
        input: AnkerCaNormalizedDocument,
    ) -> Result<AnkerCaDocument, StructuralizeError> {
        let AnkerCaNormalizedDocument { ld_json, product } = input;
        let island = ld_json.product;
        let variations_count = product.variants.len();

        let variations = product
            .variants
            .iter()
            .map(|variant| {
                let identifiers = ProductVariationIdentifier::from_normalized(&product, variant)?;
                let url = variation_url(
                    &island.url,
                    &identifiers.shopify_product_variation_gid,
                    variations_count,
                )?;

                Ok(ProductVariation {
                    title: island.name.clone(),
                    identifiers,
                    images: gallery(&product),
                    price: variant.price.clone(),
                    url,
                })
            })
            .collect::<Result<_, StructuralizeError>>()?;

        Ok(AnkerCaDocument { variations })
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ProductVariation {
    pub identifiers: ProductVariationIdentifier,
    /// The product's title, shared by every variation. From the island: record titles contain
    /// `<b>` markup on some pages; island names are plain text.
    pub title: String,
    /// The variation's own price.
    pub price: Money,
    pub url: String,
    /// The product's gallery, from the record.
    pub images: Vec<String>,
}

/// No GTIN is available: the storefront's query does not select a barcode.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ProductVariationIdentifier {
    /// Some variants have no merchant SKU; Shopify identifiers still identify them.
    pub sku: Option<Sku>,
    pub shopify_product_gid: ShopifyProductGid,
    pub shopify_product_variation_gid: ShopifyProductVariationGid,
    pub shopify_handle: ShopifyHandle,
}

impl ProductVariationIdentifier {
    fn from_normalized(
        product: &normalized::product::ProductRecord,
        variant: &normalized::product::ProductVariant,
    ) -> Result<ProductVariationIdentifier, StructuralizeError> {
        Ok(ProductVariationIdentifier {
            sku: (!variant.sku.trim().is_empty())
                .then(|| Sku::try_from(variant.sku.clone()))
                .transpose()?,
            shopify_product_gid: product.id.clone().try_into()?,
            shopify_product_variation_gid: variant.id.clone().try_into()?,
            shopify_handle: product.handle.clone().try_into()?,
        })
    }
}

// The record's gallery rather than the island's `images`, which despite its name renders one URL.
fn gallery(product: &normalized::product::ProductRecord) -> Vec<String> {
    product
        .images
        .iter()
        .map(|image| image.url.clone())
        .collect()
}

// The island supplies a clean base URL in the inspected corpus, without queries or HTML escaping.
fn variation_url(
    island_url: &str,
    gid: &ShopifyProductVariationGid,
    variations: usize,
) -> Result<String, StructuralizeError> {
    if variations < 2 {
        return Ok(island_url.to_owned());
    }

    Ok(format!("{}?variant={}", island_url, gid.numeric_id()?))
}

#[cfg(test)]
mod tests {
    use super::{AnkerCaDocument, StructuralizeError};
    use crate::ankerca::offer_details::v1::normalized::AnkerCaNormalizedDocument;
    use crate::shared::IdentifierError;
    use html_extractor::serde_json::{self, Value, json};

    #[test]
    fn a_variation_carries_its_title_identifiers_images_price_and_url_only() {
        let mut input = normalized_document();
        input["product"]["variants"] = json!([variant("1", "A3331Z01", "Black")]);

        assert_eq!(
            structuralized(&input),
            json!({"variations": [{
                "identifiers": {
                    "sku": "A3331Z01",
                    "shopify_product_gid": "gid://shopify/Product/8302268514493",
                    "shopify_product_variation_gid": "gid://shopify/ProductVariant/1",
                    "shopify_handle": "a3331-c40i-clip-earbuds",
                },
                "title": "soundcore C40i | Clip-On Open Earbuds",
                "price": {"amount": 10999, "currency": "CAD"},
                "url": "https://www.anker.com/ca-fr/products/a3331-c40i-clip-earbuds",
                "images": [
                    "https://cdn.shopify.com/one.png",
                    "https://cdn.shopify.com/two.png"
                ],
            }]})
        );
    }

    #[test]
    fn a_single_variant_document_gets_the_bare_island_url() {
        let mut input = normalized_document();
        input["product"]["variants"] = json!([variant("1", "A3331Z01", "Black")]);

        assert_eq!(
            urls(&structuralized(&input)),
            vec!["https://www.anker.com/ca-fr/products/a3331-c40i-clip-earbuds"]
        );
    }

    #[test]
    fn each_variation_of_a_multi_variant_document_gets_its_own_url() {
        assert_eq!(
            urls(&structuralized(&normalized_document())),
            vec![
                "https://www.anker.com/ca-fr/products/a3331-c40i-clip-earbuds?variant=1",
                "https://www.anker.com/ca-fr/products/a3331-c40i-clip-earbuds?variant=2",
            ]
        );
    }

    #[test]
    fn the_title_comes_from_the_island_so_record_markup_does_not_cross() {
        let mut input = normalized_document();
        input["product"]["title"] = json!("Hub USB-C Anker <b>332</b> (5 en 1)");
        input["ld_json"]["product"]["name"] = json!("Hub USB-C Anker 332 (5 en 1)");

        let output = structuralized(&input);

        assert_eq!(
            output["variations"][0]["title"],
            json!("Hub USB-C Anker 332 (5 en 1)")
        );
        assert!(
            !output.to_string().contains("<b>"),
            "the record's markup must not reach any field of the document"
        );
    }

    #[test]
    fn the_gallery_comes_from_the_record_not_the_islands_single_url() {
        let output = structuralized(&normalized_document());

        assert_eq!(
            output["variations"][0]["images"],
            json!([
                "https://cdn.shopify.com/one.png",
                "https://cdn.shopify.com/two.png"
            ])
        );
    }

    #[test]
    fn a_page_without_images_structuralizes_without_them() {
        let mut input = normalized_document();
        input["ld_json"]["product"]["images"] = Value::Null;
        input["product"]["featured_image"] = Value::Null;
        input["product"]["images"] = json!([]);
        let mut only = variant("1", "A3968Z21", "White");
        only["image"] = Value::Null;
        input["product"]["variants"] = json!([only]);

        assert_eq!(structuralized(&input)["variations"][0]["images"], json!([]));
    }

    #[test]
    fn an_empty_sku_keeps_the_variation_with_its_shopify_identifiers() {
        for sku in ["", " \t "] {
            let mut input = normalized_document();
            input["product"]["variants"][0]["sku"] = json!(sku);
            let output = structuralized(&input);
            assert!(output["variations"][0]["identifiers"]["sku"].is_null());
            assert_eq!(
                output["variations"][0]["identifiers"]["shopify_product_variation_gid"],
                input["product"]["variants"][0]["id"]
            );
            assert_eq!(
                output["variations"].as_array().unwrap().len(),
                input["product"]["variants"].as_array().unwrap().len()
            );
            assert_eq!(
                output["variations"][0]["price"],
                input["product"]["variants"][0]["price"]
            );
        }
    }

    #[test]
    fn a_missing_handle_still_fails_identifier_validation() {
        let mut input = normalized_document();
        input["product"]["handle"] = json!("");

        assert_eq!(
            error(&input),
            StructuralizeError::Identifier(IdentifierError::EmptyShopifyHandle)
        );
    }

    #[test]
    fn a_variant_id_without_a_numeric_tail_fails_the_document() {
        let mut input = normalized_document();
        input["product"]["variants"][0]["id"] = json!("gid://shopify/ProductVariant/not-a-number");

        let message = structuralize_error_message(&input);

        assert!(
            message.contains("does not end in a numeric id"),
            "unexpected message: {message}"
        );
    }

    #[test]
    fn a_document_without_a_product_island_never_reaches_this_stage() {
        let mut input = normalized_document();
        input["ld_json"]["product"] = Value::Null;

        serde_json::from_value::<AnkerCaNormalizedDocument>(input)
            .expect_err("normalization requires the island, so it cannot be absent here");
    }

    fn variant(id: &str, sku: &str, color: &str) -> Value {
        json!({
            "id": format!("gid://shopify/ProductVariant/{id}"),
            "title": color,
            "sku": sku,
            "available_for_sale": true,
            "requires_shipping": true,
            "quantity_available": 34,
            "price": {"amount": 10999, "currency": "CAD"},
            "compare_at_price": null,
            "weight": 0.3616,
            "image": {"url": "https://cdn.shopify.com/variant.png"},
            "selected_options": [{"name": "Color", "value": color}],
            "selling_plan_allocations": [],
        })
    }

    fn normalized_document() -> Value {
        json!({
            "ld_json": {
                "product": {
                    "name": "soundcore C40i | Clip-On Open Earbuds",
                    "description": "Ultra-flexible clip-on open earbuds.",
                    "images": "https://cdn.shopify.com/island-only.png",
                    "url": "https://www.anker.com/ca-fr/products/a3331-c40i-clip-earbuds",
                    "brand": "Anker",
                    "offer": {
                        "availability": "in_stock",
                        "price": {"amount": 10999, "currency": "CAD"},
                    },
                },
            },
            "product": {
                "id": "gid://shopify/Product/8302268514493",
                "handle": "a3331-c40i-clip-earbuds",
                "title": "soundcore C40i | Clip-On Open Earbuds",
                "vendor": "Soundcore",
                "product_type": "Earbuds",
                "description": "The record's own copy, which differs from the island's.",
                "available_for_sale": true,
                "created_at": "2026-05-13T10:27:03Z",
                "updated_at": "2026-09-01T03:18:48Z",
                "published_at": "2026-05-25T03:41:47Z",
                "seo": {"title": null, "description": null},
                "price": {"amount": 10999, "currency": "CAD"},
                "compare_at_price": null,
                "featured_image": {"url": "https://cdn.shopify.com/one.png"},
                "images": [
                    {"url": "https://cdn.shopify.com/one.png"},
                    {"url": "https://cdn.shopify.com/two.png"},
                ],
                "options": [{
                    "id": "gid://shopify/ProductOption/9",
                    "name": "Color",
                    "values": ["Black", "Midnight"],
                }],
                "variants": [variant("1", "A3331Z01", "Black"), variant("2", "A3331Z02", "Midnight")],
                "selling_plan_groups": [],
            },
        })
    }

    fn structuralized(input: &Value) -> Value {
        let result = crate::ankerca::offer_details::v1::structuralize(&input.to_string())
            .expect("the normalized document structuralizes");

        let document_json = result
            .document_json
            .expect("ankerca hands over a structuralized document");

        serde_json::from_str(&document_json).expect("the document is JSON")
    }

    fn structuralize_error_message(input: &Value) -> String {
        match crate::ankerca::offer_details::v1::structuralize(&input.to_string()) {
            Err(message) => message,
            Ok(_) => panic!("the document must fail to structuralize"),
        }
    }

    fn error(input: &Value) -> StructuralizeError {
        let normalized: AnkerCaNormalizedDocument =
            serde_json::from_value(input.clone()).expect("the fixture is a normalized document");

        AnkerCaDocument::from_normalized(normalized)
            .expect_err("the document must fail to structuralize")
    }

    fn urls(output: &Value) -> Vec<String> {
        output["variations"]
            .as_array()
            .expect("variations is a list")
            .iter()
            .map(|variation| variation["url"].as_str().unwrap().to_string())
            .collect()
    }
}
