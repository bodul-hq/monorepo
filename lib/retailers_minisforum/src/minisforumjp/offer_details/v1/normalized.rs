//! Expresses the storefront's product facts in the system's vocabulary. Liquid's product record
//! crosses as the product; the theme's schema.org island restates its offers, so normalization
//! checks the two agree and keeps from the island only its `category`. The reviews app's island
//! and the FAQ island carry no offer fact and are not read. What nothing needs stays behind as
//! the commented-out exclusion records in `product`.
//!
//! # Yen
//!
//! Liquid's `product | json` renders every amount in hundredths of the shop currency, yen
//! included: the record's `13039900` is ¥130,399, which the island states as `"130399"`. This
//! holds on all 277 variants of the 73 saved pages (177 island offers compared one by one, see
//! [`raw::productjson::ProductJson`]). `money` stores yen as whole yen (`JPY` has exponent 0), so
//! the record's amounts are divided by 100 and must divide exactly, as they do on every saved
//! variant; a remainder would be a price in sen, which the storefront does not sell.

mod island;
pub mod product;

use super::raw;
use money::Currency;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// The storefront the pages are saved from; the record states only the product's handle.
const ORIGIN: &str = "https://www.minisforum.jp";

#[derive(Debug, Clone, PartialEq, Error)]
pub enum NormalizeError {
    #[error("shop currency `{currency}` is not supported: {reason}")]
    UnsupportedCurrency { currency: String, reason: String },

    #[error("`{field}` of {hundredths} hundredths is not a whole number of `{currency}`")]
    HundredthsNotWholeUnits {
        field: String,
        hundredths: i64,
        currency: String,
    },

    #[error("the document carries {count} product islands; at most one is read")]
    SeveralProductIslands { count: usize },

    #[error("the island is for `{island}`, the record for `{record}`")]
    IslandProductMismatch { island: String, record: String },

    #[error("offer `{offer}` is in `{island}`, the shop in `{shop}`")]
    IslandCurrencyMismatch {
        offer: String,
        island: String,
        shop: String,
    },

    #[error("offer URL `{url}` names no numeric `variant`")]
    OfferUrlWithoutVariant { url: String },

    #[error("offer `{offer}` price `{price}` is not a decimal amount of `{currency}`")]
    OfferPriceNotDecimal {
        offer: String,
        price: String,
        currency: String,
    },

    #[error("offer `{offer}` availability `{value}` is neither schema.org InStock nor OutOfStock")]
    UnknownAvailability { offer: String, value: String },

    #[error("the island offers variants {island:?}, the record {record:?}")]
    VariantIdsMismatch { island: Vec<u64>, record: Vec<u64> },

    #[error(
        "variant `{variant_id}` `{field}` is `{island}` on the island, `{record}` on the record"
    )]
    VariantFieldMismatch {
        variant_id: u64,
        field: String,
        island: String,
        record: String,
    },

    #[error("variant `{variant_id}` selects {selected} option values against {offered} options")]
    OptionCountMismatch {
        variant_id: u64,
        offered: usize,
        selected: usize,
    },
}

// Liquid renders every amount in hundredths of the shop currency, so a currency can be read
// only if its minor unit is no finer than that: a hundredth (`AUD`) or a whole unit (`JPY`).
fn shop_currency(code: &str) -> Result<Currency, NormalizeError> {
    let currency: Currency =
        code.parse()
            .map_err(|reason| NormalizeError::UnsupportedCurrency {
                currency: code.to_string(),
                reason,
            })?;

    if currency.exponent() > 2 {
        return Err(NormalizeError::UnsupportedCurrency {
            currency: code.to_string(),
            reason: "its minor unit is finer than the hundredths Liquid renders".to_string(),
        });
    }

    Ok(currency)
}

// A record amount, in minor units of `currency`. A currency with a hundredth for a minor unit
// takes the amount as it is; a zero-decimal one (yen) must divide evenly, since its smallest
// price is one whole unit. `shop_currency` guarantees the exponent is at most 2.
fn minor_units_from_hundredths(
    hundredths: i64,
    currency: Currency,
    field: &str,
) -> Result<i64, NormalizeError> {
    let per_minor_unit = 10_i64.pow(u32::from(2 - currency.exponent()));
    if hundredths % per_minor_unit != 0 {
        return Err(NormalizeError::HundredthsNotWholeUnits {
            field: field.to_string(),
            hundredths,
            currency: currency.as_str().to_string(),
        });
    }

    Ok(hundredths / per_minor_unit)
}

fn https_from_protocol_relative(url: String) -> String {
    match url.strip_prefix("//") {
        Some(rest) => format!("https://{rest}"),
        None => url,
    }
}

