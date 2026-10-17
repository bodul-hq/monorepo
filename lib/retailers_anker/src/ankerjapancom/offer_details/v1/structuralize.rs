//! Turns normalized data into the product variations the rest of the system consumes: one
//! entry per variation, carrying its title, identifiers, images, price and URL.
//!
//! The storefront renders the offer twice - as a schema.org island and as Liquid's product record -
//! and normalization keeps both. This is where they become one variation per variant, each
//! field taken from whichever rendering states it at the right scope.

use super::normalized::{AnkerJapanComNormalizedDocument, record};
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
pub struct AnkerJapanComDocument {
    pub variations: Vec<ProductVariation>,
}

impl AnkerJapanComDocument {
    pub fn from_normalized(
        input: AnkerJapanComNormalizedDocument,
    ) -> Result<AnkerJapanComDocument, StructuralizeError> {
        let AnkerJapanComNormalizedDocument { ld_json, product } = input;
        // The older PowerBag pages omit Product JSON-LD; their handle still identifies the
        // product on this storefront. Do not invent an island or drop the Liquid variants.
        let product_url = ld_json
            .product
            .map(|p| p.url)
            .unwrap_or_else(|| format!("https://www.ankerjapan.com/products/{}", product.handle));

        let variations_count = product.variants.len();
        let mut variations = Vec::with_capacity(variations_count);

        for variant in &product.variants {
            let identifiers = ProductVariationIdentifier::from_normalized(&product, variant)?;
            let url = variation_url(
                &product_url,
                &identifiers.shopify_product_variation_gid,
                variations_count,
            )?;

            variations.push(ProductVariation {
                identifiers,
                title: product.title.clone(),
                price: variant.price.clone(),
                url,
                images: product.images.clone(),
            });
        }

        Ok(AnkerJapanComDocument { variations })
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ProductVariation {
    pub identifiers: ProductVariationIdentifier,
    /// The record's, shared by every variation: the island's `name` prefixes it with `【公式】`
    /// ("official").
    pub title: String,
    /// The variation's own price. `None` where the storefront does not sell the variant and
    /// renders ¥0 instead.
    pub price: Option<Money>,
    pub url: String,
    /// The product's images, from the record.
    pub images: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ProductVariationIdentifier {
    /// The barcode field verbatim, which is not always a GTIN: some variants carry their handle
    /// or SKU there.
    pub gtin: Option<Gtin>,
    /// `None` where the storefront renders an empty SKU.
    pub sku: Option<Sku>,
    pub shopify_product_gid: ShopifyProductGid,
    pub shopify_product_variation_gid: ShopifyProductVariationGid,
    pub shopify_handle: ShopifyHandle,
}

impl ProductVariationIdentifier {
    fn from_normalized(
        product: &record::ProductRecord,
        variant: &record::Variant,
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

// The island's offer URLs pin `?variant=` even on a product with one variant; the bare product URL
// is the canonical.
fn variation_url(
    product_url: &str,
    gid: &ShopifyProductVariationGid,
    variations: usize,
) -> Result<String, StructuralizeError> {
    if variations < 2 {
        return Ok(product_url.to_owned());
    }

    Ok(format!("{}?variant={}", product_url, gid.numeric_id()?))
}

#[cfg(test)]
mod tests {
    use super::{AnkerJapanComDocument, StructuralizeError};
    use crate::ankerjapancom::offer_details::v1::normalized::AnkerJapanComNormalizedDocument;
    use crate::shared::IdentifierError;
    use html_extractor::serde_json::{self, Value, json};

    fn yen(amount: i64) -> Value {
        json!({"amount": amount, "currency": "JPY"})
    }

    fn variant(id: u64, sku: &str, label: &str, available: bool) -> Value {
        json!({
            "id": format!("gid://shopify/ProductVariant/{id}"), "sku": sku,
            "barcode": "4571411181795", "name": format!("Anker Hub - {label}"),
            "price": yen(6199), "compare_at_price": null, "available": available,
            "requires_shipping": true, "weight": null,
            "image": format!("https://www.ankerjapan.com/cdn/shop/products/{sku}.jpg"),
            "options": [{"name": "カラー", "value": label}]
        })
    }

    fn normalized_document() -> Value {
        json!({
            "ld_json": {
                "product": {
                    "description": "CTA", "url": "https://www.ankerjapan.com/products/hub",
                    "brand": "Anker", "currency": "JPY",
                    "offers": [], "aggregate_rating": null, "reviews": null
                },
                "organization": null, "breadcrumb_list": null
            },
            "product": {
                "id": "gid://shopify/Product/1", "handle": "hub", "title": "Anker Hub",
                "description": "<p>Description</p>",
                "published_at": "2021-06-07T00:28:57Z", "created_at": "2021-06-07T00:28:57Z",
                "product_type": "Hub", "tags": [], "options": ["カラー"],
                "images": ["https://www.ankerjapan.com/cdn/shop/products/one.jpg"],
                "variants": [
                    variant(40082645057697, "A1", "ブラック", true),
                    variant(40082645057698, "A2", "ホワイト", false)
                ]
            }
        })
    }

    fn document(input: &Value) -> Result<AnkerJapanComDocument, StructuralizeError> {
        let normalized: AnkerJapanComNormalizedDocument =
            serde_json::from_value(input.clone()).expect("the normalized fixture deserializes");
        AnkerJapanComDocument::from_normalized(normalized)
    }

    fn structuralized(input: &Value) -> Value {
        serde_json::to_value(document(input).expect("the fixture structuralizes"))
            .expect("serializes")
    }

    fn variations(output: &Value) -> &Vec<Value> {
        output["variations"].as_array().expect("variations")
    }

    fn single_variant(input: &mut Value) {
        input["product"]["variants"]
            .as_array_mut()
            .expect("variants")
            .truncate(1);
    }

    #[test]
    fn a_variation_carries_its_title_identifiers_images_price_and_url_only() {
        let mut input = normalized_document();
        single_variant(&mut input);

        assert_eq!(
            structuralized(&input),
            json!({"variations": [{
                "identifiers": {
                    "gtin": "4571411181795",
                    "sku": "A1",
                    "shopify_product_gid": "gid://shopify/Product/1",
                    "shopify_product_variation_gid": "gid://shopify/ProductVariant/40082645057697",
                    "shopify_handle": "hub",
                },
                "title": "Anker Hub",
                "price": yen(6199),
                "url": "https://www.ankerjapan.com/products/hub",
                "images": ["https://www.ankerjapan.com/cdn/shop/products/one.jpg"],
            }]})
        );
    }

    #[test]
    fn each_variation_of_a_multi_variant_document_gets_its_own_url() {
        let output = structuralized(&normalized_document());
        let urls: Vec<_> = variations(&output)
            .iter()
            .map(|variation| variation["url"].clone())
            .collect();

        assert_eq!(
            urls,
            [
                json!("https://www.ankerjapan.com/products/hub?variant=40082645057697"),
                json!("https://www.ankerjapan.com/products/hub?variant=40082645057698")
            ]
        );
    }

    #[test]
    fn a_single_variant_document_gets_the_bare_product_url() {
        let mut input = normalized_document();
        single_variant(&mut input);

        let output = structuralized(&input);
        assert_eq!(
            variations(&output)[0]["url"],
            json!("https://www.ankerjapan.com/products/hub")
        );
    }

    #[test]
    fn a_product_without_json_ld_uses_its_liquid_handle_for_the_url() {
        let mut input = normalized_document();
        input["ld_json"]["product"] = Value::Null;
        let output = serde_json::to_value(document(&input).unwrap()).unwrap();
        assert_eq!(
            output["variations"][0]["url"],
            "https://www.ankerjapan.com/products/hub?variant=40082645057697"
        );
        single_variant(&mut input);
        let output = serde_json::to_value(document(&input).unwrap()).unwrap();
        assert_eq!(
            output["variations"][0]["url"],
            "https://www.ankerjapan.com/products/hub"
        );
    }

    #[test]
    fn the_title_is_the_records_without_the_islands_prefix() {
        let output = structuralized(&normalized_document());

        assert_eq!(variations(&output)[0]["title"], json!("Anker Hub"));
    }

    #[test]
    fn the_images_are_the_records_not_the_variants_own() {
        let output = structuralized(&normalized_document());

        for variation in variations(&output) {
            assert_eq!(
                variation["images"],
                json!(["https://www.ankerjapan.com/cdn/shop/products/one.jpg"])
            );
        }
    }

    #[test]
    fn a_variant_the_storefront_does_not_sell_has_no_price_and_its_siblings_keep_theirs() {
        let mut input = normalized_document();
        input["product"]["variants"][1]["price"] = Value::Null;

        let output = structuralized(&input);
        assert_eq!(variations(&output)[0]["price"], yen(6199));
        assert_eq!(variations(&output)[1]["price"], Value::Null);
    }

    #[test]
    fn a_variant_without_a_sku_still_crosses() {
        let mut input = normalized_document();
        input["product"]["variants"][0]["sku"] = Value::Null;

        let output = structuralized(&input);
        assert_eq!(variations(&output)[0]["identifiers"]["sku"], Value::Null);
    }

    #[test]
    fn a_junk_barcode_crosses_as_the_gtin_verbatim() {
        let mut input = normalized_document();
        input["product"]["variants"][0]["barcode"] = json!("a1233,");

        let output = structuralized(&input);
        assert_eq!(
            variations(&output)[0]["identifiers"]["gtin"],
            json!("a1233,")
        );
    }

    #[test]
    fn an_unusable_identifier_crosses_as_the_identifier_failure() {
        let mut input = normalized_document();
        input["product"]["handle"] = json!("");

        assert_eq!(
            document(&input),
            Err(StructuralizeError::Identifier(
                IdentifierError::EmptyShopifyHandle
            ))
        );
    }

    #[test]
    fn a_gid_without_a_numeric_tail_fails_a_multi_variant_document() {
        let mut input = normalized_document();
        input["product"]["variants"][0]["id"] = json!("gid://shopify/ProductVariant/x");

        assert!(matches!(
            document(&input),
            Err(StructuralizeError::Identifier(
                IdentifierError::VariationGidWithoutNumericId { .. }
            ))
        ));
    }
}
