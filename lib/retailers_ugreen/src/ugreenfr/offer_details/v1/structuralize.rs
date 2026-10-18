//! Turns normalized data into the product variations the rest of the system consumes: one
//! entry per variation, carrying its title, identifiers, images, price and URL.

use super::normalized::{UgreenFrNormalizedDocument, product};
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
pub struct UgreenFrDocument {
    pub variations: Vec<ProductVariation>,
}

impl UgreenFrDocument {
    pub fn from_normalized(
        input: UgreenFrNormalizedDocument,
    ) -> Result<UgreenFrDocument, StructuralizeError> {
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
                    price: Some(variant.price.clone()),
                    url,
                })
            })
            .collect::<Result<_, StructuralizeError>>()?;

        Ok(UgreenFrDocument { variations })
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ProductVariation {
    pub identifiers: ProductVariationIdentifier,
    /// The product's title, shared by every variation.
    pub title: String,
    /// The price customers see: the automatic discount's for the selected variant on the 40
    /// pages that have one, otherwise the Shopify variant price. Every saved variant is priced;
    /// `Option` keeps the family's shape.
    pub price: Option<Money>,
    pub url: String,
    /// The product's images, from the main gallery.
    pub images: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ProductVariationIdentifier {
    /// `None` on 400 of the 598 saved variants, which render no barcode.
    pub gtin: Option<Gtin>,
    /// `None` on the three gift cards; the Shopify gids still identify them.
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
            gtin: variant.gtin.clone().map(Gtin::try_from).transpose()?,
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
    // The ld+json offer URL is not used: the handle's page on the storefront origin is the
    // canonical URL, and `?variant=` selects a variation on it.
    if variations < 2 {
        return Ok(product_url.to_owned());
    }

    Ok(format!("{}?variant={}", product_url, gid.numeric_id()?))
}

#[cfg(test)]
mod tests {
    use super::{StructuralizeError, UgreenFrDocument};
    use crate::ugreenfr::offer_details::v1::normalized::UgreenFrNormalizedDocument;
    use crate::ugreenfr::offer_details::v1::raw::UgreenFrRawDocument;
    use crate::ugreenfr::offer_details::v1::raw::tests::{discounted_document, document};
    use html_extractor::serde_json::{self, Value, json};
    use retailer_sauce::identifier::IdentifierError;

    /// A raw fixture, normalized, as the JSON structuralize reads.
    fn normalized_from(raw: Value) -> Value {
        let raw: UgreenFrRawDocument =
            serde_json::from_value(raw).expect("the raw fixture deserializes");
        let normalized =
            UgreenFrNormalizedDocument::from_raw(raw).expect("the raw fixture normalizes");
        serde_json::to_value(normalized).expect("serializes")
    }

    fn normalized_document() -> Value {
        normalized_from(document())
    }

    fn structuralize(input: Value) -> Result<UgreenFrDocument, StructuralizeError> {
        let normalized: UgreenFrNormalizedDocument =
            serde_json::from_value(input).expect("the normalized fixture deserializes");
        UgreenFrDocument::from_normalized(normalized)
    }

    fn structuralized(input: Value) -> Value {
        serde_json::to_value(structuralize(input).expect("the fixture structuralizes"))
            .expect("serializes")
    }

    #[test]
    fn each_variation_carries_its_title_identifiers_images_price_and_url_only() {
        let images = json!([
            "https://fr.ugreen.com/cdn/shop/files/charger.png?v=1738861172",
            "https://fr.ugreen.com/cdn/shop/files/61kMB5x0AfL.jpg?v=1738834470"
        ]);
        let product_url = "https://fr.ugreen.com/products/45699";

        assert_eq!(
            structuralized(normalized_document()),
            json!({"variations": [
                {
                    "identifiers": {
                        "gtin": "6941876246991",
                        "sku": "45699",
                        "shopify_product_gid": "gid://shopify/Product/9526794977600",
                        "shopify_product_variation_gid": "gid://shopify/ProductVariant/50354801180992",
                        "shopify_handle": "45699",
                    },
                    "title": "UGREEN 300W Nexode Chargeur",
                    "price": {"amount": 13499, "currency": "EUR"},
                    "url": format!("{product_url}?variant=50354801180992"),
                    "images": images,
                },
                {
                    "identifiers": {
                        "gtin": null,
                        "sku": "45699+70427P",
                        "shopify_product_gid": "gid://shopify/Product/9526794977600",
                        "shopify_product_variation_gid": "gid://shopify/ProductVariant/50354801213760",
                        "shopify_handle": "45699",
                    },
                    "title": "UGREEN 300W Nexode Chargeur",
                    "price": {"amount": 18799, "currency": "EUR"},
                    "url": format!("{product_url}?variant=50354801213760"),
                    "images": images,
                },
            ]})
        );
    }

    #[test]
    fn a_single_discounted_variation_takes_the_product_url_and_the_discounted_price() {
        let output = structuralized(normalized_from(discounted_document()));

        assert_eq!(
            output,
            json!({"variations": [{
                "identifiers": {
                    "gtin": "6938638046609",
                    "sku": "15532",
                    "shopify_product_gid": "gid://shopify/Product/8986551157056",
                    "shopify_product_variation_gid": "gid://shopify/ProductVariant/47543757046080",
                    "shopify_handle": "15532",
                },
                "title": "UGREEN Revodok Pro 109 Hub USB C",
                "price": {"amount": 5299, "currency": "EUR"},
                "url": "https://fr.ugreen.com/products/15532",
                "images": ["https://fr.ugreen.com/cdn/shop/files/hub.png?v=1746741214"],
            }]})
        );
    }

    /// As on the gift cards (`products-ugreen-carte-cadeau-*.html`).
    #[test]
    fn a_variation_without_a_sku_is_kept() {
        let mut input = normalized_document();
        input["product"]["variants"][1]["sku"] = Value::Null;

        let output = structuralized(input);
        assert_eq!(output["variations"][1]["identifiers"]["sku"], Value::Null);
        assert_eq!(
            output["variations"][1]["identifiers"]["shopify_product_variation_gid"],
            json!("gid://shopify/ProductVariant/50354801213760")
        );
    }

    #[test]
    fn a_normalized_variant_without_a_price_does_not_deserialize() {
        let mut input = normalized_document();
        input["product"]["variants"][0]["price"] = Value::Null;

        assert!(serde_json::from_value::<UgreenFrNormalizedDocument>(input).is_err());
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

        let mut gtin = normalized_document();
        gtin["product"]["variants"][0]["gtin"] = json!("");
        assert_eq!(
            structuralize(gtin),
            Err(StructuralizeError::Identifier(IdentifierError::EmptyGtin))
        );

        let mut product_gid = normalized_document();
        product_gid["product"]["id"] = json!("");
        assert_eq!(
            structuralize(product_gid),
            Err(StructuralizeError::Identifier(
                IdentifierError::EmptyShopifyProductGid
            ))
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
