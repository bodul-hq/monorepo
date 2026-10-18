//! Expresses the storefront's product facts in the system's vocabulary. The page states the
//! product across several islands - Shopify's analytics `meta`, the schema.org product island,
//! the variant picker's record of the selected variant, its legends and the gallery - and they
//! cross here as one product. Where two islands state the same fact, normalization checks that
//! they agree; what nothing needs stays behind as the commented-out exclusion records in
//! `product`.

pub mod product;

use super::raw;
use money::{Currency, Money, MoneyError};
use serde::{Deserialize, Serialize};
use std::num::ParseFloatError;
use thiserror::Error;

/// The storefront the pages are saved from.
const ORIGIN: &str = "https://www.minisforum.kr";

#[derive(Debug, Clone, PartialEq, Error)]
pub enum NormalizeError {
    #[error("the document carries {count} product ld+json islands; at most one is read")]
    MultipleProductIslands { count: usize },

    #[error("the island's offers disagree on their currency: {currencies:?}")]
    MixedOfferCurrencies { currencies: Vec<String> },

    #[error("offer currency `{currency}` is not supported: {reason}")]
    UnsupportedCurrency { currency: String, reason: String },

    #[error("variant `{variant_id}` has a price, but no product island states its currency")]
    MissingCurrency { variant_id: u64 },

    #[error("`{field}` of {cents} hundredths is not a whole number of `{currency}`")]
    CentsNotWholeUnits {
        field: String,
        cents: i64,
        currency: String,
    },

    #[error("offer price `{value}` is not a number: {source}")]
    PriceNotANumber {
        value: String,
        source: ParseFloatError,
    },

    #[error("offer price `{value}` is not money: {source:?}")]
    OfferNotMoney { value: String, source: MoneyError },

    #[error("offer URL `{url}` names no numeric `variant`")]
    OfferUrlWithoutVariant { url: String },

    #[error("offer availability `{value}` is neither schema.org InStock nor OutOfStock")]
    UnknownAvailability { value: String },

    #[error("the {island} is for product `{other}`, the analytics for `{analytics}`")]
    ProductIdMismatch {
        island: String,
        other: String,
        analytics: u64,
    },

    #[error("the variant picker's URL `{picker}` is not the product island's `{island}`")]
    ProductUrlMismatch { picker: String, island: String },

    #[error("the analytics list variants {analytics:?}, the product island {island:?}")]
    VariantIdsMismatch {
        analytics: Vec<u64>,
        island: Vec<u64>,
    },

    #[error("variant `{variant_id}` costs {analytics} hundredths in the analytics, but {other}")]
    PriceMismatch {
        variant_id: u64,
        analytics: i64,
        other: String,
    },

    #[error(
        "variant `{variant_id}` has SKU {analytics:?} in the analytics, {island:?} on the island"
    )]
    SkuMismatch {
        variant_id: u64,
        analytics: Option<String>,
        island: Option<String>,
    },

    #[error("the selected variant `{variant_id}` is not among the analytics' variants")]
    UnknownSelectedVariant { variant_id: u64 },

    #[error("variant `{variant_id}` selects {selected} option values against {offered} options")]
    OptionCountMismatch {
        variant_id: u64,
        offered: usize,
        selected: usize,
    },
}

fn currency(code: &str) -> Result<Currency, NormalizeError> {
    code.parse()
        .map_err(|reason| NormalizeError::UnsupportedCurrency {
            currency: code.to_string(),
            reason,
        })
}

// Liquid renders every amount in hundredths of the shop currency, won included: a currency whose
// minor unit is a hundredth takes them as they are, a zero-decimal one must divide evenly.
fn money_from_shopify_cents(
    cents: i64,
    currency: Currency,
    field: &str,
) -> Result<Money, NormalizeError> {
    if currency.exponent() == 2 {
        return Ok(Money::new(cents, currency));
    }
    if cents % 100 != 0 {
        return Err(NormalizeError::CentsNotWholeUnits {
            field: field.to_string(),
            cents,
            currency: currency.as_str().to_string(),
        });
    }

    Ok(Money::new(cents / 100, currency))
}

// The island states its prices in major units, as strings.
fn money_from_offer(price: &str, currency: Currency) -> Result<Money, NormalizeError> {
    let amount = price
        .parse::<f64>()
        .map_err(|source| NormalizeError::PriceNotANumber {
            value: price.to_string(),
            source,
        })?;

    Money::from_raw(amount, currency.as_str()).map_err(|source| NormalizeError::OfferNotMoney {
        value: price.to_string(),
        source,
    })
}

