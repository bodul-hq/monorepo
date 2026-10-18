//! Expresses the brand site's product facts in the system's vocabulary. Shopify's analytics
//! `meta` crosses as the product and its variant; the page adds the title (`og:title`), the
//! canonical URL, the gallery and the regional stores the "Shop Now" button links to. Where two
//! structures state the same fact, normalization checks that they agree; what nothing needs stays
//! behind as the commented-out exclusion records in `product`.
//!
//! The brand site does not sell, and the analytics price is a placeholder (USD 1,300.00 on 65
//! pages, USD 5.00 on one). The offers keep it anyway, by the user's decision: see
//! `product::Variant::price`.

pub mod product;

use super::raw;
use money::{Currency, Money};
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// The storefront the pages are saved from.
const ORIGIN: &str = "https://www.ugreen.com";

#[derive(Debug, Clone, PartialEq, Error)]
pub enum NormalizeError {
    #[error("shop currency `{currency}` is not supported: {reason}")]
    UnsupportedCurrency { currency: String, reason: String },

    #[error("shop currency `{currency}` has no hundredths, the unit Liquid renders amounts in")]
    CurrencyWithoutHundredths { currency: String },

    #[error("the analytics product gid `{gid}` is not product `{id}`'s")]
    ProductGidMismatch { gid: String, id: u64 },

    #[error("the page is {resource_type} `{resource_id}`, the analytics product `{product}`")]
    PageResourceMismatch {
        resource_type: String,
        resource_id: u64,
        product: u64,
    },

    #[error("the canonical URL `{url}` is not the product's `/products/{handle}`")]
    CanonicalUrlMismatch { url: String, handle: String },

    #[error("variant `{variant_id}` is named `{name}`, but the page title is `{title}`")]
    TitleMismatch {
        variant_id: u64,
        name: String,
        title: String,
    },

    #[error("the Shop Now button's SKU `{sku}` is not among the analytics variants")]
    ShopNowSkuNotListed { sku: String },
}

// Liquid renders every amount in hundredths of the shop currency, so only a currency whose minor
// unit is a hundredth takes them as they are.
fn shop_currency(code: &str) -> Result<Currency, NormalizeError> {
    let currency: Currency =
        code.parse()
            .map_err(|reason| NormalizeError::UnsupportedCurrency {
                currency: code.to_string(),
                reason,
            })?;

    if currency.exponent() != 2 {
        return Err(NormalizeError::CurrencyWithoutHundredths {
            currency: code.to_string(),
        });
    }

    Ok(currency)
}

fn money_from_shopify_cents(cents: i64, currency: Currency) -> Money {
    Money::new(cents, currency)
}

// The analytics record gives bare numeric ids; the family identifies Shopify objects by their
// Admin API gid.
fn product_gid(id: u64) -> String {
    format!("gid://shopify/Product/{id}")
}

