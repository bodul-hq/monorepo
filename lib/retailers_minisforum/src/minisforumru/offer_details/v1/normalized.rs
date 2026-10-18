//! Expresses the storefront's product facts in the system's vocabulary. The page states the
//! product across several islands - Shopify's analytics `meta`, the variant picker's records, the
//! title and the gallery - and they cross here as one product. Where two islands state the same
//! fact, normalization checks that they agree; what nothing needs stays behind as the
//! commented-out exclusion records in `product`.

pub mod product;

use super::raw;
use money::{Currency, Money};
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// The storefront the pages are saved from; the page states its own address only as a path.
const ORIGIN: &str = "https://minisforum.ru";

#[derive(Debug, Clone, PartialEq, Error)]
pub enum NormalizeError {
    #[error("shop currency `{currency}` is not supported: {reason}")]
    UnsupportedCurrency { currency: String, reason: String },

    #[error("shop currency `{currency}` has no hundredths, the unit Liquid renders amounts in")]
    CurrencyWithoutHundredths { currency: String },

    #[error("the variant picker is for product `{picker}`, the analytics for `{analytics}`")]
    ProductIdMismatch { picker: String, analytics: u64 },

    #[error("the variant picker's URL `{url}` is not the product's `/products/{handle}`")]
    ProductUrlMismatch { url: String, handle: String },

    #[error("the analytics list variants {analytics:?}, the variant picker {picker:?}")]
    VariantIdsMismatch {
        analytics: Vec<u64>,
        picker: Vec<u64>,
    },

