//! Turns normalized data into the product variations the rest of the system consumes: one
//! entry per variation, carrying its title, identifiers, images, price and URL.
//!
//! The storefront renders the offer twice - as a schema.org island and as its own product record -
//! and normalization keeps both. This is where they become one variation per variant, each
//! field taken from whichever rendering states it at the right scope.

use super::normalized::{self, AnkerNzNormalizedDocument};
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
pub struct AnkerNzDocument {
    pub variations: Vec<ProductVariation>,
}

impl AnkerNzDocument {
    pub fn from_normalized(
        input: AnkerNzNormalizedDocument,
    ) -> Result<AnkerNzDocument, StructuralizeError> {
        let AnkerNzNormalizedDocument {
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

        Ok(AnkerNzDocument { variations })
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ProductVariation {
    pub identifiers: ProductVariationIdentifier,
    /// The record's, verbatim and shared by every variation: some titles wrap the model number in
    /// `<b>` markup.
    pub title: String,
    /// Labelled `USD` as the page states it, though the amounts are the AU store's AUD prices.
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
    use super::{AnkerNzDocument, StructuralizeError};
    use crate::ankernz::offer_details::v1::normalized::AnkerNzNormalizedDocument;
    use crate::shared::IdentifierError;
    use html_extractor::serde_json::{self, Value, json};

    fn usd(amount: i64) -> Value {
        json!({"amount": amount, "currency": "USD"})
    }

    fn variant(id: u64, sku: &str, label: &str) -> Value {
        json!({
            "id": format!("gid://shopify/ProductVariant/{id}"), "sku": sku,
            "barcode": "194644098117", "name": label, "price": usd(9995),
            "compare_at_price": null, "quantity_available": 0, "available_for_sale": false,
            "currently_not_in_stock": true, "requires_shipping": true, "weight": 0.2127,
            "image": {"url": format!("https://cdn.shopify.com/{sku}.png"), "width": 1,
                      "height": 1},
            "options": [{"name": "Color", "display_name": "color",
                         "values": [{"label": label, "hex_colors": null}]}]
        })
    }

    fn normalized_document() -> Value {
        json!({
            "ld_json": {"product": {
                "description": "CTA", "images": ["https://cdn.shopify.com/one.png"],
                "sku": "A1259011", "gtin": "194644098117", "brand": "Anker",
                "offer": {
                    "availability": "sold_out", "price": usd(9995), "item_condition": "new",
                    "url": "https://www.anker.com/nz/products/a1259?variant=1&amp;ref=collection"
                }
            }},
            "canonical_url": "https://www.anker.com/nz/products/a1259",
            "product": {
                "id": "gid://shopify/Product/1", "handle": "a1259",
                "title": "Anker <b>334</b> MagGo Battery", "description": "Description",
                "specifications": [], "product_type": null,
                "published_at": "2025-09-04T06:34:16Z", "price": usd(9995),
                "compare_at_price": null, "available_for_sale": false,
                "options": [{"id": "gid://shopify/ProductOption/7", "name": "Color",
                             "display_name": "color",
                             "values": [{"label": "Black", "hex_colors": null},
                                        {"label": "White", "hex_colors": null}]}],
                "variants": [variant(1, "A1259011", "Black"), variant(2, "A1259021", "White")]
            }
        })
    }

    fn structuralized(input: &Value) -> Value {
        let result = crate::ankernz::offer_details::v1::structuralize(&input.to_string())
            .expect("the document structuralizes");

        serde_json::from_str(&result.document_json.unwrap()).unwrap()
    }

    fn error(input: Value) -> StructuralizeError {
        let normalized: AnkerNzNormalizedDocument = serde_json::from_value(input).unwrap();

        AnkerNzDocument::from_normalized(normalized).unwrap_err()
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
                "title": "Anker <b>334</b> MagGo Battery",
                "identifiers": {
                    "gtin": "194644098117",
                    "sku": "A1259011",
                    "shopify_product_gid": "gid://shopify/Product/1",
                    "shopify_product_variation_gid": "gid://shopify/ProductVariant/1",
                    "shopify_handle": "a1259",
                },
                "images": ["https://cdn.shopify.com/one.png"],
                "price": usd(9995),
                "url": "https://www.anker.com/nz/products/a1259",
            }]})
        );
    }

    #[test]
    fn each_variation_of_a_multi_variant_document_gets_its_own_url() {
        assert_eq!(
            urls(&structuralized(&normalized_document())),
            [
                "https://www.anker.com/nz/products/a1259?variant=1",
                "https://www.anker.com/nz/products/a1259?variant=2"
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
            ["https://www.anker.com/nz/products/a1259"]
        );
    }

    #[test]
    fn the_islands_arrival_url_reaches_nothing() {
        let document =
            crate::ankernz::offer_details::v1::structuralize(&normalized_document().to_string())
                .unwrap()
                .document_json
                .unwrap();

        for leaked in ["&amp;", "ref=collection", "variant=1&"] {
            assert!(!document.contains(leaked), "{leaked} survived: {document}");
        }
    }

    #[test]
    fn the_title_crosses_with_the_records_markup() {
        assert_eq!(
            structuralized(&normalized_document())["variations"][0]["title"],
            "Anker <b>334</b> MagGo Battery"
        );
    }

    #[test]
    fn the_pages_usd_label_crosses_as_stated() {
        assert_eq!(
            structuralized(&normalized_document())["variations"][0]["price"],
            usd(9995)
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
}
