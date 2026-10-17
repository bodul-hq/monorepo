//! Turns normalized data into the product variations the rest of the system consumes: one
//! entry per variation, carrying its title, identifiers, images, price and URL.
//!
//! The storefront renders the offer twice - as a schema.org island and as its own product record -
//! and normalization keeps both. This is where they become one variation per variant, each
//! field taken from whichever rendering states it at the right scope.

use super::normalized::{self, AnkerVnNormalizedDocument};
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
pub struct AnkerVnDocument {
    pub variations: Vec<ProductVariation>,
}

impl AnkerVnDocument {
    pub fn from_normalized(
        input: AnkerVnNormalizedDocument,
    ) -> Result<AnkerVnDocument, StructuralizeError> {
        let AnkerVnNormalizedDocument {
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
                identifiers,
                title: product.title.clone(),
                price: variant.price.clone(),
                url,
                images: island.images.clone(),
            });
        }

        Ok(AnkerVnDocument { variations })
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ProductVariation {
    pub identifiers: ProductVariationIdentifier,
    /// The record's, verbatim and shared by every variation: some titles wrap the model number in
    /// `<b>` markup.
    pub title: String,
    /// The variation's own price, as the page states it, including the few amounts of tens of dong
    /// that sit beside siblings priced in hundreds of thousands.
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

// The island's `offer.url` is the URL the crawler arrived through, escaped and carrying a pinned
// variant; the page's canonical is clean.
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
    use super::{AnkerVnDocument, StructuralizeError};
    use crate::ankervn::offer_details::v1::normalized::AnkerVnNormalizedDocument;
    use crate::shared::IdentifierError;
    use html_extractor::serde_json::{self, Value, json};

    fn vnd(amount: i64) -> Value {
        json!({"amount": amount, "currency": "VND"})
    }

    fn variant(id: u64, sku: &str, label: &str, price: i64) -> Value {
        json!({
            "id": format!("gid://shopify/ProductVariant/{id}"), "sku": sku,
            "barcode": "194644098117", "name": label, "price": vnd(price),
            "compare_at_price": null, "quantity_available": 10, "available_for_sale": true,
            "currently_not_in_stock": false, "requires_shipping": true, "weight": 0.2,
            "image": {"url": format!("https://cdn/{sku}.png"), "alt_text": null, "width": 1, "height": 1},
            "options": [{"name": "Màu sắc", "display_name": "màu sắc", "values": [{"label": label}]}]
        })
    }

    fn normalized_document() -> Value {
        json!({
            "ld_json": {"product": {
                "description": "cta", "images": ["https://cdn/1.png"], "sku": "A2331111",
                "gtin": "194644098117", "brand": "Anker",
                "offer": {
                    "availability": "in_stock", "price": vnd(20),
                    "item_condition": "new",
                    "url": "https://www.anker.com/vn/products/a2331?variant=1&amp;ref=x"
                }
            }},
            "canonical_url": "https://www.anker.com/vn/products/a2331",
            "product": {
                "id": "gid://shopify/Product/9", "handle": "a2331",
                "title": "Sạc Anker <b>323</b> (33W)", "description": "d",
                "specifications": [{"name": "Công suất", "value": "33W"}],
                "product_type": "charging only", "published_at": "2025-09-04T06:34:16Z",
                "price": vnd(20), "compare_at_price": null, "available_for_sale": true,
                "options": [{"id": "gid://shopify/ProductOption/7", "name": "Màu sắc",
                    "display_name": "màu sắc", "values": [{"label": "Đen"}, {"label": "Trắng"}]}],
                "variants": [variant(11, "A2331111", "Đen", 20), variant(12, "A2331121", "Trắng", 380000)]
            }
        })
    }

    fn structuralized(input: &Value) -> Value {
        let normalized: AnkerVnNormalizedDocument = serde_json::from_value(input.clone()).unwrap();

        serde_json::to_value(AnkerVnDocument::from_normalized(normalized).unwrap()).unwrap()
    }

    fn error(input: Value) -> StructuralizeError {
        let normalized: AnkerVnNormalizedDocument = serde_json::from_value(input).unwrap();

        AnkerVnDocument::from_normalized(normalized).unwrap_err()
    }

    fn variations(output: &Value) -> &Vec<Value> {
        output["variations"].as_array().unwrap()
    }

    #[test]
    fn a_variation_carries_its_title_identifiers_images_price_and_url_only() {
        let mut input = normalized_document();
        input["product"]["variants"] = json!([variant(11, "A2331111", "Đen", 380000)]);

        assert_eq!(
            structuralized(&input),
            json!({"variations": [{
                "identifiers": {
                    "gtin": "194644098117",
                    "sku": "A2331111",
                    "shopify_product_gid": "gid://shopify/Product/9",
                    "shopify_product_variation_gid": "gid://shopify/ProductVariant/11",
                    "shopify_handle": "a2331",
                },
                "title": "Sạc Anker <b>323</b> (33W)",
                "price": vnd(380000),
                "url": "https://www.anker.com/vn/products/a2331",
                "images": ["https://cdn/1.png"],
            }]})
        );
    }

    #[test]
    fn each_variation_of_a_multi_variant_document_gets_its_own_url() {
        let output = structuralized(&normalized_document());

        assert_eq!(
            variations(&output)[0]["url"],
            "https://www.anker.com/vn/products/a2331?variant=11"
        );
        assert_eq!(
            variations(&output)[1]["url"],
            "https://www.anker.com/vn/products/a2331?variant=12"
        );
    }

    #[test]
    fn a_single_variant_document_gets_the_bare_canonical() {
        let mut input = normalized_document();
        input["product"]["variants"] = json!([variant(11, "A2331111", "Đen", 380000)]);

        let output = structuralized(&input);

        assert_eq!(
            variations(&output)[0]["url"],
            "https://www.anker.com/vn/products/a2331"
        );
    }

    #[test]
    fn the_islands_arrival_url_reaches_nothing() {
        let output = structuralized(&normalized_document());

        assert!(!output.to_string().contains("ref=x"));
    }

    #[test]
    fn the_title_crosses_with_the_records_markup() {
        let output = structuralized(&normalized_document());

        assert_eq!(
            variations(&output)[0]["title"],
            "Sạc Anker <b>323</b> (33W)"
        );
    }

    #[test]
    fn the_images_are_the_islands_not_the_variants_own() {
        let output = structuralized(&normalized_document());

        for variation in variations(&output) {
            assert_eq!(variation["images"], json!(["https://cdn/1.png"]));
        }
    }

    #[test]
    fn a_low_price_crosses_verbatim_beside_its_siblings_real_one() {
        let output = structuralized(&normalized_document());

        assert_eq!(variations(&output)[0]["price"], vnd(20));
        assert_eq!(variations(&output)[1]["price"], vnd(380000));
    }

    #[test]
    fn a_variant_without_a_barcode_crosses_without_a_gtin() {
        let mut input = normalized_document();
        input["product"]["variants"][0]["barcode"] = Value::Null;

        let output = structuralized(&input);

        assert_eq!(variations(&output)[0]["identifiers"]["gtin"], Value::Null);
        assert_eq!(
            variations(&output)[1]["identifiers"]["gtin"],
            "194644098117"
        );
    }

    #[test]
    fn an_unusable_identifier_crosses_as_the_identifier_failure() {
        let mut input = normalized_document();
        input["product"]["variants"][0]["sku"] = json!("");

        assert!(matches!(
            error(input),
            StructuralizeError::Identifier(IdentifierError::EmptySku)
        ));
    }
}