    #[error("variant `{variant_id}` selects {selected} option values against {offered} options")]
    OptionCountMismatch {
        variant_id: u64,
        offered: usize,
        selected: usize,
    },
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

// Liquid gives bare numeric ids; the family identifies Shopify objects by their Admin API gid.
fn variant_gid(id: u64) -> String {
    format!("gid://shopify/ProductVariant/{id}")
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MinisforumRuNormalizedDocument {
    pub product: product::Product,
}

impl MinisforumRuNormalizedDocument {
    pub fn from_raw(
        raw: raw::MinisforumRuRawDocument,
    ) -> Result<MinisforumRuNormalizedDocument, NormalizeError> {
        Ok(MinisforumRuNormalizedDocument {
            product: product::Product::from_raw(raw)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{MinisforumRuNormalizedDocument, NormalizeError};
    use crate::minisforumru::offer_details::v1::raw::MinisforumRuRawDocument;
    use crate::minisforumru::offer_details::v1::raw::tests::document;
    use html_extractor::serde_json::{self, Value, json};

    fn normalize(document: Value) -> Result<MinisforumRuNormalizedDocument, NormalizeError> {
        let raw: MinisforumRuRawDocument =
            serde_json::from_value(document).expect("the raw fixture deserializes");
        MinisforumRuNormalizedDocument::from_raw(raw)
    }

    fn normalized(document: Value) -> Value {
        serde_json::to_value(normalize(document).expect("the fixture normalizes"))
            .expect("serializes")
    }

    fn picker_variant_mut(document: &mut Value, index: usize) -> &mut Value {
        &mut document["variants"]["data"][index]
    }

    #[test]
    fn the_product_is_assembled_from_every_island() {
        let product = &normalized(document())["product"];

        assert_eq!(product["id"], json!("gid://shopify/Product/8640842891442"));
        assert_eq!(product["handle"], json!("minisforum-ms-s1-max"));
        assert_eq!(
            product["url"],
            json!("https://minisforum.ru/products/minisforum-ms-s1-max")
        );
        assert_eq!(product["title"], json!("MINISFORUM MS-S1 MAX"));
        assert_eq!(product["vendor"], json!("Minisforum RU"));
        assert_eq!(product["product_type"], json!("Mini PC"));
        assert_eq!(product["options"], json!(["CPU", "Type", "Adapter"]));
        assert_eq!(product["variants"].as_array().map(Vec::len), Some(2));
    }

    #[test]
    fn a_price_is_hundredths_of_the_shop_currency() {
        let mut input = document();
        picker_variant_mut(&mut input, 0)["compare_at_price"] = json!(289900);

        let variant = &normalized(input)["product"]["variants"][0];
        assert_eq!(
            variant["price"],
            json!({"amount": 259900, "currency": "USD"})
        );
        assert_eq!(
            variant["compare_at_price"],
            json!({"amount": 289900, "currency": "USD"})
        );
    }

    #[test]
    fn a_currency_without_hundredths_fails() {
        let mut input = document();
        input["shop"]["data"]["currencyCode"] = json!("JPY");

        assert!(matches!(
            normalize(input),
            Err(NormalizeError::CurrencyWithoutHundredths { currency }) if currency == "JPY"
        ));
    }

    #[test]
    fn an_unknown_currency_fails() {
        let mut input = document();
        input["shop"]["data"]["currencyCode"] = json!("RUB");

        assert!(matches!(
            normalize(input),
            Err(NormalizeError::UnsupportedCurrency { currency, .. }) if currency == "RUB"
        ));
    }

    #[test]
    fn a_single_variant_page_reads_the_selected_variant() {
        let mut input = document();
        input.as_object_mut().expect("an object").remove("variants");
        input["analytics"]["meta"]["product"]["variants"]
            .as_array_mut()
            .expect("variants")
            .truncate(1);

        let variants = &normalized(input)["product"]["variants"];
        assert_eq!(
            variants,
            &json!([normalized(document())["product"]["variants"][0].clone()])
        );
    }

    #[test]
    fn the_analytics_and_the_picker_must_list_the_same_variants() {
        let mut input = document();
        picker_variant_mut(&mut input, 1)["id"] = json!(1);

        assert!(matches!(
            normalize(input),
            Err(NormalizeError::VariantIdsMismatch { picker, .. }) if picker == [47567232336050, 1]
        ));
    }

    #[test]
    fn the_picker_and_the_analytics_must_name_the_same_product() {
        let mut id = document();
        id["variant_picker"]["product_id"] = json!("1");
        assert!(matches!(
            normalize(id),
            Err(NormalizeError::ProductIdMismatch { .. })
        ));

        let mut url = document();
        url["variant_picker"]["product_url"] = json!("/products/other");
        assert!(matches!(
            normalize(url),
            Err(NormalizeError::ProductUrlMismatch { .. })
        ));
    }

    #[test]
    fn option_values_pair_with_the_legends_by_position() {
        let output = normalized(document());
        assert_eq!(
            output["product"]["variants"][0]["options"],
            json!([
                {"name": "CPU", "value": "AMD Ryzen™ AI Max+ 395"},
                {"name": "Type", "value": "64GB RAM+2TB SSD"},
                {"name": "Adapter", "value": "EU"}
            ])
        );

        let mut input = document();
        input["option_names"] = json!([{"text": "CPU:"}]);
        assert!(matches!(
            normalize(input),
            Err(NormalizeError::OptionCountMismatch {
                offered: 1,
                selected: 3,
                ..
            })
        ));
    }

    #[test]
    fn an_empty_or_null_sku_or_barcode_is_absent() {
        for value in [json!(""), Value::Null] {
            let mut input = document();
            picker_variant_mut(&mut input, 0)["sku"] = value.clone();
            picker_variant_mut(&mut input, 0)["barcode"] = value.clone();

            let variant = &normalized(input)["product"]["variants"][0];
            assert_eq!(variant["sku"], Value::Null, "sku {value}");
            assert_eq!(variant["barcode"], Value::Null, "barcode {value}");
        }
    }

    #[test]
    fn an_empty_product_type_is_absent() {
        let mut input = document();
        input["analytics"]["meta"]["product"]["type"] = json!("");

        assert_eq!(normalized(input)["product"]["product_type"], Value::Null);
    }

    #[test]
    fn ids_become_gids_and_images_become_https_without_a_display_width() {
        let product = &normalized(document())["product"];

        assert_eq!(
            product["variants"][0]["id"],
            json!("gid://shopify/ProductVariant/47567232336050")
        );
        assert_eq!(
            product["images"],
            json!(["https://minisforum.ru/cdn/shop/files/S1MAX.png?v=1781059869"])
        );
    }
}