// The record holds the handle as typed, with the Japanese characters of 2 saved pages (`u型枕`)
// as they are; the storefront's URL, the island's included, percent-encodes every byte outside
// the RFC 3986 unreserved set, in upper-case hex.
fn percent_encoded(handle: &str) -> String {
    handle
        .bytes()
        .map(|byte| match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' => {
                char::from(byte).to_string()
            }
            other => format!("%{other:02X}"),
        })
        .collect()
}

fn non_empty(value: String) -> Option<String> {
    (!value.is_empty()).then_some(value)
}

// Liquid gives bare numeric ids; the family identifies Shopify objects by their Admin API gid.
fn product_gid(id: u64) -> String {
    format!("gid://shopify/Product/{id}")
}

fn variant_gid(id: u64) -> String {
    format!("gid://shopify/ProductVariant/{id}")
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MinisforumJpNormalizedDocument {
    pub product: product::Product,
}

impl MinisforumJpNormalizedDocument {
    pub fn from_raw(
        raw: raw::MinisforumJpRawDocument,
    ) -> Result<MinisforumJpNormalizedDocument, NormalizeError> {
        let currency = shop_currency(&raw.paypal.currency)?;
        let island = island::Island::from_raw(raw.ld_json)?;

        Ok(MinisforumJpNormalizedDocument {
            product: product::Product::from_raw(raw.product_json.data, island, currency)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{MinisforumJpNormalizedDocument, NormalizeError, percent_encoded};
    use crate::minisforumjp::offer_details::v1::raw::MinisforumJpRawDocument;
    use crate::minisforumjp::offer_details::v1::raw::tests::document;
    use html_extractor::serde_json::{self, Value, json};

    const HANDLE: &str = "minisforum-ms-03-core-ultra-5-336h-ultra-9-386h";

    fn normalize(document: Value) -> Result<MinisforumJpNormalizedDocument, NormalizeError> {
        let raw: MinisforumJpRawDocument =
            serde_json::from_value(document).expect("the raw fixture deserializes");
        MinisforumJpNormalizedDocument::from_raw(raw)
    }

    fn normalized(document: Value) -> Value {
        serde_json::to_value(normalize(document).expect("the fixture normalizes"))
            .expect("serializes")
    }

    fn record_variant_mut(document: &mut Value, index: usize) -> &mut Value {
        &mut document["product_json"]["data"]["variants"][index]
    }

    /// The `ProductGroup` island is the second script of the fixture.
    fn island_offer_mut(document: &mut Value, index: usize) -> &mut Value {
        &mut document["ld-json"][1]["data"]["hasVariant"][index]["offers"]
    }

    /// The page as the theme renders a product without a price: the Organization alone.
    fn without_product_island(mut document: Value) -> Value {
        document["ld-json"]
            .as_array_mut()
            .expect("islands")
            .drain(1..);
        document
    }

    #[test]
    fn the_product_is_the_record_with_the_islands_category() {
        let product = &normalized(document())["product"];

        assert_eq!(product["id"], json!("gid://shopify/Product/15388676128934"));
        assert_eq!(product["handle"], json!(HANDLE));
        assert_eq!(
            product["url"],
            json!(format!("https://www.minisforum.jp/products/{HANDLE}"))
        );
        assert_eq!(
            product["title"],
            json!("MINISFORUM MS-03 ローカルAIエッジワークステーション")
        );
        assert_eq!(product["vendor"], json!("Minisforum JP"));
        assert_eq!(product["product_type"], json!("Mini WorkStation"));
        assert_eq!(product["category"], json!("デスクトップPC"));
        assert_eq!(product["options"], json!(["CPU", "TYPE"]));
        assert_eq!(
            product["images"],
            json!(["https://www.minisforum.jp/cdn/shop/files/MS-01MS-03_NO_BP.webp?v=1787303758"])
        );
        assert_eq!(product["variants"].as_array().map(Vec::len), Some(2));
    }

    /// The record's `13039900` is the island's `"130399"` yen.
    #[test]
    fn a_price_is_hundredths_of_a_yen_stored_as_whole_yen() {
        let variant = &normalized(document())["product"]["variants"][0];
        assert_eq!(
            variant["price"],
            json!({"amount": 130399, "currency": "JPY"})
        );
        assert_eq!(
            variant["compare_at_price"],
            json!({"amount": 162999, "currency": "JPY"})
        );
    }

    #[test]
    fn hundredths_that_are_not_whole_yen_fail() {
        let mut price = without_product_island(document());
        record_variant_mut(&mut price, 0)["price"] = json!(13039950);
        assert_eq!(
            normalize(price),
            Err(NormalizeError::HundredthsNotWholeUnits {
                field: "variant 56749131137190 price".to_string(),
                hundredths: 13039950,
                currency: "JPY".to_string(),
            })
        );

        let mut compare_at = without_product_island(document());
        record_variant_mut(&mut compare_at, 1)["compare_at_price"] = json!(30499901);
        assert!(matches!(
            normalize(compare_at),
            Err(NormalizeError::HundredthsNotWholeUnits { field, .. })
                if field == "variant 56749137494182 compare_at_price"
        ));
    }

    /// With the island present, the record's price is read the same way before the comparison.
    #[test]
    fn hundredths_that_are_not_whole_yen_fail_before_the_island_comparison() {
        let mut input = document();
        record_variant_mut(&mut input, 0)["price"] = json!(13039950);

        assert!(matches!(
            normalize(input),
            Err(NormalizeError::HundredthsNotWholeUnits { field, .. })
                if field == "variant 56749131137190 price"
        ));
    }

    #[test]
    fn a_shop_currency_with_hundredths_takes_the_record_as_it_is() {
        let mut input = document();
        input["paypal"]["data-currency"] = json!("USD");
        for index in 0..2 {
            let offer = island_offer_mut(&mut input, index);
            offer["priceCurrency"] = json!("USD");
            let price = offer["price"].as_str().expect("a price").to_string();
            offer["price"] = json!(format!("{price}.00"));
        }

        let variant = &normalized(input)["product"]["variants"][0];
        assert_eq!(
            variant["price"],
            json!({"amount": 13039900, "currency": "USD"})
        );
    }

    #[test]
    fn an_unsupported_shop_currency_fails() {
        let mut input = document();
        input["paypal"]["data-currency"] = json!("XYZ");

        assert!(matches!(
            normalize(input),
            Err(NormalizeError::UnsupportedCurrency { currency, .. }) if currency == "XYZ"
        ));
    }

    /// As on `products-minisforum-all-flash-nas-s5-intel-core-3-304.html`, announced at ¥0.
    #[test]
    fn a_zero_price_is_no_price() {
        let mut input = without_product_island(document());
        record_variant_mut(&mut input, 0)["price"] = json!(0);
        record_variant_mut(&mut input, 0)["compare_at_price"] = json!(0);

        let variants = &normalized(input)["product"]["variants"];
        assert_eq!(variants[0]["price"], Value::Null);
        assert_eq!(variants[0]["compare_at_price"], Value::Null);
        assert_eq!(variants[1]["price"]["amount"], json!(249999));
    }

    /// As on `products-minisforum-n5-pro-nas.html`, whose marked-down variants carry a
    /// `compare_at_price` equal to the price.
    #[test]
    fn a_compare_at_price_that_is_not_higher_is_kept_as_stated() {
        let mut input = document();
        record_variant_mut(&mut input, 0)["compare_at_price"] = json!(13039900);

        let variant = &normalized(input)["product"]["variants"][0];
        assert_eq!(variant["compare_at_price"]["amount"], json!(130399));
    }

    #[test]
    fn a_page_without_a_product_island_has_no_category() {
        let product = &normalized(without_product_island(document()))["product"];

        assert_eq!(product["category"], Value::Null);
        assert_eq!(product["variants"].as_array().map(Vec::len), Some(2));
    }

    /// The reviews app's island is neither read nor required: removing it changes nothing, and
    /// a second one (`products-minisforum-ai-x1-pro-470.html` has two) is no conflict.
    #[test]
    fn the_reviews_apps_island_is_not_read() {
        let expected = normalized(document());

        let mut without = document();
        without["ld-json"].as_array_mut().expect("islands").pop();
        assert_eq!(normalized(without), expected);

        let mut twice = document();
        let reviews = twice["ld-json"][2].clone();
        twice["ld-json"]
            .as_array_mut()
            .expect("islands")
            .push(reviews);
        assert_eq!(normalized(twice), expected);
    }

    #[test]
    fn a_faq_island_is_no_product_island() {
        let mut input = document();
        input["ld-json"]
            .as_array_mut()
            .expect("islands")
            .push(json!({
                "type": "application/ld+json",
                "data": crate::minisforumjp::offer_details::v1::raw::ldjson::tests::faq_page()
            }));

        assert_eq!(normalized(input), normalized(document()));
    }

    #[test]
    fn a_single_variant_product_island_is_read() {
        let mut input = document();
        let record = &mut input["product_json"]["data"];
        record["variants"]
            .as_array_mut()
            .expect("variants")
            .truncate(1);
        let group = input["ld-json"][1]["data"].clone();
        input["ld-json"][1]["data"] = json!({
            "@context": "http://schema.org/",
            "@id": format!("/products/{HANDLE}#product"),
            "@type": "Product",
            "brand": group["brand"],
            "category": "デスクトップPC",
            "description": "",
            "image": group["hasVariant"][0]["image"],
            "name": group["name"],
            "offers": group["hasVariant"][0]["offers"],
            "sku": "MS-03-33600JP",
            "url": group["url"]
        });

        let product = &normalized(input.clone())["product"];
        assert_eq!(product["category"], json!("デスクトップPC"));
        assert_eq!(product["variants"].as_array().map(Vec::len), Some(1));

        // As on `products-exclusive-use-link.html`, whose island states an empty category.
        input["ld-json"][1]["data"]["category"] = json!("");
        assert_eq!(normalized(input)["product"]["category"], Value::Null);
    }

    /// As on `products-minisforum-u型枕.html`: the record's handle is raw, the island's URL is
    /// percent-encoded.
    #[test]
    fn a_handle_with_japanese_characters_is_percent_encoded_in_the_url_only() {
        let mut input = document();
        input["product_json"]["data"]["handle"] = json!("minisforum-u型枕");
        input["ld-json"][1]["data"]["url"] =
            json!("https://www.minisforum.jp/products/minisforum-u%E5%9E%8B%E6%9E%95");

        let product = &normalized(input)["product"];
        assert_eq!(product["handle"], json!("minisforum-u型枕"));
        assert_eq!(
            product["url"],
            json!("https://www.minisforum.jp/products/minisforum-u%E5%9E%8B%E6%9E%95")
        );
    }

    #[test]
    fn a_handle_is_percent_encoded_outside_the_unreserved_characters() {
        assert_eq!(percent_encoded("m1-lite_2.0~x"), "m1-lite_2.0~x");
        assert_eq!(
            percent_encoded("minisforum-収納ポーチ"),
            "minisforum-%E5%8F%8E%E7%B4%8D%E3%83%9D%E3%83%BC%E3%83%81"
        );
        assert_eq!(percent_encoded("a b"), "a%20b");
    }

    #[test]
    fn an_island_in_another_currency_fails() {
        let mut input = document();
        island_offer_mut(&mut input, 1)["priceCurrency"] = json!("USD");

        assert!(matches!(
            normalize(input),
            Err(NormalizeError::IslandCurrencyMismatch { island, .. }) if island == "USD"
        ));
    }

    #[test]
    fn an_island_for_another_product_fails() {
        let mut input = document();
        input["ld-json"][1]["data"]["productGroupID"] = json!("1");

        assert!(matches!(
            normalize(input),
            Err(NormalizeError::IslandProductMismatch { .. })
        ));

        let mut url = document();
        url["ld-json"][1]["data"]["url"] = json!("https://www.minisforum.jp/products/other");
        assert!(matches!(
            normalize(url),
            Err(NormalizeError::IslandProductMismatch { .. })
        ));
    }

    #[test]
    fn the_island_and_the_record_must_offer_the_same_variants() {
        let mut input = document();
        island_offer_mut(&mut input, 1)["url"] = json!(format!(
            "https://www.minisforum.jp/products/{HANDLE}?variant=1"
        ));

        assert!(matches!(
            normalize(input),
            Err(NormalizeError::VariantIdsMismatch { island, .. })
                if island == [56749131137190, 1]
        ));
    }

    #[test]
    fn the_island_and_the_record_must_agree_on_a_variant() {
        let mut price = document();
        island_offer_mut(&mut price, 0)["price"] = json!("130400");
        assert!(matches!(
            normalize(price),
            Err(NormalizeError::VariantFieldMismatch { field, island, record, .. })
                if field == "price" && island == "130400" && record == "130399"
        ));

        let mut available = document();
        island_offer_mut(&mut available, 0)["availability"] = json!("http://schema.org/OutOfStock");
        assert!(matches!(
            normalize(available),
            Err(NormalizeError::VariantFieldMismatch { field, .. }) if field == "available"
        ));

        let mut sku = document();
        sku["ld-json"][1]["data"]["hasVariant"][0]
            .as_object_mut()
            .expect("a variant")
            .remove("sku");
        assert!(matches!(
            normalize(sku),
            Err(NormalizeError::VariantFieldMismatch { field, .. }) if field == "sku"
        ));
    }

    #[test]
    fn an_offer_price_that_is_not_whole_yen_fails() {
        for price in ["130399.00", "130,399", "130.399", "", "-1"] {
            let mut input = document();
            island_offer_mut(&mut input, 0)["price"] = json!(price);

            assert!(
                matches!(
                    normalize(input),
                    Err(NormalizeError::OfferPriceNotDecimal { price: found, .. }) if found == price
                ),
                "{price}"
            );
        }
    }

    #[test]
    fn an_offer_without_a_variant_in_its_url_fails() {
        let mut input = document();
        island_offer_mut(&mut input, 0)["url"] =
            json!(format!("https://www.minisforum.jp/products/{HANDLE}"));

        assert!(matches!(
            normalize(input),
            Err(NormalizeError::OfferUrlWithoutVariant { .. })
        ));
    }

    #[test]
    fn an_availability_other_than_in_or_out_of_stock_fails() {
        let mut input = document();
        island_offer_mut(&mut input, 0)["availability"] = json!("http://schema.org/PreOrder");

        assert!(matches!(
            normalize(input),
            Err(NormalizeError::UnknownAvailability { value, .. })
                if value == "http://schema.org/PreOrder"
        ));
    }

    /// As on `products-minisforum-ai-x1-refurbished.html`, whose variants have neither.
    #[test]
    fn a_missing_island_sku_matches_a_missing_record_sku() {
        let mut input = document();
        input["ld-json"][1]["data"]["hasVariant"][0]
            .as_object_mut()
            .expect("a variant")
            .remove("sku");
        record_variant_mut(&mut input, 0)["sku"] = Value::Null;

        assert_eq!(
            normalized(input)["product"]["variants"][0]["sku"],
            Value::Null
        );
    }

    #[test]
    fn a_missing_island_sku_matches_an_empty_record_sku() {
        let mut input = document();
        input["ld-json"][1]["data"]["hasVariant"][0]
            .as_object_mut()
            .expect("a variant")
            .remove("sku");
        record_variant_mut(&mut input, 0)["sku"] = json!("");

        assert_eq!(
            normalized(input)["product"]["variants"][0]["sku"],
            Value::Null
        );
    }

    /// As on `products-shipping-protection.html`, whose 98 variants all carry one SKU.
    #[test]
    fn variants_sharing_a_sku_are_kept() {
        let mut input = without_product_island(document());
        record_variant_mut(&mut input, 1)["sku"] = json!("MS-03-33600JP");

        let variants = &normalized(input)["product"]["variants"];
        assert_eq!(variants[0]["sku"], variants[1]["sku"]);
    }

    #[test]
    fn option_values_pair_with_the_products_names_by_position() {
        let output = normalized(document());
        assert_eq!(
            output["product"]["variants"][1]["options"],
            json!([
                {"name": "CPU", "value": "Intel® Core™ Ultra 5 336H"},
                {"name": "TYPE", "value": "32GB RAM + 1TB SSD"}
            ])
        );

        let mut input = document();
        input["product_json"]["data"]["options"] = json!(["CPU"]);
        assert!(matches!(
            normalize(input),
            Err(NormalizeError::OptionCountMismatch {
                offered: 1,
                selected: 2,
                ..
            })
        ));
    }

    #[test]
    fn several_product_islands_fail() {
        let mut input = document();
        let island = input["ld-json"][1].clone();
        input["ld-json"]
            .as_array_mut()
            .expect("islands")
            .push(island);

        assert!(matches!(
            normalize(input),
            Err(NormalizeError::SeveralProductIslands { count: 2 })
        ));
    }

    #[test]
    fn ids_become_gids_and_a_variant_image_becomes_https() {
        let mut input = document();
        record_variant_mut(&mut input, 0)["featured_image"] = json!({
            "id": 1, "product_id": 15388676128934_u64, "position": 1,
            "created_at": "2026-08-21T17:15:56+08:00", "updated_at": "2026-08-21T17:15:58+08:00",
            "alt": null, "width": 1600, "height": 1600,
            "src": "//www.minisforum.jp/cdn/shop/files/v.jpg?v=1", "variant_ids": [56749131137190_u64]
        });

        let variant = &normalized(input)["product"]["variants"][0];
        assert_eq!(
            variant["id"],
            json!("gid://shopify/ProductVariant/56749131137190")
        );
        assert_eq!(
            variant["image"],
            json!("https://www.minisforum.jp/cdn/shop/files/v.jpg?v=1")
        );
    }

    /// The record's videos are neither images nor read: a page with one normalizes the same.
    #[test]
    fn a_video_among_the_media_is_not_an_image() {
        let input = document();
        assert_eq!(
            input["product_json"]["data"]["media"][1]["media_type"],
            json!("external_video")
        );

        let product = &normalized(input)["product"];
        assert_eq!(product["images"].as_array().map(Vec::len), Some(1));
    }
}