// Liquid gives bare numeric ids; the family identifies Shopify objects by their Admin API gid.
fn variant_gid(id: u64) -> String {
    format!("gid://shopify/ProductVariant/{id}")
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MinisforumKrNormalizedDocument {
    pub product: product::Product,
}

impl MinisforumKrNormalizedDocument {
    pub fn from_raw(
        raw: raw::MinisforumKrRawDocument,
    ) -> Result<MinisforumKrNormalizedDocument, NormalizeError> {
        Ok(MinisforumKrNormalizedDocument {
            product: product::Product::from_raw(raw)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{MinisforumKrNormalizedDocument, NormalizeError};
    use crate::minisforumkr::offer_details::v1::raw::MinisforumKrRawDocument;
    use crate::minisforumkr::offer_details::v1::raw::ldjsonitem::tests::{organization, product};
    use crate::minisforumkr::offer_details::v1::raw::tests::document;
    use html_extractor::serde_json::{self, Value, json};

    fn normalize(document: Value) -> Result<MinisforumKrNormalizedDocument, NormalizeError> {
        let raw: MinisforumKrRawDocument =
            serde_json::from_value(document).expect("the raw fixture deserializes");
        MinisforumKrNormalizedDocument::from_raw(raw)
    }

    fn normalized(document: Value) -> Value {
        serde_json::to_value(normalize(document).expect("the fixture normalizes"))
            .expect("serializes")
    }

    fn group_mut(document: &mut Value) -> &mut Value {
        &mut document["ld-json"][1]["data"]
    }

    fn meta_variants_mut(document: &mut Value) -> &mut Value {
        &mut document["analytics"]["meta"]["product"]["variants"]
    }

    /// A landing page: no product island, no picker, no gallery, and nothing for sale.
    fn landing_page() -> Value {
        let mut input = document();
        input["ld-json"] = json!([{"type": "application/ld+json", "data": organization()}]);
        let object = input.as_object_mut().expect("an object");
        object.remove("variant_picker");
        object.remove("selected_variant");
        object.insert("option_names".to_string(), json!([]));
        object.insert("images".to_string(), json!([]));
        for variant in meta_variants_mut(&mut input)
            .as_array_mut()
            .expect("variants")
        {
            variant["price"] = json!(0);
        }
        input
    }

    #[test]
    fn the_product_is_assembled_from_every_island() {
        let product = &normalized(document())["product"];

        assert_eq!(product["id"], json!("gid://shopify/Product/7128100143150"));
        assert_eq!(product["handle"], json!("minisforum-um890-pro"));
        assert_eq!(
            product["url"],
            json!("https://www.minisforum.kr/products/minisforum-um890-pro")
        );
        assert_eq!(
            product["title"],
            json!("MINISFORUM UM890 Pro 고성능 미니PC")
        );
        assert_eq!(product["description"], Value::Null);
        assert_eq!(product["category"], json!("데스크톱 컴퓨터"));
        assert_eq!(product["vendor"], json!("Minisforum KR"));
        assert_eq!(product["product_type"], json!("Mini PC"));
        assert_eq!(product["options"], json!(["CPU", "Type", "사양"]));
        assert_eq!(
            product["images"],
            json!(["https://www.minisforum.kr/cdn/shop/files/UM890Pro1.jpg?v=1776075578"])
        );
        assert_eq!(product["variants"].as_array().map(Vec::len), Some(2));
    }

    #[test]
    fn won_prices_are_hundredths_divided_out() {
        let variant = &normalized(document())["product"]["variants"][0];

        assert_eq!(
            variant["price"],
            json!({"amount": 756900, "currency": "KRW"})
        );
        assert_eq!(
            variant["selected"]["compare_at_price"],
            json!({"amount": 899990, "currency": "KRW"})
        );
    }

    #[test]
    fn hundredths_that_are_not_whole_won_fail() {
        let mut input = document();
        meta_variants_mut(&mut input)[0]["price"] = json!(75690050);

        assert!(matches!(
            normalize(input),
            Err(NormalizeError::CentsNotWholeUnits {
                cents: 75690050,
                ..
            })
        ));
    }

    #[test]
    fn the_island_and_the_analytics_must_agree_on_a_price() {
        let mut input = document();
        group_mut(&mut input)["hasVariant"][1]["offers"]["price"] = json!("1071993");

        assert!(matches!(
            normalize(input),
            Err(NormalizeError::PriceMismatch {
                variant_id: 41332824113198,
                ..
            })
        ));
    }

    #[test]
    fn the_island_and_the_analytics_must_list_the_same_variants() {
        let mut input = document();
        group_mut(&mut input)["hasVariant"][1]["offers"]["url"] =
            json!("https://www.minisforum.kr/products/minisforum-um890-pro?variant=1");

        assert!(matches!(
            normalize(input),
            Err(NormalizeError::VariantIdsMismatch { island, .. }) if island == [41332824080430, 1]
        ));
    }

    #[test]
    fn mixed_offer_currencies_fail() {
        let mut input = document();
        group_mut(&mut input)["hasVariant"][1]["offers"]["priceCurrency"] = json!("USD");

        assert!(matches!(
            normalize(input),
            Err(NormalizeError::MixedOfferCurrencies { .. })
        ));
    }

    #[test]
    fn availability_comes_from_the_island() {
        let variants = &normalized(document())["product"]["variants"];
        assert_eq!(variants[0]["available"], json!(true));
        assert_eq!(variants[1]["available"], json!(false));

        let mut input = document();
        group_mut(&mut input)["hasVariant"][0]["offers"]["availability"] =
            json!("http://schema.org/PreOrder");
        assert!(matches!(
            normalize(input),
            Err(NormalizeError::UnknownAvailability { .. })
        ));
    }

    #[test]
    fn only_the_selected_variant_carries_the_records_details() {
        let variants = &normalized(document())["product"]["variants"];

        assert_eq!(
            variants[0]["selected"]["options"],
            json!([
                {"name": "CPU", "value": "AMD Ryzen™ 9 8945HS"},
                {"name": "Type", "value": "FreeDOS"},
                {"name": "사양", "value": "EU"}
            ])
        );
        assert_eq!(variants[0]["selected"]["weight"], json!(1700));
        assert_eq!(variants[1]["selected"], Value::Null);
    }

    #[test]
    fn the_selected_variant_must_be_listed_and_fit_the_legends() {
        let mut unknown = document();
        unknown["selected_variant"]["data"]["id"] = json!(1);
        assert!(matches!(
            normalize(unknown),
            Err(NormalizeError::UnknownSelectedVariant { variant_id: 1 })
        ));

        let mut legends = document();
        legends["option_names"] = json!([{"text": "CPU:"}]);
        assert!(matches!(
            normalize(legends),
            Err(NormalizeError::OptionCountMismatch {
                offered: 1,
                selected: 3,
                ..
            })
        ));
    }

    #[test]
    fn the_islands_must_name_the_same_product() {
        let mut group = document();
        group_mut(&mut group)["productGroupID"] = json!("1");
        assert!(matches!(
            normalize(group),
            Err(NormalizeError::ProductIdMismatch { .. })
        ));

        let mut picker = document();
        picker["variant_picker"]["product_id"] = json!("1");
        assert!(matches!(
            normalize(picker),
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
    fn a_single_product_island_reads_like_a_one_variant_group() {
        let mut input = document();
        input["ld-json"][1]["data"] = product();
        let object = input.as_object_mut().expect("an object");
        object.remove("variant_picker");
        object.remove("selected_variant");
        object.insert("option_names".to_string(), json!([]));
        input["analytics"]["meta"]["product"]["id"] = json!(7216628465710_u64);
        input["analytics"]["meta"]["product"]["handle"] = json!("u-shaped-neck-pillow");
        input["analytics"]["meta"]["product"]["variants"] = json!([{
            "id": 41622560604206_u64, "price": 6800000, "name": "MINISFORUM U자형 목베개",
            "public_title": null, "sku": "U-SHAPEDP"
        }]);

        let product = &normalized(input)["product"];
        assert_eq!(product["title"], json!("MINISFORUM U자형 목베개"));
        assert_eq!(
            product["variants"][0]["price"],
            json!({"amount": 68000, "currency": "KRW"})
        );
        assert_eq!(product["variants"][0]["available"], json!(false));
        assert_eq!(product["variants"][0]["title"], Value::Null);
    }

    #[test]
    fn a_landing_page_has_no_title_and_nothing_for_sale() {
        let product = &normalized(landing_page())["product"];

        assert_eq!(product["title"], Value::Null);
        assert_eq!(product["url"], Value::Null);
        assert_eq!(product["variants"][0]["price"], Value::Null);
        assert_eq!(product["variants"][0]["available"], Value::Null);
    }

    #[test]
    fn a_price_without_a_product_island_fails() {
        let mut input = landing_page();
        meta_variants_mut(&mut input)[1]["price"] = json!(100);

        assert!(matches!(
            normalize(input),
            Err(NormalizeError::MissingCurrency {
                variant_id: 41332824113198
            })
        ));
    }

    #[test]
    fn ids_become_gids_and_island_images_lose_their_width() {
        let variant = &normalized(document())["product"]["variants"][0];

        assert_eq!(
            variant["id"],
            json!("gid://shopify/ProductVariant/41332824080430")
        );
        assert_eq!(
            variant["image"],
            json!("https://www.minisforum.kr/cdn/shop/files/UM890Pro1.jpg?v=1776075578")
        );
        assert_eq!(
            variant["url"],
            json!("https://www.minisforum.kr/products/minisforum-um890-pro?variant=41332824080430")
        );
    }
}
