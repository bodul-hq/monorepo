//! Expresses the storefront's product facts in the system's vocabulary. The schema.org `Product`
//! island carries the offers, Shopify's analytics `meta` the ids, the variant picker (where there
//! is one) availability and compare-at prices, and the main product section the rendered price
//! block and gallery; they cross here as one product. Where two islands state the same fact,
//! normalization checks that they agree; what nothing needs stays behind as the commented-out
//! exclusion records in `product`.

pub mod product;

use super::raw;
use money::{Currency, Money};
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// The storefront the pages are saved from.
const ORIGIN: &str = "https://ugreen.jp";

#[derive(Debug, Clone, PartialEq, Error)]
pub enum NormalizeError {
    #[error("no `Product` ld+json island on the document")]
    MissingProductIsland,

    #[error("{count} `Product` ld+json islands on the document")]
    MultipleProductIslands { count: usize },

    #[error("`{field}` is in `{currency}`; only JPY is read")]
    UnsupportedCurrency { field: String, currency: String },

    #[error("`{field}` of {cents} hundredths is not a whole number of yen")]
    CentsNotWholeYen { field: String, cents: i64 },

    #[error("`{field}` of {amount} is not a whole number of yen")]
    AmountNotWholeYen { field: String, amount: f64 },

    #[error("`{field}` text `{text}` is not a yen amount such as `¥65,880`")]
    PriceTextNotYen { field: String, text: String },

    #[error("the analytics gid `{gid}` is not product `{id}`'s")]
    ProductGidMismatch { gid: String, id: u64 },

    #[error("the island's URL `{url}` is not the analytics handle `{handle}`'s product page")]
    ProductUrlMismatch { url: String, handle: String },

    #[error("offer URL `{url}` is not the product URL with a numeric `variant`")]
    OfferUrlWithoutVariant { url: String },

    #[error("the analytics list variants {analytics:?}, {island} {other:?}")]
    VariantIdsMismatch {
        island: String,
        analytics: Vec<u64>,
        other: Vec<u64>,
    },

    #[error(
        "variant `{variant_id}` `{field}` is `{analytics}` in the analytics, `{other}` in {island}"
    )]
    VariantFieldMismatch {
        variant_id: u64,
        field: String,
        island: String,
        analytics: String,
        other: String,
    },

    #[error("offer availability `{value}` is neither in nor out of stock")]
    UnknownAvailability { value: String },

    #[error("no variant picker on a product with options ({count} variants)")]
    MissingVariantPicker { count: usize },

    #[error("selected variant `{value}` is not one of the product's variants")]
    SelectedVariantNotOffered { value: String },

    #[error("the price block shows `{rendered}` for the selected variant, which costs {price} yen")]
    RenderedPriceMismatch { rendered: String, price: i64 },
}

fn yen_currency(code: &str, field: &str) -> Result<Currency, NormalizeError> {
    match code {
        "JPY" => Ok(Currency::JPY),
        other => Err(NormalizeError::UnsupportedCurrency {
            field: field.to_string(),
            currency: other.to_string(),
        }),
    }
}

// The island states whole yen as a float, such as `13980.0`.
fn whole_yen_from_float(amount: f64, field: &str) -> Result<i64, NormalizeError> {
    if amount.fract() != 0.0 || amount < 0.0 || amount > i64::MAX as f64 {
        return Err(NormalizeError::AmountNotWholeYen {
            field: field.to_string(),
            amount,
        });
    }

    Ok(amount as i64)
}

// Liquid renders every amount in hundredths of the shop currency, yen included, although yen has
// no minor unit: `1398000` is ¥13,980.
fn whole_yen_from_shopify_cents(cents: i64, field: &str) -> Result<i64, NormalizeError> {
    if cents % 100 != 0 {
        return Err(NormalizeError::CentsNotWholeYen {
            field: field.to_string(),
            cents,
        });
    }

    Ok(cents / 100)
}

// The price block renders `¥65,880`: the yen sign, then whole yen grouped in threes by commas.
fn whole_yen_from_text(text: &str, field: &str) -> Result<i64, NormalizeError> {
    let not_yen = || NormalizeError::PriceTextNotYen {
        field: field.to_string(),
        text: text.to_string(),
    };

    let digits = text.trim().strip_prefix('¥').ok_or_else(not_yen)?;
    let groups: Vec<&str> = digits.split(',').collect();
    let well_grouped = groups.iter().enumerate().all(|(index, group)| {
        let width_ok = if index == 0 {
            (1..=3).contains(&group.len())
        } else {
            group.len() == 3
        };
        width_ok && group.bytes().all(|b| b.is_ascii_digit())
    });
    if !well_grouped {
        return Err(not_yen());
    }

    groups.concat().parse().map_err(|_| not_yen())
}

