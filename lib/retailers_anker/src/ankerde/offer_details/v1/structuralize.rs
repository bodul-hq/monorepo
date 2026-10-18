//! Turns normalized data into the product variations the rest of the system consumes: one
//! entry per variation, carrying its title, identifiers, images, price and URL.
//!
//! The storefront renders the offer twice - as a schema.org island and as its own product record -
//! and normalization keeps both. Each field is taken from whichever rendering states it cleanly.

use super::normalized::{self, AnkerdeNormalizedDocument};
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
pub struct AnkerDeDocument {
    pub variations: Vec<ProductVariation>,
}

impl AnkerDeDocument {
    pub fn from_normalized(
        input: AnkerdeNormalizedDocument,
    ) -> Result<AnkerDeDocument, StructuralizeError> {
        let AnkerdeNormalizedDocument { ld_json, product } = input;
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
                    title: product.title.clone(),
                    identifiers,
                    images: gallery(&product),
                    price: variant.price.clone(),
                    url,
                })
            })
            .collect::<Result<_, StructuralizeError>>()?;

        Ok(AnkerDeDocument { variations })
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ProductVariation {
    pub identifiers: ProductVariationIdentifier,
    /// The product's title, from the record, shared by every variation.
    pub title: String,
    /// The variation's own price, before any discount code.
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
    use super::{AnkerDeDocument, StructuralizeError};
    use crate::ankerde::offer_details::v1::normalized::AnkerdeNormalizedDocument;
    use crate::shared::IdentifierError;
    use html_extractor::serde_json::{self, Value, json};

    fn eur(amount: i64) -> Value {
        json!({"amount": amount, "currency": "EUR"})
    }

    fn variant(id: u64, sku: &str, colour: &str) -> Value {
        json!({
            "id": format!("gid://shopify/ProductVariant/{id}"), "title": colour, "sku": sku,
            "available_for_sale": true, "requires_shipping": true, "quantity_available": 3,
            "price": eur(15999), "compare_at_price": null, "weight": 1.052,
            "image": {"url": format!("https://cdn.shopify.com/{sku}.png")},
            "selected_options": [{"name": "Farbe", "value": colour}],
            "selling_plan_allocations": [], "discounts": []
        })
    }

    fn normalized_document() -> Value {
        json!({
            "ld_json": {"product": {
                "description": "CTA", "url": "https://www.anker.com/de/products/a2345",
                "brand": "Anker",
                "offer": {"availability": "in_stock", "price": eur(15999)}
            }},
            "product": {
                "id": "gid://shopify/Product/1", "handle": "a2345",
                "title": "Anker Prime Ladegerät", "vendor": "Soundcore", "product_type": "",
                "description": "Beschreibung", "available_for_sale": true,
                "created_at": "2024-01-01T00:00:00Z", "updated_at": "2024-01-02T00:00:00Z",
                "published_at": "2024-01-03T00:00:00Z",
                "seo": {"title": null, "description": null},
                "price": eur(15999), "compare_at_price": null,
                "featured_image": {"url": "https://cdn.shopify.com/one.png"},
                "images": [{"url": "https://cdn.shopify.com/one.png"},
                           {"url": "https://cdn.shopify.com/two.png"}],
                "options": [{"id": "gid://shopify/ProductOption/7", "name": "Farbe",
                             "values": ["Schwarz", "Weiß"]}],
                "variants": [variant(1, "A2345341", "Schwarz"), variant(2, "A2345321", "Weiß")],
                "selling_plan_groups": []
            }
        })
    }

    fn structuralized(input: &Value) -> Value {
        let result = crate::ankerde::offer_details::v1::structuralize(&input.to_string())
            .expect("the document structuralizes");

        serde_json::from_str(&result.document_json.unwrap()).unwrap()
    }

    fn error(input: Value) -> StructuralizeError {
        let normalized: AnkerdeNormalizedDocument = serde_json::from_value(input).unwrap();

        AnkerDeDocument::from_normalized(normalized).unwrap_err()
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
                "identifiers": {
                    "sku": "A2345341",
                    "shopify_product_gid": "gid://shopify/Product/1",
                    "shopify_product_variation_gid": "gid://shopify/ProductVariant/1",
                    "shopify_handle": "a2345",
                },
                "title": "Anker Prime Ladegerät",
                "price": eur(15999),
                "url": "https://www.anker.com/de/products/a2345",
                "images": [
                    "https://cdn.shopify.com/one.png",
                    "https://cdn.shopify.com/two.png"
                ],
            }]})
        );
    }

    #[test]
    fn each_variation_of_a_multi_variant_document_gets_its_own_url() {
        assert_eq!(
            urls(&structuralized(&normalized_document())),
            [
                "https://www.anker.com/de/products/a2345?variant=1",
                "https://www.anker.com/de/products/a2345?variant=2"
            ]
        );
    }

    #[test]
    fn a_single_variant_document_gets_the_bare_island_url() {
        let mut input = normalized_document();
        input["product"]["variants"]
            .as_array_mut()
            .unwrap()
            .truncate(1);

        assert_eq!(
            urls(&structuralized(&input)),
            ["https://www.anker.com/de/products/a2345"]
        );
    }

    #[test]
    fn the_title_and_gallery_come_from_the_record() {
        let variation = structuralized(&normalized_document())["variations"][0].clone();

        assert_eq!(variation["title"], "Anker Prime Ladegerät");
        assert_eq!(
            variation["images"],
            json!([
                "https://cdn.shopify.com/one.png",
                "https://cdn.shopify.com/two.png"
            ])
        );
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
    fn an_unusable_identifier_crosses_as_the_identifier_failure() {
        let mut input = normalized_document();
        input["product"]["handle"] = json!("");

        assert_eq!(
            error(input),
            StructuralizeError::Identifier(IdentifierError::EmptyShopifyHandle)
        );
    }
}
