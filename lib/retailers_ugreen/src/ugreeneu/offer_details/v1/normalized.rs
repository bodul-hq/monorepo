//! Expresses the storefront's product facts in the system's vocabulary. The ld+json and
//! Shopify's analytics `meta` each list every variant; normalization checks they agree one for
//! one and crosses them as one product, with the selected variant's displayed prices from the
//! main section's price block, checked against the picker's record where the page renders one.
//! What nothing needs stays behind as the commented-out exclusion records in `product`.
//!
//! Every locale prefix is its own document, tagged with its locale and carrying its own URL.

mod island;
mod page_url;
pub mod product;

use super::raw;
use money::{Currency, Money};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Error)]
pub enum NormalizeError {
    #[error("shop currency `{currency}` is not supported: {reason}")]
    UnsupportedCurrency { currency: String, reason: String },

    #[error("shop currency `{currency}` has no hundredths, the unit Liquid renders amounts in")]
    CurrencyWithoutHundredths { currency: String },

    #[error("`{field}` `{value}` is not a whole number")]
    NotAWholeNumber { field: String, value: String },

    #[error("the document carries no Product or ProductGroup island")]
    MissingProductIsland,

    #[error("the document carries {count} product islands; one is read")]
    SeveralProductIslands { count: usize },

    #[error("offer URL `{url}` names no numeric `variant`")]
    OfferUrlWithoutVariant { url: String },

    #[error("offer `{offer}` price `{price}` is not a decimal amount")]
    OfferPriceNotDecimal { offer: String, price: String },

    #[error("offer `{offer}` availability `{availability}` is neither in nor out of stock")]
    UnknownAvailability { offer: String, availability: String },

    #[error("offer `{offer}` is in `{island}`, the shop in `{shop}`")]
    OfferCurrencyMismatch {
        offer: String,
        island: String,
        shop: String,
    },

    #[error("product URL `{url}` is not `https://eu.ugreen.com[/<locale>]/products/<handle>`")]
    UnexpectedProductUrl { url: String },

    #[error("product URL `{url}` is not for the analytics handle `{handle}`")]
    ProductUrlMismatch { url: String, handle: String },

    #[error("the {island} is for product `{id}`, the analytics for `{analytics}`")]
    ProductIdMismatch {
        island: String,
        id: String,
        analytics: u64,
    },

    #[error("the ld+json lists {island} variants, the analytics {analytics}")]
    VariantCountMismatch { island: usize, analytics: usize },

    #[error(
        "offer `{offer}` `{field}` is `{island}` on the ld+json, `{analytics}` on the analytics"
    )]
    VariantMismatch {
        offer: String,
        field: String,
        island: String,
        analytics: String,
    },

    #[error("the selected variant `{id}` is not among the analytics variants")]
    SelectedVariantNotListed { id: u64 },

    #[error("a ProductGroup page renders no picker record of the selected variant")]
    PickerRecordMissing,

    #[error("a single Product page renders a picker record")]
    UnexpectedPickerRecord,

    #[error("the picker's record of the selected variant disagrees with the page on `{field}`")]
    SelectedVariantMismatch { field: String },

    #[error("the displayed price {displayed} is above the variant's price {listed}")]
    DisplayedPriceAboveListed { displayed: i64, listed: i64 },

    #[error("the struck-through price {compare} is below the variant's price {listed}")]
    StruckPriceBelowListed { compare: i64, listed: i64 },
}

// Liquid renders every amount in hundredths of the shop currency, so only a currency whose minor
// unit is a hundredth takes them as they are. Every page prices in EUR.
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
pub struct UgreenEuNormalizedDocument {
    pub product: product::Product,
}

