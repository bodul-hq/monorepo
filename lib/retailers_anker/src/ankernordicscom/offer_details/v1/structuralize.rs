//! Turns normalized data into the product variations the rest of the system consumes: one
//! entry per variation, carrying its title, identifiers, images, price and URL.
//!
//! On this storefront the product record states nearly everything and the schema.org island only
//! its gallery, so each variation is the record's product facts joined to one of its variants.

use super::normalized::{self, AnkerNordicsComNormalizedDocument};
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
pub struct AnkerNordicsComDocument {
    pub variations: Vec<ProductVariation>,
}

impl AnkerNordicsComDocument {
    pub fn from_normalized(
        input: AnkerNordicsComNormalizedDocument,
    ) -> Result<AnkerNordicsComDocument, StructuralizeError> {
        let AnkerNordicsComNormalizedDocument {
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

        Ok(AnkerNordicsComDocument { variations })
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ProductVariation {
    pub identifiers: ProductVariationIdentifier,
    /// The record's, verbatim and shared by every variation: some titles wrap the model number in
    /// `<b>` markup.
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
    if variations < 2 {
        return Ok(canonical_url.to_owned());
    }

    Ok(format!("{}?variant={}", canonical_url, gid.numeric_id()?))
}

#[cfg(test)]
mod tests {
    use super::{AnkerNordicsComDocument, StructuralizeError};
    use crate::ankernordicscom::offer_details::v1::normalized::AnkerNordicsComNormalizedDocument;
    use crate::shared::IdentifierError;
    use html_extractor::serde_json::{self, Value, json};

    fn sek(amount: i64) -> Value {
        json!({"amount": amount * 100, "currency": "SEK"})
    }

    fn variant(id: u64, sku: &str, label: &str) -> Value {
        json!({
            "id": format!("gid://shopify/ProductVariant/{id}"), "sku": sku,
            "barcode": "194644239299", "price": sek(2490), "compare_at_price": null,
            "quantity_available": 3, "available_for_sale": true, "currently_not_in_stock": false,
            "image": {"url": format!("https://cdn/{sku}.png"), "alt_text": null, "width": 1, "height": 1},
            "options": [{"name": "Color", "display_name": "color",
                "values": [{"label": label, "hex_colors": ["#000000"]}]}],
            "discounts": []
        })
    }

    fn normalized_document() -> Value {
        json!({
            "ld_json": {"product": {"images": ["https://cdn/1.png", "https://cdn/2.png"]}},
            "canonical_url": "https://www.ankernordics.com/products/a3947",
            "product": {
                "id": "gid://shopify/Product/9", "handle": "a3947",
                "title": "Anker <b>315</b> Wireless Charger (Pad)", "description": "d",
                "vendor": "soundcoreeu",
                "key_features": [{"title": "t", "body": "b"}],
                "specifications": [{"name": "Weight", "value": "600 g"}],
                "product_type": null, "updated_at": "2026-09-01T10:00:00Z",
                "price": sek(2490), "compare_at_price": null, "available_for_sale": true,
                "options": [{"id": "gid://shopify/ProductOption/7", "name": "color", "values": [
                    {"label": "Black", "hex_colors": ["#000000"]},
                    {"label": "White", "hex_colors": ["#FFFFFF"]}
                ]}],
                "variants": [variant(11, "A3947G11", "Black"), variant(12, "A3947G11-F0", "White")]
            }
        })
    }

    fn structuralized(input: &Value) -> Value {
        let normalized: AnkerNordicsComNormalizedDocument =
            serde_json::from_value(input.clone()).unwrap();

        serde_json::to_value(AnkerNordicsComDocument::from_normalized(normalized).unwrap()).unwrap()
    }

    fn error(input: Value) -> StructuralizeError {
        let normalized: AnkerNordicsComNormalizedDocument = serde_json::from_value(input).unwrap();

        AnkerNordicsComDocument::from_normalized(normalized).unwrap_err()
    }

    fn variations(output: &Value) -> &Vec<Value> {
        output["variations"].as_array().unwrap()
    }

    fn single_variant_document() -> Value {
        let mut input = normalized_document();
        input["product"]["variants"]
            .as_array_mut()
            .unwrap()
            .truncate(1);
        input
    }

    #[test]
    fn a_variation_carries_its_title_identifiers_images_price_and_url_only() {
        assert_eq!(
            structuralized(&single_variant_document()),
            json!({"variations": [{
                "identifiers": {
                    "gtin": "194644239299",
                    "sku": "A3947G11",
                    "shopify_product_gid": "gid://shopify/Product/9",
                    "shopify_product_variation_gid": "gid://shopify/ProductVariant/11",
                    "shopify_handle": "a3947",
                },
                "title": "Anker <b>315</b> Wireless Charger (Pad)",
                "price": sek(2490),
                "url": "https://www.ankernordics.com/products/a3947",
                "images": ["https://cdn/1.png", "https://cdn/2.png"],
            }]})
        );
    }

    #[test]
    fn each_variation_of_a_multi_variant_document_gets_its_own_url() {
        let output = structuralized(&normalized_document());

        assert_eq!(
            variations(&output)[0]["url"],
            "https://www.ankernordics.com/products/a3947?variant=11"
        );
        assert_eq!(
            variations(&output)[1]["url"],
            "https://www.ankernordics.com/products/a3947?variant=12"
        );
    }

    #[test]
    fn a_single_variant_document_gets_the_bare_canonical() {
        let output = structuralized(&single_variant_document());

        assert_eq!(
            variations(&output)[0]["url"],
            "https://www.ankernordics.com/products/a3947"
        );
    }

    #[test]
    fn the_title_crosses_with_the_records_markup() {
        let output = structuralized(&normalized_document());

        assert_eq!(
            variations(&output)[0]["title"],
            "Anker <b>315</b> Wireless Charger (Pad)"
        );
    }

    #[test]
    fn the_images_are_the_islands_not_the_variants_own() {
        let output = structuralized(&normalized_document());

        assert_eq!(
            variations(&output)[1]["images"],
            json!(["https://cdn/1.png", "https://cdn/2.png"])
        );
    }

    #[test]
    fn the_price_is_the_variants_own_whatever_its_discounts() {
        let mut input = normalized_document();
        input["product"]["variants"][0]["price"] = sek(29990);
        input["product"]["variants"][0]["discounts"] = json!([{
            "code": "WS24PY31CNNH", "kind": "WS24", "value": {"Percentage": -33.0},
            "price": sek(23443), "saving": sek(11546),
            "starts_at": "2026-05-18T21:59:00Z", "ends_at": "2026-12-31T22:59:00Z",
            "variant_shopify_id": 11, "amazon_deal": false, "sales_channels": null
        }]);

        let output = structuralized(&input);

        assert_eq!(variations(&output)[0]["price"], sek(29990));
        assert_eq!(variations(&output)[1]["price"], sek(2490));
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

    #[test]
    fn a_gid_without_a_numeric_tail_fails_a_multi_variant_document() {
        let mut input = normalized_document();
        input["product"]["variants"][0]["id"] = json!("gid://shopify/ProductVariant/x");

        assert!(matches!(
            error(input),
            StructuralizeError::Identifier(IdentifierError::VariationGidWithoutNumericId { .. })
        ));
    }
}
