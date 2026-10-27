//! Turns normalized data into the product variations the rest of the system consumes: one
//! entry per variation, carrying its title, identifiers, images, price and URL.

use super::normalized::{self, AnkerEuNormalizedDocument};
use crate::offer_facts::{self, Availability, Content, ItemCondition};
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
pub struct AnkerEuDocument {
    pub variations: Vec<ProductVariation>,
}

impl AnkerEuDocument {
    pub fn from_normalized(
        input: AnkerEuNormalizedDocument,
    ) -> Result<AnkerEuDocument, StructuralizeError> {
        let AnkerEuNormalizedDocument {
            ld_json,
            canonical_url,
            product,
        } = input;
        let island = ld_json.product;

        let variations_count = product.variants.len();
        let mut variations = Vec::with_capacity(variations_count);

        for variant in &product.variants {
            let identifiers = ProductVariationIdentifier::from_normalized(&product, variant)?;
            let url = variation_url(
                &canonical_url,
                &identifiers.shopify_product_variation_gid,
                variations_count,
            )?;

            variations.push(ProductVariation {
                was_price: offer_facts::was_price(
                    Some(&variant.price),
                    variant.compare_at_price.as_ref(),
                ),
                availability: Some(
                    if offer_facts::unique_sku(
                        &island.sku,
                        &variant.sku,
                        product.variants.iter().map(|v| v.sku.as_str()),
                    ) {
                        island.offer.availability.clone()
                    } else {
                        offer_facts::purchase_available(variant.available_for_sale)
                    },
                ),
                item_condition: (offer_facts::unique_sku(
                    &island.sku,
                    &variant.sku,
                    product.variants.iter().map(|v| v.sku.as_str()),
                ))
                .then(|| island.offer.item_condition.clone()),
                content: offer_facts::content(Some(product.description.as_str())),
                identifiers,
                title: product.title.clone(),
                price: variant.price.clone(),
                url,
                images: island.images.clone(),
            });
        }

        Ok(AnkerEuDocument { variations })
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ProductVariation {
    pub was_price: Option<Money>,
    pub availability: Option<Availability>,
    pub item_condition: Option<ItemCondition>,
    pub content: Content,
    pub identifiers: ProductVariationIdentifier,
    /// The product's title, shared by every variation. The record's, verbatim: some titles wrap the
    /// model number in `<b>` markup.
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
    use super::{AnkerEuDocument, StructuralizeError};
    use crate::ankereu::offer_details::v1::normalized::AnkerEuNormalizedDocument;
    use crate::shared::IdentifierError;
    use html_extractor::serde_json::{self, Value, json};

    fn eur(amount: i64) -> Value {
        json!({"amount": amount, "currency": "EUR"})
    }

    fn variant(id: u64, sku: &str, label: &str) -> Value {
        json!({
            "id": format!("gid://shopify/ProductVariant/{id}"), "sku": sku,
            "barcode": "194644098117", "name": label, "price": eur(7998),
            "compare_at_price": null, "quantity_available": 13, "available_for_sale": true,
            "currently_not_in_stock": false, "requires_shipping": true, "weight": 0.291,
            "image": {"url": format!("https://cdn.shopify.com/{sku}.png"), "alt_text": null,
                      "width": 1, "height": 1},
            "options": [{"name": "Color", "display_name": "color",
                         "values": [{"label": label, "hex_colors": null}]}],
            "discounts": []
        })
    }

    fn normalized_document() -> Value {
        json!({
            "ld_json": {"product": {
                "description": "CTA", "images": ["https://cdn.shopify.com/one.png"],
                "sku": "A121D321", "gtin": "194644098117", "brand": "Anker",
                "offer": {
                    "availability": "in_stock", "price": eur(7998), "item_condition": "new",
                    "url": "https://www.anker.com/eu-en/products/a121d?variant=1&amp;utm_source=google"
                }
            }},
            "canonical_url": "https://www.anker.com/eu-en/products/a121d",
            "product": {
                "id": "gid://shopify/Product/1", "handle": "a121d",
                "title": "Anker <b>641</b> USB-C to Lightning Cable", "description": "Description",
                "key_features": [], "specifications": [], "product_type": null,
                "published_at": "2025-09-04T06:34:16Z", "price": eur(7998),
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
        let result = crate::ankereu::offer_details::v1::structuralize(&input.to_string())
            .expect("the document structuralizes");

        serde_json::from_str(&result.document_json.unwrap()).unwrap()
    }

    fn error(input: Value) -> StructuralizeError {
        let normalized: AnkerEuNormalizedDocument = serde_json::from_value(input).unwrap();

        AnkerEuDocument::from_normalized(normalized).unwrap_err()
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
    fn a_variation_carries_its_title_identifiers_images_price_url_and_offer_facts() {
        let mut input = normalized_document();
        input["product"]["variants"]
            .as_array_mut()
            .unwrap()
            .truncate(1);

        assert_eq!(
            structuralized(&input),
            json!({"variations": [{
                "was_price": null,
                "availability": "in_stock",
                "item_condition": "new",
                "content": {"description": "Description"},
                "identifiers": {
                    "gtin": "194644098117",
                    "sku": "A121D321",
                    "shopify_product_gid": "gid://shopify/Product/1",
                    "shopify_product_variation_gid": "gid://shopify/ProductVariant/1",
                    "shopify_handle": "a121d",
                },
                "title": "Anker <b>641</b> USB-C to Lightning Cable",
                "price": eur(7998),
                "url": "https://www.anker.com/eu-en/products/a121d",
                "images": ["https://cdn.shopify.com/one.png"],
            }]})
        );
    }

    #[test]
    fn each_variation_of_a_multi_variant_document_gets_its_own_url() {
        assert_eq!(
            urls(&structuralized(&normalized_document())),
            [
                "https://www.anker.com/eu-en/products/a121d?variant=1",
                "https://www.anker.com/eu-en/products/a121d?variant=2"
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
            ["https://www.anker.com/eu-en/products/a121d"]
        );
    }

    #[test]
    fn the_islands_arrival_url_reaches_nothing() {
        let document =
            crate::ankereu::offer_details::v1::structuralize(&normalized_document().to_string())
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
    fn an_unusable_identifier_crosses_as_the_identifier_failure() {
        let mut input = normalized_document();
        input["product"]["variants"][0]["sku"] = json!("");

        assert_eq!(
            error(input),
            StructuralizeError::Identifier(IdentifierError::EmptySku)
        );
    }

    #[test]
    fn island_facts_apply_only_to_the_unique_matching_variant() {
        let mut input = normalized_document();
        input["ld_json"]["product"]["offer"]["availability"] = json!("sold_out");
        input["ld_json"]["product"]["offer"]["item_condition"] =
            json!({"other":"https://schema.org/UsedCondition"});
        let output = structuralized(&input);
        assert_eq!(output["variations"][0]["availability"], "sold_out");
        assert_eq!(
            output["variations"][0]["item_condition"],
            json!({"other":"https://schema.org/UsedCondition"})
        );
        assert_eq!(output["variations"][1]["availability"], "available");
        assert!(output["variations"][1]["item_condition"].is_null());
        input["product"]["variants"][1]["sku"] = input["product"]["variants"][0]["sku"].clone();
        for variant in structuralized(&input)["variations"].as_array().unwrap() {
            assert_eq!(variant["availability"], "available");
            assert!(variant["item_condition"].is_null());
        }
    }

    #[test]
    fn comparison_is_per_variant_and_description_uses_only_the_primary_record() {
        let mut input = normalized_document();
        input["product"]["variants"][0]["compare_at_price"] = eur(9999);
        input["product"]["variants"][1]["compare_at_price"] = eur(7998);
        input["product"]["description"] = json!(" ");
        let output = structuralized(&input);
        assert_eq!(output["variations"][0]["was_price"], eur(9999));
        assert!(output["variations"][1]["was_price"].is_null());
        assert_eq!(output["variations"][0]["price"], eur(7998));
        assert_eq!(
            output["variations"][0]["content"],
            json!({"description":" "})
        );
    }
}
