//! Turns normalized data into the product variations the rest of the system consumes: one
//! entry per variation, carrying its title, identifiers, images, price and URL, under the
//! locale of the page they were read from.

use super::normalized::{UgreenEuNormalizedDocument, product};
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
pub struct UgreenEuDocument {
    /// The page's locale prefix, such as `de-at`; `None` on the unprefixed root. One shop
    /// serves every locale, so the same variation appears once per locale page, each with its
    /// own URL and translated title. Carried here so those documents stay apart.
    pub locale: Option<String>,
    pub variations: Vec<ProductVariation>,
}

impl UgreenEuDocument {
    pub fn from_normalized(
        input: UgreenEuNormalizedDocument,
    ) -> Result<UgreenEuDocument, StructuralizeError> {
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

        Ok(UgreenEuDocument {
            locale: product.locale,
            variations,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ProductVariation {
    pub identifiers: ProductVariationIdentifier,
    /// The product's title in the page's language, shared by every variation.
    pub title: String,
    /// The price customers see: after the shop's automatic discount on the selected variation
    /// where one applies, the Shopify price otherwise. `None` where Shopify states 0 (no page
    /// so far).
    pub price: Option<Money>,
    pub url: String,
    /// The product's gallery images.
    pub images: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ProductVariationIdentifier {
    /// `None` where the ld+json states no GTIN: on every variant of 2,875 pages and some
    /// variants of 389 others.
    pub gtin: Option<Gtin>,
    /// A SKU on every saved variant; `None` where one is empty.
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
    // The ld+json offer URL is not used: the page's own product URL, under its locale prefix,
    // is the canonical one.
    if variations < 2 {
        return Ok(product_url.to_owned());
    }

    Ok(format!("{}?variant={}", product_url, gid.numeric_id()?))
}

#[cfg(test)]
mod tests {
    use super::{StructuralizeError, UgreenEuDocument};
    use crate::ugreeneu::offer_details::v1::normalized::UgreenEuNormalizedDocument;
    use crate::ugreeneu::offer_details::v1::raw::UgreenEuRawDocument;
    use crate::ugreeneu::offer_details::v1::raw::tests::document;
    use html_extractor::serde_json::{self, Value, json};
    use retailer_sauce::identifier::IdentifierError;

    /// The raw fixture, normalized, as the JSON structuralize reads.
    fn normalized_document() -> Value {
        let raw: UgreenEuRawDocument =
            serde_json::from_value(document()).expect("the raw fixture deserializes");
        let normalized =
            UgreenEuNormalizedDocument::from_raw(raw).expect("the raw fixture normalizes");
        serde_json::to_value(normalized).expect("serializes")
    }

    fn structuralize(input: Value) -> Result<UgreenEuDocument, StructuralizeError> {
        let normalized: UgreenEuNormalizedDocument =
            serde_json::from_value(input).expect("the normalized fixture deserializes");
        UgreenEuDocument::from_normalized(normalized)
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

    #[test]
    fn each_variation_carries_its_title_identifiers_images_price_and_url_only() {
        let images = json!([
            "https://eu.ugreen.com/cdn/shop/files/ugreen-wireless-powerbank-10000mah-magnetische-power-bank-451511.png?v=1719994480",
            "https://eu.ugreen.com/cdn/shop/products/ugreen-wireless-powerbank-10000mah-magnetische-power-bank-685866.png?v=1719994480"
        ]);
        let product_url = "https://eu.ugreen.com/de-at/products/pb561";
        let title = "UGREEN Kabelloses Magnetisches Powerbank 10000mAh";

        assert_eq!(
            structuralized(normalized_document()),
            json!({
                "locale": "de-at",
                "variations": [
                    {
                        "identifiers": {
                            "gtin": "6941876229192",
                            "sku": "25919",
                            "shopify_product_gid": "gid://shopify/Product/8625471816028",
                            "shopify_product_variation_gid": "gid://shopify/ProductVariant/48981450031452",
                            "shopify_handle": "pb561",
                        },
                        "title": title,
                        "price": {"amount": 5599, "currency": "EUR"},
                        "url": format!("{product_url}?variant=48981450031452"),
                        "images": images,
                    },
                    {
                        "identifiers": {
                            "gtin": null,
                            "sku": "25208",
                            "shopify_product_gid": "gid://shopify/Product/8625471816028",
                            "shopify_product_variation_gid": "gid://shopify/ProductVariant/48981449998684",
                            "shopify_handle": "pb561",
                        },
                        "title": title,
                        "price": {"amount": 4999, "currency": "EUR"},
                        "url": format!("{product_url}?variant=48981449998684"),
                        "images": images,
                    },
                ]
            })
        );
    }

    #[test]
    fn a_single_variation_takes_the_product_url() {
        let output = structuralized(single_variant(normalized_document()));

        assert_eq!(
            output["variations"][0]["url"],
            json!("https://eu.ugreen.com/de-at/products/pb561")
        );
    }

    #[test]
    fn the_root_page_has_no_locale() {
        let mut input = normalized_document();
        input["product"]["locale"] = Value::Null;
        input["product"]["url"] = json!("https://eu.ugreen.com/products/pb561");

        let output = structuralized(input);
        assert_eq!(output["locale"], Value::Null);
        assert_eq!(
            output["variations"][1]["url"],
            json!("https://eu.ugreen.com/products/pb561?variant=48981449998684")
        );
    }

    #[test]
    fn a_discounted_variation_carries_the_price_customers_see() {
        let mut input = normalized_document();
        input["product"]["variants"][0]["price"] = json!({"amount": 4199, "currency": "EUR"});
        input["product"]["variants"][0]["automatic_discount"] = json!(true);

        let output = structuralized(input);
        assert_eq!(
            output["variations"][0]["price"],
            json!({"amount": 4199, "currency": "EUR"})
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
            json!("gid://shopify/ProductVariant/48981449998684")
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