fn variant_gid(id: u64) -> String {
    format!("gid://shopify/ProductVariant/{id}")
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UgreenComNormalizedDocument {
    pub product: product::Product,
}

impl UgreenComNormalizedDocument {
    pub fn from_raw(
        raw: raw::UgreenComRawDocument,
    ) -> Result<UgreenComNormalizedDocument, NormalizeError> {
        let currency = shop_currency(&raw.paypal.currency)?;
        // The ld+json islands are the shop's `Organization` only: nothing about the product.

        Ok(UgreenComNormalizedDocument {
            product: product::Product::from_raw(
                raw.analytics.meta,
                product::Page {
                    title: raw.og_title.content,
                    canonical: raw.canonical.href,
                    shop_now: raw.shop_now,
                    media: raw.media,
                },
                currency,
            )?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{NormalizeError, UgreenComNormalizedDocument};
    use crate::ugreencom::offer_details::v1::raw::UgreenComRawDocument;
    use crate::ugreencom::offer_details::v1::raw::tests::document;
    use html_extractor::serde_json::{self, Value, json};

    fn normalize(document: Value) -> Result<UgreenComNormalizedDocument, NormalizeError> {
        let raw: UgreenComRawDocument =
            serde_json::from_value(document).expect("the raw fixture deserializes");
        UgreenComNormalizedDocument::from_raw(raw)
    }

    fn normalized(document: Value) -> Value {
        serde_json::to_value(normalize(document).expect("the fixture normalizes"))
            .expect("serializes")
    }

    fn analytics_product_mut(document: &mut Value) -> &mut Value {
        &mut document["analytics"]["meta"]["product"]
    }

    #[test]
    fn the_product_is_the_analytics_record_with_the_pages_facts() {
        assert_eq!(
            normalized(document()),
            json!({"product": {
                "id": "gid://shopify/Product/9027458236635",
                "handle": "usa-15495",
                "url": "https://www.ugreen.com/products/usa-15495",
                "title": "UGREEN Revodok 105 5-in-1 USB-C Hub",
                "vendor": "UGREEN Brandsite",
                "product_type": null,
                "images": [
                    "https://www.ugreen.com/cdn/shop/files/d0a9824bf408d4d39de70224bbafb22d.png?v=1762322082"
                ],
                "sales_channel_urls": ["https://us.ugreen.com/products/15495"],
                "variants": [{
                    "id": "gid://shopify/ProductVariant/47897996820699",
                    "name": "UGREEN Revodok 105 5-in-1 USB-C Hub",
                    "sku": "usa-15495",
                    "price": {"amount": 130000, "currency": "USD"}
                }]
            }})
        );
    }

    /// As on `products-us-85185.html`, the one page without a sales channel.
    #[test]
    fn a_page_without_the_shop_now_button_has_no_sales_channel() {
        let mut input = document();
        input.as_object_mut().expect("an object").remove("shop_now");

        let product = &normalized(input)["product"];
        assert_eq!(product["sales_channel_urls"], json!([]));
        assert_eq!(product["variants"].as_array().map(Vec::len), Some(1));
    }

    /// The placeholder is kept: the offers are priced by the user's decision.
    #[test]
    fn the_placeholder_price_is_hundredths_of_the_paypal_currency() {
        let mut input = document();
        analytics_product_mut(&mut input)["variants"][0]["price"] = json!(500);

        assert_eq!(
            normalized(input)["product"]["variants"][0]["price"],
            json!({"amount": 500, "currency": "USD"})
        );
    }

    #[test]
    fn a_zero_price_is_no_price() {
        let mut input = document();
        analytics_product_mut(&mut input)["variants"][0]["price"] = json!(0);

        assert_eq!(
            normalized(input)["product"]["variants"][0]["price"],
            Value::Null
        );
    }

    #[test]
    fn an_empty_sku_is_no_sku() {
        let mut input = document();
        analytics_product_mut(&mut input)["variants"][0]["sku"] = json!("");
        input.as_object_mut().expect("an object").remove("shop_now");

        assert_eq!(
            normalized(input)["product"]["variants"][0]["sku"],
            Value::Null
        );
    }

    #[test]
    fn an_unsupported_currency_fails() {
        let mut input = document();
        input["paypal"]["data-currency"] = json!("XYZ");

        assert!(matches!(
            normalize(input),
            Err(NormalizeError::UnsupportedCurrency { currency, .. }) if currency == "XYZ"
        ));
    }

    #[test]
    fn a_currency_without_hundredths_fails() {
        let mut input = document();
        input["paypal"]["data-currency"] = json!("JPY");

        assert!(matches!(
            normalize(input),
            Err(NormalizeError::CurrencyWithoutHundredths { currency }) if currency == "JPY"
        ));
    }

    #[test]
    fn a_gid_for_another_product_fails() {
        let mut input = document();
        analytics_product_mut(&mut input)["gid"] = json!("gid://shopify/Product/1");

        assert!(matches!(
            normalize(input),
            Err(NormalizeError::ProductGidMismatch {
                id: 9027458236635,
                ..
            })
        ));
    }

    #[test]
    fn a_page_for_another_resource_fails() {
        let mut id = document();
        id["analytics"]["meta"]["page"]["resourceId"] = json!(1);
        assert!(matches!(
            normalize(id),
            Err(NormalizeError::PageResourceMismatch { resource_id: 1, .. })
        ));

        let mut kind = document();
        kind["analytics"]["meta"]["page"]["resourceType"] = json!("collection");
        assert!(matches!(
            normalize(kind),
            Err(NormalizeError::PageResourceMismatch { resource_type, .. })
                if resource_type == "collection"
        ));
    }

    #[test]
    fn a_canonical_url_for_another_handle_fails() {
        let mut input = document();
        input["canonical"]["href"] = json!("https://www.ugreen.com/products/usa-15532");

        assert!(matches!(
            normalize(input),
            Err(NormalizeError::CanonicalUrlMismatch { handle, .. }) if handle == "usa-15495"
        ));
    }

    #[test]
    fn a_title_the_variant_name_disagrees_with_fails() {
        let mut input = document();
        input["og_title"]["content"] = json!("UGREEN Revodok 105 | UGREEN");

        assert!(matches!(
            normalize(input),
            Err(NormalizeError::TitleMismatch {
                variant_id: 47897996820699,
                ..
            })
        ));
    }

    #[test]
    fn a_named_variant_is_not_held_to_the_title() {
        let mut input = document();
        let variant = &mut analytics_product_mut(&mut input)["variants"][0];
        variant["public_title"] = json!("Black");
        variant["name"] = json!("UGREEN Revodok 105 5-in-1 USB-C Hub - Black");

        assert_eq!(
            normalized(input)["product"]["variants"][0]["name"],
            json!("UGREEN Revodok 105 5-in-1 USB-C Hub - Black")
        );
    }

    #[test]
    fn a_shop_now_sku_no_variant_has_fails() {
        let mut input = document();
        input["shop_now"]["data-product-sku"] = json!("usa-15532");

        assert!(matches!(
            normalize(input),
            Err(NormalizeError::ShopNowSkuNotListed { sku }) if sku == "usa-15532"
        ));
    }
}
