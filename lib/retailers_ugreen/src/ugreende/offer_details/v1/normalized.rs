//! Expresses the storefront's product facts in the system's vocabulary. The page states the
//! product across several islands - Shopify's analytics `meta`, the main section's price block,
//! product form, variant picker, title and gallery - and they cross here as one product. Where two
//! islands state the same fact, normalization checks that they agree; what nothing needs stays
//! behind as the commented-out exclusion records in `product`.
//!
//! Only the selected variant is described beyond its price: the page renders no record of the
//! others' availability or markdowns outside the ld+json island, which is not read.

pub mod product;

use super::raw;
use money::{Currency, Money};
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// The storefront the pages are saved from; the page states its own address only as a path.
const ORIGIN: &str = "https://de.ugreen.com";

#[derive(Debug, Clone, PartialEq, Error)]
pub enum NormalizeError {
    #[error("shop currency `{currency}` is not supported: {reason}")]
    UnsupportedCurrency { currency: String, reason: String },

    #[error("shop currency `{currency}` has no hundredths, the unit Liquid renders amounts in")]
    CurrencyWithoutHundredths { currency: String },

    #[error("`{field}` `{value}` is not a whole number")]
    NotAWholeNumber { field: String, value: String },

    #[error("the {island} is for product `{id}`, the analytics for `{analytics}`")]
    ProductIdMismatch {
        island: String,
        id: String,
        analytics: u64,
    },

    #[error("the {island}'s URL `{url}` is not the product's `/products/{handle}`")]
    ProductUrlMismatch {
        island: String,
        url: String,
        handle: String,
    },

    #[error("the selected variant `{id}` is not among the analytics variants")]
    SelectedVariantNotListed { id: u64 },

    #[error("the variant picker renders no record of the selected variant")]
    PickerWithoutSelectedVariant,

    #[error("the picker's selected variant disagrees with the page on `{field}`")]
    SelectedVariantMismatch { field: String },

    #[error("the displayed price {displayed} is above the variant's price {listed}")]
    DisplayedPriceAboveListed { displayed: i64, listed: i64 },