impl UgreenEuNormalizedDocument {
    pub fn from_raw(
        raw: raw::UgreenEuRawDocument,
    ) -> Result<UgreenEuNormalizedDocument, NormalizeError> {
        Ok(UgreenEuNormalizedDocument {
            product: product::Product::from_raw(raw)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{NormalizeError, UgreenEuNormalizedDocument};
    use crate::ugreeneu::offer_details::v1::raw::UgreenEuRawDocument;
    use crate::ugreeneu::offer_details::v1::raw::tests::{document, single_variant_document};
    use html_extractor::serde_json::{self, Value, json};

    fn normalize(document: Value) -> Result<UgreenEuNormalizedDocument, NormalizeError> {
        let raw: UgreenEuRawDocument =
            serde_json::from_value(document).expect("the raw fixture deserializes");
        UgreenEuNormalizedDocument::from_raw(raw)
    }

    fn normalized(document: Value) -> Value {
        serde_json::to_value(normalize(document).expect("the fixture normalizes"))
            .expect("serializes")
    }

    fn eur(amount: i64) -> Value {
        json!({"amount": amount, "currency": "EUR"})
    }

    fn island_mut(document: &mut Value) -> &mut Value {
        &mut document["ld-json"][1]["data"]
    }

    fn offer_mut(document: &mut Value, index: usize) -> &mut Value {
        &mut document["ld-json"][1]["data"]["hasVariant"][index]["offers"]
    }

    fn analytics_variant_mut(document: &mut Value, index: usize) -> &mut Value {
        &mut document["analytics"]["meta"]["product"]["variants"][index]
    }

    /// The single-variant fixture with the automatic discount: 41.99 shown against 55.99.
    fn discounted_document() -> Value {
        let mut input = single_variant_document();
        input["price"]["base_price_cents"] = json!("4199");
        input["price"]["base_compare_cents"] = json!("5599");
        input
    }

    #[test]
    fn the_product_is_assembled_from_every_island() {
        let product = &normalized(document())["product"];

        assert_eq!(product["id"], json!("gid://shopify/Product/8625471816028"));
        assert_eq!(product["handle"], json!("pb561"));
        assert_eq!(
            product["url"],
            json!("https://eu.ugreen.com/de-at/products/pb561")
        );
        assert_eq!(product["locale"], json!("de-at"));
        assert_eq!(
            product["title"],
            json!("UGREEN Kabelloses Magnetisches Powerbank 10000mAh")
        );
        assert_eq!(product["vendor"], json!("UGREEN EU"));
        assert_eq!(product["product_type"], json!("Angebot"));
        assert_eq!(product["category"], json!("Powerbanks"));
        assert_eq!(
            product["images"],
            json!([
                "https://eu.ugreen.com/cdn/shop/files/ugreen-wireless-powerbank-10000mah-magnetische-power-bank-451511.png?v=1719994480",
                "https://eu.ugreen.com/cdn/shop/products/ugreen-wireless-powerbank-10000mah-magnetische-power-bank-685866.png?v=1719994480"
            ])
        );
        assert_eq!(
            product["selected_variant_id"],
            json!("gid://shopify/ProductVariant/48981450031452")
        );
    }

    #[test]
    fn every_variant_carries_its_own_facts_and_only_the_selected_one_a_compare_at_price() {
        let variants = &normalized(document())["product"]["variants"];

        assert_eq!(
            variants,
            &json!([
                {
                    "id": "gid://shopify/ProductVariant/48981450031452",
                    "title": "Schwarz",
                    "name": "UGREEN Kabelloses Magnetisches Powerbank 10000mAh - Schwarz",
                    "sku": "25919",
                    "gtin": "6941876229192",
                    "price": eur(5599),
                    "compare_at_price": eur(6999),
                    "automatic_discount": false,
                    "available": true,
                    "image": "https://eu.ugreen.com/cdn/shop/files/ugreen-wireless-powerbank-10000mah-magnetische-power-bank-451511.png?v=1719994480"
                },
                {
                    "id": "gid://shopify/ProductVariant/48981449998684",
                    "title": "Weiß",
                    "name": "UGREEN Kabelloses Magnetisches Powerbank 10000mAh - Weiß",
                    "sku": "25208",
                    "gtin": null,
                    "price": eur(4999),
                    "compare_at_price": null,
                    "automatic_discount": false,
                    "available": true,
                    "image": "https://eu.ugreen.com/cdn/shop/products/ugreen-wireless-powerbank-10000mah-magnetische-power-bank-685866.png?v=1719994480"
                }
            ])
        );
    }

    #[test]
    fn a_single_variant_page_is_read_without_a_picker() {
        let product = &normalized(single_variant_document())["product"];

        assert_eq!(product["variants"].as_array().map(Vec::len), Some(1));
        assert_eq!(product["variants"][0]["compare_at_price"], eur(6999));
        assert_eq!(product["variants"][0]["gtin"], json!("6941876229192"));
    }

    #[test]
    fn the_root_page_has_no_locale() {
        let mut input = document();
        island_mut(&mut input)["url"] = json!("https://eu.ugreen.com/products/pb561");

        let product = &normalized(input)["product"];
        assert_eq!(product["locale"], Value::Null);
        assert_eq!(
            product["url"],
            json!("https://eu.ugreen.com/products/pb561")
        );
    }

    #[test]
    fn an_escaped_url_matches_the_decoded_handle() {
        let mut input = document();
        island_mut(&mut input)["url"] =
            json!("https://eu.ugreen.com/bg-bg/products/%D0%BA%D1%83%D1%82%D0%B8%D1%8F");
        input["analytics"]["meta"]["product"]["handle"] = json!("кутия");

        let product = &normalized(input)["product"];
        assert_eq!(product["handle"], json!("кутия"));
        assert_eq!(
            product["url"],
            json!("https://eu.ugreen.com/bg-bg/products/%D0%BA%D1%83%D1%82%D0%B8%D1%8F")
        );
        assert_eq!(product["locale"], json!("bg-bg"));
    }

    #[test]
    fn the_title_and_names_are_trimmed() {
        let mut input = document();
        island_mut(&mut input)["name"] = json!("Adaptateur Ethernet RJ45 ");
        analytics_variant_mut(&mut input, 1)["name"] = json!("Adaptateur - Blanc ");

        let product = &normalized(input)["product"];
        assert_eq!(product["title"], json!("Adaptateur Ethernet RJ45"));
        assert_eq!(product["variants"][1]["name"], json!("Adaptateur - Blanc"));
    }

    #[test]
    fn an_automatic_discount_is_the_price_with_the_shopify_price_struck_through() {
        let variant = &normalized(discounted_document())["product"]["variants"][0];

        assert_eq!(variant["price"], eur(4199));
        assert_eq!(variant["compare_at_price"], eur(5599));
        assert_eq!(variant["automatic_discount"], json!(true));
    }

    /// As on the 4 `en-fi`, `en-hr` and `hr-hr` power bank pages.
    #[test]
    fn an_automatic_discount_keeps_the_shopify_price_over_a_higher_struck_price() {
        let mut input = discounted_document();
        input["price"]["base_compare_cents"] = json!("6999");

        let variant = &normalized(input)["product"]["variants"][0];
        assert_eq!(variant["price"], eur(4199));
        assert_eq!(variant["compare_at_price"], eur(5599));
    }

    #[test]
    fn only_the_selected_variant_takes_the_discount() {
        let mut input = document();
        input["price"]["base_price_cents"] = json!("4199");
        input["price"]["base_compare_cents"] = json!("5599");
        input["selected_variant"]["data"]["compare_at_price"] = json!(5599);

        let variants = &normalized(input)["product"]["variants"];
        assert_eq!(variants[0]["price"], eur(4199));
        assert_eq!(variants[0]["compare_at_price"], eur(5599));
        assert_eq!(variants[1]["price"], eur(4999));
        assert_eq!(variants[1]["automatic_discount"], json!(false));
    }

    #[test]
    fn a_struck_through_price_equal_to_the_price_is_no_markdown() {
        let mut input = single_variant_document();
        input["price"]["base_compare_cents"] = json!("5599");

        assert_eq!(
            normalized(input)["product"]["variants"][0]["compare_at_price"],
            Value::Null
        );
    }

    #[test]
    fn the_selected_variant_need_not_be_the_first() {
        let mut input = document();
        input["selected_variant_id"]["value"] = json!("48981449998684");
        input["price"]["base_price_cents"] = json!("4999");
        input["price"]["base_compare_cents"] = json!("4999");
        let record = &mut input["selected_variant"]["data"];
        record["id"] = json!(48981449998684_u64);
        record["price"] = json!(4999);
        record["sku"] = json!("25208");
        record["barcode"] = Value::Null;
        record["compare_at_price"] = Value::Null;

        let product = &normalized(input)["product"];
        assert_eq!(
            product["selected_variant_id"],
            json!("gid://shopify/ProductVariant/48981449998684")
        );
        assert_eq!(product["variants"][0]["compare_at_price"], Value::Null);
    }

    #[test]
    fn an_out_of_stock_offer_is_unavailable() {
        let mut input = document();
        offer_mut(&mut input, 1)["availability"] = json!("http://schema.org/OutOfStock");

        assert_eq!(
            normalized(input)["product"]["variants"][1]["available"],
            json!(false)
        );
    }

    #[test]
    fn a_barcode_that_is_not_all_digits_is_no_gtin() {
        let mut input = single_variant_document();
        island_mut(&mut input)["gtin"] = json!("N/A");

        assert_eq!(
            normalized(input)["product"]["variants"][0]["gtin"],
            Value::Null
        );
    }

    #[test]
    fn empty_sku_type_and_category_are_absent() {
        let mut input = single_variant_document();
        input["analytics"]["meta"]["product"]["type"] = json!("");
        island_mut(&mut input)["category"] = json!("");
        island_mut(&mut input)["sku"] = json!("");
        analytics_variant_mut(&mut input, 0)["sku"] = json!("");

        let product = &normalized(input)["product"];
        assert_eq!(product["product_type"], Value::Null);
        assert_eq!(product["category"], Value::Null);
        assert_eq!(product["variants"][0]["sku"], Value::Null);
    }

    #[test]
    fn a_zero_price_is_no_price() {
        let mut input = single_variant_document();
        island_mut(&mut input)["offers"]["price"] = json!("0.00");
        analytics_variant_mut(&mut input, 0)["price"] = json!(0);
        input["price"]["base_price_cents"] = json!("0");
        input["price"]["base_compare_cents"] = json!("0");

        assert_eq!(
            normalized(input)["product"]["variants"][0]["price"],
            Value::Null
        );
    }

    #[test]
    fn an_unknown_or_cent_less_currency_fails() {
        let mut unknown = document();
        unknown["price"]["currency"] = json!("XYZ");
        assert!(matches!(
            normalize(unknown),
            Err(NormalizeError::UnsupportedCurrency { currency, .. }) if currency == "XYZ"
        ));

        let mut yen = document();
        yen["price"]["currency"] = json!("JPY");
        assert!(matches!(
            normalize(yen),
            Err(NormalizeError::CurrencyWithoutHundredths { currency }) if currency == "JPY"
        ));
    }

    #[test]
    fn an_offer_in_another_currency_fails() {
        let mut input = document();
        offer_mut(&mut input, 1)["priceCurrency"] = json!("SEK");

        assert!(matches!(
            normalize(input),
            Err(NormalizeError::OfferCurrencyMismatch { island, .. }) if island == "SEK"
        ));
    }

    #[test]
    fn a_price_attribute_that_is_not_a_number_fails() {
        let mut input = document();
        input["price"]["base_price_cents"] = json!("55,99");

        assert!(matches!(
            normalize(input),
            Err(NormalizeError::NotAWholeNumber { field, .. }) if field == "base_price_cents"
        ));

        let mut selected = document();
        selected["selected_variant_id"]["value"] = json!("");
        assert!(matches!(
            normalize(selected),
            Err(NormalizeError::NotAWholeNumber { field, .. }) if field == "selected_variant_id"
        ));
    }

    #[test]
    fn the_product_island_must_be_there_once() {
        let mut missing = document();
        missing["ld-json"]
            .as_array_mut()
            .expect("islands")
            .truncate(1);
        assert!(matches!(
            normalize(missing),
            Err(NormalizeError::MissingProductIsland)
        ));

        let mut several = document();
        let island = several["ld-json"][1].clone();
        several["ld-json"]
            .as_array_mut()
            .expect("islands")
            .push(island);
        assert!(matches!(
            normalize(several),
            Err(NormalizeError::SeveralProductIslands { count: 2 })
        ));
    }

    #[test]
    fn an_unreadable_offer_fails() {
        let mut url = document();
        offer_mut(&mut url, 0)["url"] = json!("https://eu.ugreen.com/de-at/products/pb561");
        assert!(matches!(
            normalize(url),
            Err(NormalizeError::OfferUrlWithoutVariant { .. })
        ));

        let mut price = document();
        offer_mut(&mut price, 0)["price"] = json!("55,99");
        assert!(matches!(
            normalize(price),
            Err(NormalizeError::OfferPriceNotDecimal { price, .. }) if price == "55,99"
        ));

        let mut availability = document();
        offer_mut(&mut availability, 0)["availability"] = json!("http://schema.org/PreOrder");
        assert!(matches!(
            normalize(availability),
            Err(NormalizeError::UnknownAvailability { .. })
        ));
    }

    #[test]
    fn the_product_url_must_be_the_analytics_product_on_the_storefront() {
        let mut origin = document();
        island_mut(&mut origin)["url"] = json!("https://de.ugreen.com/products/pb561");
        assert!(matches!(
            normalize(origin),
            Err(NormalizeError::UnexpectedProductUrl { .. })
        ));

        let mut handle = document();
        island_mut(&mut handle)["url"] = json!("https://eu.ugreen.com/de-at/products/other");
        assert!(matches!(
            normalize(handle),
            Err(NormalizeError::ProductUrlMismatch { handle, .. }) if handle == "pb561"
        ));
    }

    #[test]
    fn the_islands_must_name_the_analytics_product() {
        let mut group = document();
        island_mut(&mut group)["productGroupID"] = json!("1");
        assert!(matches!(
            normalize(group),
            Err(NormalizeError::ProductIdMismatch { island, .. }) if island == "ld+json"
        ));

        let mut block = document();
        block["price"]["product_id"] = json!("1");
        assert!(matches!(
            normalize(block),
            Err(NormalizeError::ProductIdMismatch { island, .. }) if island == "price block"
        ));
    }

    #[test]
    fn the_ld_json_and_the_analytics_must_list_the_same_variants() {
        let mut count = document();
        island_mut(&mut count)["hasVariant"]
            .as_array_mut()
            .expect("variants")
            .truncate(1);
        assert!(matches!(
            normalize(count),
            Err(NormalizeError::VariantCountMismatch {
                island: 1,
                analytics: 2
            })
        ));

        let mut order = document();
        island_mut(&mut order)["hasVariant"]
            .as_array_mut()
            .expect("variants")
            .reverse();
        assert!(matches!(
            normalize(order),
            Err(NormalizeError::VariantMismatch { field, .. }) if field == "id"
        ));

        let mut price = document();
        offer_mut(&mut price, 1)["price"] = json!("50.00");
        assert!(matches!(
            normalize(price),
            Err(NormalizeError::VariantMismatch { field, .. }) if field == "price"
        ));

        let mut sku = document();
        island_mut(&mut sku)["hasVariant"][1]["sku"] = json!("25209");
        assert!(matches!(
            normalize(sku),
            Err(NormalizeError::VariantMismatch { field, .. }) if field == "sku"
        ));
    }

    #[test]
    fn the_selected_variant_must_be_listed() {
        let mut input = document();
        input["selected_variant_id"]["value"] = json!("1");

        assert!(matches!(
            normalize(input),
            Err(NormalizeError::SelectedVariantNotListed { id: 1 })
        ));
    }

    #[test]
    fn the_picker_record_goes_with_a_product_group() {
        let mut missing = document();
        missing
            .as_object_mut()
            .expect("an object")
            .remove("selected_variant");
        assert!(matches!(
            normalize(missing),
            Err(NormalizeError::PickerRecordMissing)
        ));

        let mut unexpected = single_variant_document();
        unexpected["selected_variant"] = document()["selected_variant"].clone();
        assert!(matches!(
            normalize(unexpected),
            Err(NormalizeError::UnexpectedPickerRecord)
        ));
    }

    #[test]
    fn the_picker_record_must_agree_with_the_page() {
        for (field, value) in [
            ("id", json!(48981449998684_u64)),
            ("price", json!(5500)),
            ("sku", json!("25208")),
            ("available", json!(false)),
            ("barcode", json!("6941876229193")),
            ("compare_at_price", json!(7999)),
        ] {
            let mut input = document();
            input["selected_variant"]["data"][field] = value;

            assert!(
                matches!(
                    normalize(input),
                    Err(NormalizeError::SelectedVariantMismatch { field: f }) if f == field
                ),
                "{field}"
            );
        }
    }

    #[test]
    fn a_displayed_price_above_or_a_struck_price_below_the_variant_price_fails() {
        let mut above = single_variant_document();
        above["price"]["base_price_cents"] = json!("6000");
        assert!(matches!(
            normalize(above),
            Err(NormalizeError::DisplayedPriceAboveListed {
                displayed: 6000,
                listed: 5599
            })
        ));

        let mut below = single_variant_document();
        below["price"]["base_compare_cents"] = json!("5000");
        assert!(matches!(
            normalize(below),
            Err(NormalizeError::StruckPriceBelowListed {
                compare: 5000,
                listed: 5599
            })
        ));
    }
}