fn yen(amount: i64) -> Money {
    Money::new(amount, Currency::JPY)
}

fn https_from_protocol_relative(url: &str) -> String {
    match url.strip_prefix("//") {
        Some(rest) => format!("https://{rest}"),
        None => url.to_string(),
    }
}

// The family identifies Shopify objects by their Admin API gid.
fn product_gid(id: u64) -> String {
    format!("gid://shopify/Product/{id}")
}

fn variant_gid(id: u64) -> String {
    format!("gid://shopify/ProductVariant/{id}")
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UgreenJpNormalizedDocument {
    pub product: product::Product,
}

impl UgreenJpNormalizedDocument {
    pub fn from_raw(
        raw: raw::UgreenJpRawDocument,
    ) -> Result<UgreenJpNormalizedDocument, NormalizeError> {
        yen_currency(&raw.paypal.currency, "data-currency")?;

        Ok(UgreenJpNormalizedDocument {
            product: product::Product::from_raw(raw)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{NormalizeError, UgreenJpNormalizedDocument, whole_yen_from_text};
    use crate::ugreenjp::offer_details::v1::raw::UgreenJpRawDocument;
    use crate::ugreenjp::offer_details::v1::raw::tests::{default_title_document, document};
    use html_extractor::serde_json::{self, Value, json};

    fn normalize(document: Value) -> Result<UgreenJpNormalizedDocument, NormalizeError> {
        let raw: UgreenJpRawDocument =
            serde_json::from_value(document).expect("the raw fixture deserializes");
        UgreenJpNormalizedDocument::from_raw(raw)
    }

    fn normalized(document: Value) -> Value {
        serde_json::to_value(normalize(document).expect("the fixture normalizes"))
            .expect("serializes")
    }

    fn island_mut(document: &mut Value) -> &mut Value {
        &mut document["ld-json"][1]["data"]
    }

    fn analytics_variant_mut(document: &mut Value, index: usize) -> &mut Value {
        &mut document["analytics"]["meta"]["product"]["variants"][index]
    }

    fn picker_variant_mut(document: &mut Value, index: usize) -> &mut Value {
        &mut document["variants"]["data"][index]
    }

    #[test]
    fn the_product_crosses_from_the_islands_and_the_section() {
        let product = &normalized(document())["product"];

        assert_eq!(product["id"], json!("gid://shopify/Product/9456052437272"));
        assert_eq!(product["handle"], json!("lightning-usb-cable"));
        assert_eq!(
            product["url"],
            json!("https://ugreen.jp/products/lightning-usb-cable")
        );
        assert_eq!(product["title"], json!("UGREENライトニング USB ケーブル"));
        assert_eq!(
            product["description"],
            json!("L字型コネクター設計、\nApple 正規ライセンス MFi 取得")
        );
        assert_eq!(product["vendor"], json!("UGREEN-JP"));
        assert_eq!(product["product_type"], Value::Null);
        assert_eq!(
            product["images"],
            json!([
                "https://ugreen.jp/cdn/shop/files/60521-1.png?v=1726277735",
                "https://ugreen.jp/cdn/shop/files/60521-2.png?v=1726277735"
            ])
        );
        assert_eq!(product["variants"].as_array().map(Vec::len), Some(2));
    }

    #[test]
    fn a_variant_crosses_with_its_offer_and_picker_facts() {
        let variant = &normalized(document())["product"]["variants"][1];

        assert_eq!(
            variant,
            &json!({
                "id": "gid://shopify/ProductVariant/49154830631192",
                "title": "2M",
                "name": "UGREENライトニング USB ケーブル - 2M",
                "sku": "70733",
                "gtin": null,
                "price": {"amount": 1599, "currency": "JPY"},
                "compare_at_price": null,
                "available": true,
                "options": ["2M"],
                "image": null
            })
        );
    }

    #[test]
    fn a_price_is_whole_yen() {
        let variants = &normalized(document())["product"]["variants"];
        assert_eq!(
            variants[0]["price"],
            json!({"amount": 1399, "currency": "JPY"})
        );
    }

    #[test]
    fn a_hundredths_price_that_is_not_whole_yen_fails() {
        let mut input = document();
        analytics_variant_mut(&mut input, 0)["price"] = json!(139950);

        assert!(matches!(
            normalize(input),
            Err(NormalizeError::CentsNotWholeYen { field, cents: 139950 }) if field == "meta.variants.price"
        ));

        let mut picker = document();
        picker_variant_mut(&mut picker, 0)["compare_at_price"] = json!(150050);
        assert!(matches!(
            normalize(picker),
            Err(NormalizeError::CentsNotWholeYen { field, cents: 150050 })
                if field == "variants.compare_at_price"
        ));
    }

    #[test]
    fn a_fractional_offer_price_fails() {
        let mut input = document();
        island_mut(&mut input)["offers"][0]["price"] = json!(1399.5);

        assert!(matches!(
            normalize(input),
            Err(NormalizeError::AmountNotWholeYen { field, .. }) if field == "offers.price"
        ));
    }

    #[test]
    fn a_zero_price_is_no_price() {
        let mut input = document();
        island_mut(&mut input)["offers"][0]["price"] = json!(0.0);
        analytics_variant_mut(&mut input, 0)["price"] = json!(0);
        picker_variant_mut(&mut input, 0)["price"] = json!(0);
        input["price"]["sale"]["text"] = json!("¥0");

        let variants = &normalized(input)["product"]["variants"];
        assert_eq!(variants[0]["price"], Value::Null);
        assert_eq!(variants[1]["price"]["amount"], json!(1599));
    }

    #[test]
    fn the_pickers_compare_at_price_is_kept_only_above_the_price() {
        let mut input = document();
        picker_variant_mut(&mut input, 0)["compare_at_price"] = json!(179900);
        picker_variant_mut(&mut input, 1)["compare_at_price"] = json!(159900);
        let variants = &normalized(input)["product"]["variants"];
        assert_eq!(
            variants[0]["compare_at_price"],
            json!({"amount": 1799, "currency": "JPY"})
        );
        assert_eq!(variants[1]["compare_at_price"], Value::Null);

        let mut zero = document();
        picker_variant_mut(&mut zero, 0)["compare_at_price"] = json!(0);
        assert_eq!(
            normalized(zero)["product"]["variants"][0]["compare_at_price"],
            Value::Null
        );
    }

    #[test]
    fn a_picker_page_ignores_the_rendered_compare_at_price() {
        let mut input = document();
        input["price"]["class"] = json!("price price--product price--on-sale");
        input["price"]["compare_at"]["text"] = json!("¥1,799");

        assert_eq!(
            normalized(input)["product"]["variants"][0]["compare_at_price"],
            Value::Null
        );
    }

    #[test]
    fn a_default_title_product_takes_the_rendered_compare_at_price_on_sale() {
        let product = &normalized(default_title_document())["product"];
        let variant = &product["variants"][0];

        assert_eq!(product["variants"].as_array().map(Vec::len), Some(1));
        assert_eq!(variant["title"], Value::Null);
        assert_eq!(variant["sku"], Value::Null);
        assert_eq!(variant["options"], json!([]));
        assert_eq!(
            variant["price"],
            json!({"amount": 52700, "currency": "JPY"})
        );
        assert_eq!(
            variant["compare_at_price"],
            json!({"amount": 65880, "currency": "JPY"})
        );
    }

    #[test]
    fn a_default_title_product_off_sale_has_no_compare_at_price() {
        let mut input = default_title_document();
        input["price"]["class"] = json!("price price--product");
        input["price"]["compare_at"]["text"] = json!("¥52,700");

        assert_eq!(
            normalized(input)["product"]["variants"][0]["compare_at_price"],
            Value::Null
        );
    }

    #[test]
    fn an_unreadable_rendered_compare_at_price_fails() {
        let mut input = default_title_document();
        input["price"]["compare_at"]["text"] = json!("65,880円");

        assert!(matches!(
            normalize(input),
            Err(NormalizeError::PriceTextNotYen { field, .. }) if field == "price.compare_at"
        ));
    }

    #[test]
    fn a_rendered_price_is_read_exactly() {
        assert_eq!(whole_yen_from_text("¥65,880", "f"), Ok(65880));
        assert_eq!(whole_yen_from_text("¥1,234,567", "f"), Ok(1234567));
        assert_eq!(whole_yen_from_text("¥980", "f"), Ok(980));
        assert_eq!(whole_yen_from_text(" ¥0 ", "f"), Ok(0));
        for text in [
            "", "¥", "65,880", "¥65,88", "¥6588,0", "¥65.880", "¥,880", "¥-1",
        ] {
            assert!(whole_yen_from_text(text, "f").is_err(), "{text}");
        }
    }

    #[test]
    fn the_price_block_must_show_the_selected_variants_price() {
        let mut input = document();
        input["price"]["sale"]["text"] = json!("¥1,599");

        assert!(matches!(
            normalize(input),
            Err(NormalizeError::RenderedPriceMismatch { price: 1399, .. })
        ));
    }

    #[test]
    fn the_selected_variant_must_be_offered() {
        let mut input = document();
        input["selected_variant_id"]["value"] = json!("1");

        assert!(matches!(
            normalize(input),
            Err(NormalizeError::SelectedVariantNotOffered { value }) if value == "1"
        ));
    }

    #[test]
    fn a_product_with_options_needs_its_picker() {
        let mut input = document();
        input.as_object_mut().expect("an object").remove("variants");

        assert!(matches!(
            normalize(input),
            Err(NormalizeError::MissingVariantPicker { count: 2 })
        ));
    }

    #[test]
    fn another_currency_fails() {
        let mut paypal = document();
        paypal["paypal"]["data-currency"] = json!("USD");
        assert!(matches!(
            normalize(paypal),
            Err(NormalizeError::UnsupportedCurrency { field, .. }) if field == "data-currency"
        ));

        let mut offer = document();
        island_mut(&mut offer)["offers"][1]["priceCurrency"] = json!("USD");
        assert!(matches!(
            normalize(offer),
            Err(NormalizeError::UnsupportedCurrency { field, .. }) if field == "offers.priceCurrency"
        ));
    }

    #[test]
    fn the_island_is_required_and_read_once() {
        let mut missing = document();
        missing["ld-json"]
            .as_array_mut()
            .expect("islands")
            .truncate(1);
        assert_eq!(
            normalize(missing),
            Err(NormalizeError::MissingProductIsland)
        );

        let mut twice = document();
        let island = twice["ld-json"][1].clone();
        twice["ld-json"]
            .as_array_mut()
            .expect("islands")
            .push(island);
        assert_eq!(
            normalize(twice),
            Err(NormalizeError::MultipleProductIslands { count: 2 })
        );
    }

    #[test]
    fn the_gid_must_be_the_products() {
        let mut input = document();
        input["analytics"]["meta"]["product"]["gid"] = json!("gid://shopify/Product/1");

        assert!(matches!(
            normalize(input),
            Err(NormalizeError::ProductGidMismatch {
                id: 9456052437272,
                ..
            })
        ));
    }

    #[test]
    fn the_island_url_must_be_the_handles_page() {
        let mut input = document();
        input["analytics"]["meta"]["product"]["handle"] = json!("other");

        assert!(matches!(
            normalize(input),
            Err(NormalizeError::ProductUrlMismatch { handle, .. }) if handle == "other"
        ));
    }

    /// As on `products-ugreen-nexode-65w-急速-充電器-4ポート.html`: the island percent-encodes
    /// the handle the analytics state raw.
    #[test]
    fn a_japanese_handle_matches_its_percent_encoded_url() {
        let handle = "ugreen-nexode-65w-急速-充電器-4ポート";
        let url = "https://ugreen.jp/products/ugreen-nexode-65w-%E6%80%A5%E9%80%9F-%E5%85%85%E9%9B%BB%E5%99%A8-4%E3%83%9D%E3%83%BC%E3%83%88";
        let mut input = document();
        input["analytics"]["meta"]["product"]["handle"] = json!(handle);
        island_mut(&mut input)["url"] = json!(url);
        for (index, id) in [49142586999064_u64, 49154830631192].into_iter().enumerate() {
            island_mut(&mut input)["offers"][index]["url"] = json!(format!("{url}?variant={id}"));
        }

        let product = &normalized(input)["product"];
        assert_eq!(product["handle"], json!(handle));
        assert_eq!(product["url"], json!(url));
    }

    #[test]
    fn an_offer_url_must_name_a_variant_of_the_product() {
        let mut input = document();
        island_mut(&mut input)["offers"][0]["url"] =
            json!("https://ugreen.jp/products/lightning-usb-cable");

        assert!(matches!(
            normalize(input),
            Err(NormalizeError::OfferUrlWithoutVariant { .. })
        ));
    }

    #[test]
    fn the_offers_and_the_picker_must_list_the_analytics_variants() {
        let mut offers = document();
        island_mut(&mut offers)["offers"][1]["url"] =
            json!("https://ugreen.jp/products/lightning-usb-cable?variant=1");
        assert!(matches!(
            normalize(offers),
            Err(NormalizeError::VariantIdsMismatch { island, .. }) if island == "the offers"
        ));

        let mut picker = document();
        picker_variant_mut(&mut picker, 1)["id"] = json!(1);
        assert!(matches!(
            normalize(picker),
            Err(NormalizeError::VariantIdsMismatch { island, .. }) if island == "the picker"
        ));
    }

    #[test]
    fn the_islands_must_agree_on_a_variant() {
        let mut offer_price = document();
        island_mut(&mut offer_price)["offers"][0]["price"] = json!(1400.0);
        assert!(matches!(
            normalize(offer_price),
            Err(NormalizeError::VariantFieldMismatch { field, island, .. })
                if field == "price" && island == "its offer"
        ));

        let mut picker_price = document();
        picker_variant_mut(&mut picker_price, 0)["price"] = json!(140000);
        assert!(matches!(
            normalize(picker_price),
            Err(NormalizeError::VariantFieldMismatch { field, island, .. })
                if field == "price" && island == "the picker"
        ));

        let mut offer_sku = document();
        island_mut(&mut offer_sku)["offers"][0]["sku"] = json!("60522");
        assert!(matches!(
            normalize(offer_sku),
            Err(NormalizeError::VariantFieldMismatch { field, .. }) if field == "sku"
        ));

        let mut picker_sku = document();
        picker_variant_mut(&mut picker_sku, 0)["sku"] = Value::Null;
        assert!(matches!(
            normalize(picker_sku),
            Err(NormalizeError::VariantFieldMismatch { field, island, .. })
                if field == "sku" && island == "the picker"
        ));

        let mut available = document();
        picker_variant_mut(&mut available, 0)["available"] = json!(false);
        assert!(matches!(
            normalize(available),
            Err(NormalizeError::VariantFieldMismatch { field, .. }) if field == "available"
        ));
    }

    #[test]
    fn an_unknown_availability_fails() {
        let mut input = document();
        island_mut(&mut input)["offers"][0]["availability"] = json!("http://schema.org/PreOrder");

        assert!(matches!(
            normalize(input),
            Err(NormalizeError::UnknownAvailability { .. })
        ));
    }

    #[test]
    fn an_out_of_stock_offer_is_unavailable() {
        let mut input = document();
        island_mut(&mut input)["offers"][1]["availability"] = json!("http://schema.org/OutOfStock");
        picker_variant_mut(&mut input, 1)["available"] = json!(false);

        assert_eq!(
            normalized(input)["product"]["variants"][1]["available"],
            json!(false)
        );
    }

    /// The 19 saved variants without a SKU have none anywhere on the page.
    #[test]
    fn a_missing_or_empty_sku_is_none() {
        let mut missing = document();
        island_mut(&mut missing)["offers"][0]
            .as_object_mut()
            .expect("an offer")
            .remove("sku");
        analytics_variant_mut(&mut missing, 0)["sku"] = Value::Null;
        picker_variant_mut(&mut missing, 0)["sku"] = Value::Null;
        assert_eq!(
            normalized(missing)["product"]["variants"][0]["sku"],
            Value::Null
        );

        let mut empty = document();
        island_mut(&mut empty)["offers"][0]
            .as_object_mut()
            .expect("an offer")
            .remove("sku");
        analytics_variant_mut(&mut empty, 0)["sku"] = json!("");
        picker_variant_mut(&mut empty, 0)["sku"] = json!("");
        assert_eq!(
            normalized(empty)["product"]["variants"][0]["sku"],
            Value::Null
        );
    }

    #[test]
    fn a_digit_barcode_is_the_gtin_and_anything_else_is_dropped() {
        let mut input = document();
        picker_variant_mut(&mut input, 0)["barcode"] = json!(" 6957303860521\u{a0}");
        picker_variant_mut(&mut input, 1)["barcode"] = json!("60521-B");

        let variants = &normalized(input)["product"]["variants"];
        assert_eq!(variants[0]["gtin"], json!("6957303860521"));
        assert_eq!(variants[1]["gtin"], Value::Null);
    }

    #[test]
    fn a_variant_image_becomes_https() {
        let mut input = document();
        picker_variant_mut(&mut input, 0)["featured_image"] = json!({
            "id": 1, "product_id": 9456052437272_u64, "position": 1,
            "created_at": "2026-05-11T18:06:49+09:00", "updated_at": "2026-05-11T18:07:41+09:00",
            "alt": null, "width": 1200, "height": 1200,
            "src": "//ugreen.jp/cdn/shop/files/v.webp?v=1", "variant_ids": [49142586999064_u64]
        });

        assert_eq!(
            normalized(input)["product"]["variants"][0]["image"],
            json!("https://ugreen.jp/cdn/shop/files/v.webp?v=1")
        );
    }

    #[test]
    fn a_product_type_is_kept_when_set() {
        let mut input = document();
        input["analytics"]["meta"]["product"]["type"] = json!("Power Bank");

        assert_eq!(
            normalized(input)["product"]["product_type"],
            json!("Power Bank")
        );
    }
}