    #[error(
        "the displayed price {displayed} is discounted from {listed}, but the struck-through \
         price is {compare}"
    )]
    DiscountedFromAnotherPrice {
        displayed: i64,
        listed: i64,
        compare: i64,
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

fn whole_number<T: std::str::FromStr>(value: &str, field: &str) -> Result<T, NormalizeError> {
    value.parse().map_err(|_| NormalizeError::NotAWholeNumber {
        field: field.to_string(),
        value: value.to_string(),
    })
}

// Liquid gives bare numeric ids; the family identifies Shopify objects by their Admin API gid.
fn variant_gid(id: u64) -> String {
    format!("gid://shopify/ProductVariant/{id}")
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UgreenDeNormalizedDocument {
    pub product: product::Product,
}

impl UgreenDeNormalizedDocument {
    pub fn from_raw(
        raw: raw::UgreenDeRawDocument,
    ) -> Result<UgreenDeNormalizedDocument, NormalizeError> {
        Ok(UgreenDeNormalizedDocument {
            product: product::Product::from_raw(raw)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{NormalizeError, UgreenDeNormalizedDocument};
    use crate::ugreende::offer_details::v1::raw::UgreenDeRawDocument;
    use crate::ugreende::offer_details::v1::raw::tests::{document, single_variant_document};
    use html_extractor::serde_json::{self, Value, json};

    fn normalize(document: Value) -> Result<UgreenDeNormalizedDocument, NormalizeError> {
        let raw: UgreenDeRawDocument =
            serde_json::from_value(document).expect("the raw fixture deserializes");
        UgreenDeNormalizedDocument::from_raw(raw)
    }

    fn normalized(document: Value) -> Value {
        serde_json::to_value(normalize(document).expect("the fixture normalizes"))
            .expect("serializes")
    }

    fn eur(amount: i64) -> Value {
        json!({"amount": amount, "currency": "EUR"})
    }

    #[test]
    fn the_product_is_assembled_from_every_island() {
        let product = &normalized(document())["product"];

        assert_eq!(product["id"], json!("gid://shopify/Product/10080883081559"));
        assert_eq!(product["handle"], json!("100w-usb-c-kabel-mit-e-marker"));
        assert_eq!(
            product["url"],
            json!("https://de.ugreen.com/products/100w-usb-c-kabel-mit-e-marker")
        );
        assert_eq!(
            product["title"],
            json!("UGREEN USB-C Kabel ( 100W ) mit E-Marker")
        );
        assert_eq!(product["vendor"], json!("UGREEN DE"));
        assert_eq!(product["product_type"], json!("Kabel"));
        assert_eq!(product["options"], json!(["Länge"]));
        assert_eq!(
            product["images"],
            json!(["https://de.ugreen.com/cdn/shop/files/71QIYZFjMgL.png?v=1760702732"])
        );
    }

    #[test]
    fn every_variant_carries_its_analytics_price() {
        let variants = &normalized(document())["product"]["variants"];

        assert_eq!(
            variants,
            &json!([
                {"id": "gid://shopify/ProductVariant/51557243388247", "title": "2M",
                    "name": "UGREEN USB-C Kabel ( 100W ) mit E-Marker - 2M",
                    "sku": "65833", "price": eur(1019)},
                {"id": "gid://shopify/ProductVariant/51557243421015", "title": "1M",
                    "name": "UGREEN USB-C Kabel ( 100W ) mit E-Marker - 1M",
                    "sku": "65834", "price": eur(1474)}
            ])
        );
    }

    #[test]
    fn the_selected_variant_is_described_from_the_picker_and_the_price_block() {
        let selected = &normalized(document())["product"]["selected_variant"];

        assert_eq!(
            selected,
            &json!({
                "id": "gid://shopify/ProductVariant/51557243388247",
                "available": true,
                "compare_at_price": eur(1099),
                "automatic_discount_price": null,
                "barcode": null,
                "options": [{"name": "Länge", "value": "2M"}]
            })
        );
    }

    #[test]
    fn a_single_variant_page_has_no_options() {
        let product = &normalized(single_variant_document())["product"];

        assert_eq!(product["options"], json!([]));
        assert_eq!(product["variants"][0]["title"], Value::Null);
        assert_eq!(product["selected_variant"]["options"], json!([]));
        assert_eq!(product["selected_variant"]["compare_at_price"], eur(1099));
    }

    #[test]
    fn a_disabled_add_to_cart_button_is_a_sold_out_variant() {
        let mut input = single_variant_document();
        input["add_to_cart"] = json!({"disabled": ""});

        assert_eq!(
            normalized(input)["product"]["selected_variant"]["available"],
            json!(false)
        );
    }

    #[test]
    fn a_struck_through_price_equal_to_the_price_is_no_markdown() {
        let mut input = single_variant_document();
        input["price"]["base_compare_cents"] = json!("1019");

        assert_eq!(
            normalized(input)["product"]["selected_variant"]["compare_at_price"],
            Value::Null
        );
    }

    #[test]
    fn a_displayed_price_below_the_variant_price_is_an_automatic_discount() {
        let mut input = single_variant_document();
        input["price"]["base_price_cents"] = json!("749");
        input["price"]["base_compare_cents"] = json!("1019");

        let selected = &normalized(input)["product"]["selected_variant"];
        assert_eq!(selected["automatic_discount_price"], eur(749));
        assert_eq!(selected["compare_at_price"], Value::Null);
    }

    #[test]
    fn an_automatic_discount_must_strike_through_the_variant_price() {
        let mut input = single_variant_document();
        input["price"]["base_price_cents"] = json!("749");
        input["price"]["base_compare_cents"] = json!("1099");

        assert!(matches!(
            normalize(input),
            Err(NormalizeError::DiscountedFromAnotherPrice { compare: 1099, .. })
        ));
    }

    #[test]
    fn a_displayed_price_above_the_variant_price_fails() {
        let mut input = single_variant_document();
        input["price"]["base_price_cents"] = json!("2000");

        assert!(matches!(
            normalize(input),
            Err(NormalizeError::DisplayedPriceAboveListed { .. })
        ));
    }

    #[test]
    fn the_islands_must_name_the_same_product() {
        for island in ["price", "product_form", "variant_picker"] {
            let mut input = document();
            input[island]["product_id"] = json!("1");

            assert!(
                matches!(
                    normalize(input),
                    Err(NormalizeError::ProductIdMismatch { .. })
                ),
                "{island}"
            );
        }

        let mut url = document();
        url["product_form"]["product_url"] = json!("/products/other");
        assert!(matches!(
            normalize(url),
            Err(NormalizeError::ProductUrlMismatch { .. })
        ));
    }

    #[test]
    fn the_selected_variant_must_be_listed_and_agree() {
        let mut unlisted = document();
        unlisted["selected_variant_id"]["value"] = json!("1");
        assert!(matches!(
            normalize(unlisted),
            Err(NormalizeError::SelectedVariantNotListed { id: 1 })
        ));

        let mut price = document();
        price["selected_variant"]["data"]["price"] = json!(999);
        assert!(matches!(
            normalize(price),
            Err(NormalizeError::SelectedVariantMismatch { field }) if field == "price"
        ));

        let mut missing = document();
        missing
            .as_object_mut()
            .expect("an object")
            .remove("selected_variant");
        assert!(matches!(
            normalize(missing),
            Err(NormalizeError::PickerWithoutSelectedVariant)
        ));
    }

    #[test]
    fn option_values_pair_with_the_legends_by_position() {
        let mut input = document();
        input["option_names"] = json!([{"text": "Länge:"}, {"text": "Farbe:"}]);

        assert!(matches!(
            normalize(input),
            Err(NormalizeError::OptionCountMismatch {
                offered: 2,
                selected: 1,
                ..
            })
        ));
    }

    #[test]
    fn an_unknown_currency_fails() {
        let mut input = document();
        input["price"]["currency"] = json!("CHF");

        assert!(matches!(
            normalize(input),
            Err(NormalizeError::UnsupportedCurrency { currency, .. }) if currency == "CHF"
        ));
    }

    #[test]
    fn a_price_attribute_that_is_not_a_number_fails() {
        let mut input = document();
        input["price"]["base_price_cents"] = json!("10,19");

        assert!(matches!(
            normalize(input),
            Err(NormalizeError::NotAWholeNumber { field, .. }) if field == "base_price_cents"
        ));
    }

    #[test]
    fn an_empty_sku_or_product_type_is_absent() {
        let mut input = document();
        input["analytics"]["meta"]["product"]["type"] = json!("");
        input["analytics"]["meta"]["product"]["variants"][0]["sku"] = json!("");

        let product = &normalized(input)["product"];
        assert_eq!(product["product_type"], Value::Null);
        assert_eq!(product["variants"][0]["sku"], Value::Null);
    }
}
