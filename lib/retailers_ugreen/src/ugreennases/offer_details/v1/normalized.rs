//! Expresses the storefront's product facts in the system's vocabulary. The page states the
//! product across several islands - Shopify's analytics `meta`, the picker's
//! `product.variants | json`, and the main section's title, cart form, price block and gallery -
//! and they cross here as one product. Where two islands state the same fact, normalization
//! checks that they agree; what nothing needs stays behind as the commented-out exclusion
//! records in `product`. The ld+json islands are read by the architecture but carry no product
//! fact (see `raw::ldjson`), so nothing here reads them.

pub mod product;

use super::raw;
use money::{Currency, Money};
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// The storefront the pages are saved from. The product URL is built from it and the handle,
/// never from the page's canonical link, which can point at another store:
/// 8 of the 18 saved pages carry a canonical link on `nas.ugreen.com`.
const ORIGIN: &str = "https://ai-es.ugreen.com";

#[derive(Debug, Clone, PartialEq, Error)]
pub enum NormalizeError {
    #[error("shop currency `{currency}` is not supported: {reason}")]
    UnsupportedCurrency { currency: String, reason: String },

    #[error("shop currency `{currency}` has no hundredths, the unit Liquid renders amounts in")]
    CurrencyWithoutHundredths { currency: String },

    #[error("the analytics product `{id}` carries the gid `{gid}`")]
    ProductGidMismatch { id: u64, gid: String },

    #[error("the analytics page is for resource `{resource_id}`, the product is `{product_id}`")]
    PageResourceMismatch { resource_id: u64, product_id: u64 },

    #[error("the product title is empty")]
    EmptyTitle,

    #[error("`{field}` `{value}` is not a whole number")]
    NotAWholeNumber { field: String, value: String },

    #[error("the price block's `{field}` text `{text}` is not a price with two decimals")]
    NotARenderedPrice { field: String, text: String },

    #[error("the selected variant `{id}` is not among the analytics variants")]
    SelectedVariantNotListed { id: u64 },

