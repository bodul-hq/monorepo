//! Turns normalized data into the product variations the rest of the system consumes: one
//! entry per variation, carrying its title, identifiers, images, price and URL, under the page's
//! locale.

use super::normalized::{Locale, UgreenInNormalizedDocument, product};
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
pub struct UgreenInDocument {
    /// `en` or `hi`: the language of the page, and so of every variation's title and URL. The
    /// English and Hindi pages of a product are separate documents with the same identifiers,
    /// prices and images.
    pub locale: Locale,
    pub variations: Vec<ProductVariation>,
}

impl UgreenInDocument {
    pub fn from_normalized(
        input: UgreenInNormalizedDocument,
    ) -> Result<UgreenInDocument, StructuralizeError> {
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
                    price: variant.price.clone(),
                    url,
                })
            })
            .collect::<Result<_, StructuralizeError>>()?;

        Ok(UgreenInDocument {
            locale: input.locale,
            variations,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ProductVariation {
    pub identifiers: ProductVariationIdentifier,
    /// The product's title in the page's locale, shared by every variation.
    pub title: String,
    /// The variation's own price. Set on every saved variant.
    pub price: Option<Money>,
    /// The page's URL in its locale, with the variant selected where there is a choice.
    pub url: String,
    /// The product's images, from Liquid's record.
    pub images: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ProductVariationIdentifier {
    /// `None` where the barcode is null (230 saved variants per locale) or no GTIN (30 per
    /// locale, spreadsheet-mangled); see the normalized variant's `gtin`.
    pub gtin: Option<Gtin>,
    /// Set on every saved variant.
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
    // The schema.org offer URL is not read: the record's handle on the storefront origin, in the
    // page's locale, is the page's URL. `product_url` already carries the `/hi` prefix and the
    // percent-encoding.
    if variations < 2 {
        return Ok(product_url.to_owned());
    }

    Ok(format!("{}?variant={}", product_url, gid.numeric_id()?))
}

#[cfg(test)]
mod tests {
    use super::{StructuralizeError, UgreenInDocument};
    use crate::ugreenin::offer_details::v1::normalized::UgreenInNormalizedDocument;
    use crate::ugreenin::offer_details::v1::raw::UgreenInRawDocument;
    use crate::ugreenin::offer_details::v1::raw::tests::document;
    use html_extractor::serde_json::{self, Value, json};
    use retailer_sauce::identifier::IdentifierError;

    const PRODUCT_URL: &str =
        "https://www.ugreenindia.com/products/ugreen-ergonomic-mouse-pad-black-25245";

    /// The raw fixture, normalized, as the JSON structuralize reads.
    fn normalized_document() -> Value {
        let raw: UgreenInRawDocument =
            serde_json::from_value(document()).expect("the raw fixture deserializes");
        let normalized =
            UgreenInNormalizedDocument::from_raw(raw).expect("the raw fixture normalizes");
        serde_json::to_value(normalized).expect("serializes")
    }

    fn structuralize(input: Value) -> Result<UgreenInDocument, StructuralizeError> {
        let normalized: UgreenInNormalizedDocument =
            serde_json::from_value(input).expect("the normalized fixture deserializes");
        UgreenInDocument::from_normalized(normalized)
    }

    fn structuralized(input: Value) -> Value {
        serde_json::to_value(structuralize(input).expect("the fixture structuralizes"))
            .expect("serializes")
    }

    fn single_variant(mut input: Value) -> Value {
        input["product"]["variants"]
            .as_array_mut()
            .expect("variants")
            .truncate(1);
        input
    }

    fn variation(id: u64, sku: &str, gtin: Option<&str>) -> Value {
        json!({
            "identifiers": {
                "gtin": gtin,
                "sku": sku,
                "shopify_product_gid": "gid://shopify/Product/7702424813667",
                "shopify_product_variation_gid": format!("gid://shopify/ProductVariant/{id}"),
                "shopify_handle": "ugreen-ergonomic-mouse-pad-black-25245",
            },
            "title": "UGREEN LP668 Ergonomic Mouse Pad with Wrist Rest",
            "price": {"amount": 99900, "currency": "INR"},
            "url": format!("{PRODUCT_URL}?variant={id}"),
            "images": ["https://www.ugreenindia.com/cdn/shop/files/LP668-Black.png?v=1763832672"],
        })
    }

    #[test]
    fn each_variation_carries_its_title_identifiers_images_price_and_url_only() {
        assert_eq!(
            structuralized(normalized_document()),
            json!({
                "locale": "en",
                "variations": [
                    variation(43454938677347, "25245", Some("6941876222452")),
                    variation(43742608687203, "25243", None),
                    variation(43742608719971, "25244", None),
                ]
            })
        );
    }

    #[test]
    fn a_single_variation_takes_the_product_url() {
        let output = structuralized(single_variant(normalized_document()));

        assert_eq!(output["variations"][0]["url"], json!(PRODUCT_URL));
    }

    #[test]
    fn a_hindi_document_keeps_its_locale_and_its_urls() {
        let hindi_url =
            "https://www.ugreenindia.com/hi/products/ugreen-ergonomic-mouse-pad-black-25245";
        let mut input = normalized_document();
        input["locale"] = json!("hi");
        input["product"]["url"] = json!(hindi_url);

        let output = structuralized(input);
        assert_eq!(output["locale"], json!("hi"));
        assert_eq!(
            output["variations"][1]["url"],
            json!(format!("{hindi_url}?variant=43742608687203"))
        );
    }

    #[test]
    fn a_variation_without_a_sku_is_kept() {
        let mut input = normalized_document();
        input["product"]["variants"][1]["sku"] = Value::Null;

        let output = structuralized(input);
        assert_eq!(output["variations"][1]["identifiers"]["sku"], Value::Null);
        assert_eq!(
            output["variations"][1]["identifiers"]["shopify_product_variation_gid"],
            json!("gid://shopify/ProductVariant/43742608687203")
        );
    }

    #[test]
    fn an_unpriced_variation_is_kept_without_a_price() {
        let mut input = single_variant(normalized_document());
        input["product"]["variants"][0]["price"] = Value::Null;

        let output = structuralized(input);
        assert_eq!(output["variations"][0]["price"], Value::Null);
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