    #[error(
        "the price block's `{field}` shows {displayed}, the selected variant's price is {listed}"
    )]
    DisplayedPriceMismatch {
        field: String,
        displayed: i64,
        listed: i64,
    },

    #[error("the picker lists {picker} variants, the analytics {analytics}")]
    PickerVariantCountMismatch { picker: usize, analytics: usize },

    #[error(
        "variant `{variant_id}` `{field}` is `{picker}` in the picker, `{analytics}` in the analytics"
    )]
    PickerVariantMismatch {
        variant_id: u64,
        field: String,
        picker: String,
        analytics: String,
    },

    #[error("the page renders no picker for {count} variants")]
    NoPickerForSeveralVariants { count: usize },

    #[error("the price block says `{field}` is `{block}`, the selected variant `{variant}`")]
    PriceBlockMismatch {
        field: String,
        block: String,
        variant: String,
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
fn product_gid(id: u64) -> String {
    format!("gid://shopify/Product/{id}")
}

fn variant_gid(id: u64) -> String {
    format!("gid://shopify/ProductVariant/{id}")
}

fn https_from_protocol_relative(url: &str) -> String {
    match url.strip_prefix("//") {
        Some(rest) => format!("https://{rest}"),
        None => url.to_string(),
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UgreenNasEsNormalizedDocument {
    pub product: product::Product,
}

impl UgreenNasEsNormalizedDocument {
    pub fn from_raw(
        raw: raw::UgreenNasEsRawDocument,
    ) -> Result<UgreenNasEsNormalizedDocument, NormalizeError> {
        Ok(UgreenNasEsNormalizedDocument {
            product: product::Product::from_raw(raw)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{NormalizeError, UgreenNasEsNormalizedDocument};
    use crate::ugreennases::offer_details::v1::raw::UgreenNasEsRawDocument;
    use crate::ugreennases::offer_details::v1::raw::tests::{document, document_without_picker};
    use html_extractor::serde_json::{self, Value, json};

    fn normalize(document: Value) -> Result<UgreenNasEsNormalizedDocument, NormalizeError> {
        let raw: UgreenNasEsRawDocument =
            serde_json::from_value(document).expect("the raw fixture deserializes");
        UgreenNasEsNormalizedDocument::from_raw(raw)
    }

    fn normalized(document: Value) -> Value {
        serde_json::to_value(normalize(document).expect("the fixture normalizes"))
            .expect("serializes")
    }

    fn picker_variant_mut(document: &mut Value, index: usize) -> &mut Value {
        &mut document["variants"]["data"][index]
    }

    fn analytics_variant_mut(document: &mut Value, index: usize) -> &mut Value {
        &mut document["analytics"]["meta"]["product"]["variants"][index]
    }

    #[test]
    fn the_product_is_the_analytics_product_with_the_pages_title_and_gallery() {
        let product = &normalized(document())["product"];

        assert_eq!(product["id"], json!("gid://shopify/Product/10254048985370"));
        assert_eq!(
            product["handle"],
            json!("ugreen-nasync-dxp2800-gt-2-bay-nas-storage")
        );
        assert_eq!(
            product["url"],
            json!("https://ai-es.ugreen.com/products/ugreen-nasync-dxp2800-gt-2-bay-nas-storage")
        );
        assert_eq!(product["title"], json!("UGREEN NASync DXP2800 GT"));
        assert_eq!(product["vendor"], json!("UGREEN NAS US"));
        assert_eq!(product["product_type"], Value::Null);
        assert_eq!(
            product["images"],
            json!([
                "https://ai-es.ugreen.com/cdn/shop/files/DXP2800_GT-16051-nuevo.webp?v=1780642511",
                "https://ai-es.ugreen.com/cdn/shop/files/01-dxp2800gt-almacene-80tb.jpg?v=1780731996"
            ])
        );
        assert_eq!(product["variants"].as_array().map(Vec::len), Some(2));
    }

    #[test]
    fn a_variant_takes_the_analytics_facts_and_the_pickers_record() {
        let variant = &normalized(document())["product"]["variants"][0];

        assert_eq!(
            variant,
            &json!({
                "id": "gid://shopify/ProductVariant/52296690008346",
                "title": "DXP2800 GT (Envío desde el 27 de octubre)",
                "name": "UGREEN NASync DXP2800 GT - DXP2800 GT (Envío desde el 27 de octubre)",
                "sku": "16051",
                "barcode": "6942752010514",
                "price": {"amount": 45999, "currency": "EUR"},
                "compare_at_price": {"amount": 50999, "currency": "EUR"},
                "available": true,
                "weight": null,
                "image": "https://ai-es.ugreen.com/cdn/shop/files/DXP2800_GT-16051-nuevo.webp?v=1780642511",
                "options": ["DXP2800 GT (Envío desde el 27 de octubre)"]
            })
        );
    }

    #[test]
    fn the_title_is_trimmed_and_its_spaces_collapsed() {
        let mut input = document();
        input["title"]["text"] = json!("\n  UGREEN NAS 72W DC USV  (Versand ab 2. Nov.) \n");

        assert_eq!(
            normalized(input)["product"]["title"],
            json!("UGREEN NAS 72W DC USV (Versand ab 2. Nov.)")
        );

        let mut empty = document();
        empty["title"]["text"] = json!(" \n ");
        assert_eq!(normalize(empty), Err(NormalizeError::EmptyTitle));
    }

    #[test]
    fn a_compare_at_price_not_above_the_price_is_no_markdown() {
        let mut input = document();
        input["price"]["class"] = json!("price price--large");
        input["price"]["compare_at"]["text"] = json!("€459,99");
        picker_variant_mut(&mut input, 0)["compare_at_price"] = json!(45999);
        picker_variant_mut(&mut input, 1)["compare_at_price"] = json!(1000);

        let variants = &normalized(input)["product"]["variants"];
        assert_eq!(variants[0]["compare_at_price"], Value::Null);
        assert_eq!(variants[1]["compare_at_price"], Value::Null);
    }

    #[test]
    fn a_zero_price_is_no_price() {
        let mut input = document();
        analytics_variant_mut(&mut input, 1)["price"] = json!(0);
        picker_variant_mut(&mut input, 1)["price"] = json!(0);

        let variant = &normalized(input)["product"]["variants"][1];
        assert_eq!(variant["price"], Value::Null);
        assert_eq!(variant["compare_at_price"], Value::Null);
    }

    #[test]
    fn a_barcode_that_is_not_all_digits_is_dropped() {
        let mut input = document();
        picker_variant_mut(&mut input, 0)["barcode"] = json!("\u{a0}6942752010514 ");
        picker_variant_mut(&mut input, 1)["barcode"] = json!("N/A");
        let variants = &normalized(input)["product"]["variants"];
        assert_eq!(variants[0]["barcode"], json!("6942752010514"));
        assert_eq!(variants[1]["barcode"], Value::Null);

        let mut empty = document();
        picker_variant_mut(&mut empty, 0)["barcode"] = json!(" ");
        assert_eq!(
            normalized(empty)["product"]["variants"][0]["barcode"],
            Value::Null
        );
    }

    /// As on `ugreennasde`'s bundle variants (`ugreen-nasync-dh2300-nas-speicher` and 6 more
    /// pages), which carry 14-digit `178…` codes: GTIN-14 is a valid length.
    #[test]
    fn a_fourteen_digit_barcode_is_kept() {
        let mut input = document();
        picker_variant_mut(&mut input, 1)["barcode"] = json!("17841947458048");

        assert_eq!(
            normalized(input)["product"]["variants"][1]["barcode"],
            json!("17841947458048")
        );
    }

    #[test]
    fn an_empty_sku_is_no_sku() {
        let mut input = document();
        analytics_variant_mut(&mut input, 1)["sku"] = json!("");
        picker_variant_mut(&mut input, 1)["sku"] = json!("");

        assert_eq!(
            normalized(input)["product"]["variants"][1]["sku"],
            Value::Null
        );
    }

    #[test]
    fn availability_and_weight_come_from_the_picker() {
        let mut input = document();
        picker_variant_mut(&mut input, 1)["available"] = json!(false);
        picker_variant_mut(&mut input, 1)["weight"] = json!(16800);

        let variant = &normalized(input)["product"]["variants"][1];
        assert_eq!(variant["available"], json!(false));
        assert_eq!(variant["weight"], json!(16800));
    }

    #[test]
    fn without_a_picker_the_price_block_states_the_markdown_and_availability() {
        let variant = &normalized(document_without_picker())["product"]["variants"];
        assert_eq!(
            variant,
            &json!([{
                "id": "gid://shopify/ProductVariant/52296690008346",
                "title": "DXP2800 GT (Envío desde el 27 de octubre)",
                "name": "UGREEN NASync DXP2800 GT - DXP2800 GT (Envío desde el 27 de octubre)",
                "sku": "16051",
                "barcode": null,
                "price": {"amount": 45999, "currency": "EUR"},
                "compare_at_price": {"amount": 50999, "currency": "EUR"},
                "available": true,
                "weight": null,
                "image": null,
                "options": []
            }])
        );
    }

    /// As on `ugreen-nas-10gbps-m2-nvme-ssd-enclosure` on `ugreennasde` and `ugreennases`: sold
    /// out, off sale, with the price repeated in the `<s>`.
    #[test]
    fn without_a_picker_a_sold_out_variant_off_sale_has_no_markdown() {
        let mut input = document_without_picker();
        input["price"]["class"] = json!("price price--large price--sold-out");
        input["price"]["compare_at"]["text"] = json!("€459,99");
        input["selected_variant"]["disabled"] = json!("");

        let variant = &normalized(input)["product"]["variants"][0];
        assert_eq!(variant["available"], json!(false));
        assert_eq!(variant["compare_at_price"], Value::Null);
    }

    #[test]
    fn without_a_picker_an_empty_strike_through_is_no_markdown() {
        let mut input = document_without_picker();
        input["price"]["class"] = json!("price price--large");
        input["price"]["compare_at"]["text"] = json!("");

        let variant = &normalized(input)["product"]["variants"][0];
        assert_eq!(variant["compare_at_price"], Value::Null);
    }

    #[test]
    fn several_variants_without_a_picker_fail() {
        let mut input = document();
        input.as_object_mut().expect("an object").remove("variants");

        assert_eq!(
            normalize(input),
            Err(NormalizeError::NoPickerForSeveralVariants { count: 2 })
        );
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
    fn an_unknown_currency_fails() {
        let mut input = document();
        input["paypal"]["data-currency"] = json!("XYZ");

        assert!(matches!(
            normalize(input),
            Err(NormalizeError::UnsupportedCurrency { currency, .. }) if currency == "XYZ"
        ));
    }

    #[test]
    fn the_analytics_must_describe_one_product() {
        let mut gid = document();
        gid["analytics"]["meta"]["product"]["gid"] = json!("gid://shopify/Product/1");
        assert!(matches!(
            normalize(gid),
            Err(NormalizeError::ProductGidMismatch {
                id: 10254048985370,
                ..
            })
        ));

        let mut page = document();
        page["analytics"]["meta"]["page"]["resourceId"] = json!(1);
        assert_eq!(
            normalize(page),
            Err(NormalizeError::PageResourceMismatch {
                resource_id: 1,
                product_id: 10254048985370
            })
        );
    }

    #[test]
    fn the_selected_variant_must_be_listed() {
        let mut input = document();
        input["selected_variant"]["value"] = json!("1");
        assert_eq!(
            normalize(input),
            Err(NormalizeError::SelectedVariantNotListed { id: 1 })
        );

        let mut input = document();
        input["selected_variant"]["value"] = json!("x");
        assert!(matches!(
            normalize(input),
            Err(NormalizeError::NotAWholeNumber { field, .. }) if field == "selected_variant"
        ));
    }

    #[test]
    fn the_price_block_must_show_the_selected_variants_price() {
        let mut input = document();
        input["price"]["sale"]["text"] = json!("€449,99");
        assert_eq!(
            normalize(input),
            Err(NormalizeError::DisplayedPriceMismatch {
                field: "sale".to_string(),
                displayed: 44999,
                listed: 45999
            })
        );

        let mut input = document();
        input["price"]["regular"]["text"] = json!("€459,9");
        assert!(matches!(
            normalize(input),
            Err(NormalizeError::NotARenderedPrice { field, .. }) if field == "regular"
        ));
    }

    #[test]
    fn the_price_block_must_agree_with_the_selected_variant() {
        let mut on_sale = document();
        on_sale["price"]["class"] = json!("price price--large");
        assert!(matches!(
            normalize(on_sale),
            Err(NormalizeError::PriceBlockMismatch { field, .. }) if field == "on_sale"
        ));

        let mut compare_at = document();
        compare_at["price"]["compare_at"]["text"] = json!("€499,99");
        assert!(matches!(
            normalize(compare_at),
            Err(NormalizeError::PriceBlockMismatch { field, .. }) if field == "compare_at"
        ));

        let mut sold_out = document();
        sold_out["price"]["class"] = json!("price price--large price--on-sale price--sold-out");
        assert!(matches!(
            normalize(sold_out),
            Err(NormalizeError::PriceBlockMismatch { field, .. }) if field == "sold_out"
        ));

        let mut available = document();
        available["price"]["class"] = json!("price price--large price--on-sale price--sold-out");
        available["selected_variant"]["disabled"] = json!("");
        assert!(matches!(
            normalize(available),
            Err(NormalizeError::PriceBlockMismatch { field, .. }) if field == "available"
        ));
    }

    #[test]
    fn the_picker_must_list_the_analytics_variants() {
        let mut count = document();
        count["variants"]["data"]
            .as_array_mut()
            .expect("records")
            .truncate(1);
        assert_eq!(
            normalize(count),
            Err(NormalizeError::PickerVariantCountMismatch {
                picker: 1,
                analytics: 2
            })
        );

        for (field, value) in [
            ("id", json!(1)),
            ("sku", json!("16055")),
            ("price", json!(1)),
        ] {
            let mut input = document();
            picker_variant_mut(&mut input, 1)[field] = value;
            assert!(
                matches!(
                    normalize(input),
                    Err(NormalizeError::PickerVariantMismatch { field: f, .. }) if f == field
                ),
                "{field}"
            );
        }
    }
}
