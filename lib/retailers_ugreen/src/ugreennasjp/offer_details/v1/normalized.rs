//! Expresses the storefront's product facts in the system's vocabulary. The theme's schema.org
//! `Product` island carries the offers, Shopify's analytics `meta` the ids, and judge.me's
//! `Product` island the rating; they cross here as one product. Where two islands state the same
//! fact, normalization checks that they agree; what nothing needs stays behind as the
//! commented-out exclusion records in `product`.

pub mod product;

use super::raw;
use money::{Currency, Money};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Error)]
pub enum NormalizeError {
    #[error("no theme `Product` ld+json island on the document")]
    MissingProductIsland,

    #[error("{count} theme `Product` ld+json islands on the document")]
    MultipleProductIslands { count: usize },

    #[error("{count} judge.me `Product` ld+json islands on the document")]
    MultipleReviewIslands { count: usize },

    #[error("`{field}` is in `{currency}`; only JPY is read")]
    UnsupportedCurrency { field: String, currency: String },

    #[error("`{field}` of {cents} hundredths is not a whole number of yen")]
    CentsNotWholeYen { field: String, cents: i64 },

    #[error("`{field}` of {amount} is not a whole number of yen")]
    AmountNotWholeYen { field: String, amount: f64 },

    #[error("offer URL `{url}` is not the product URL with a numeric `variant`")]
    OfferUrlWithoutVariant { url: String },

    #[error("the island's URL `{url}` is not the analytics handle's `{handle}`")]
    ProductUrlMismatch { url: String, handle: String },

    #[error("the analytics list variants {analytics:?}, the island's offers {offers:?}")]
    VariantIdsMismatch {
        analytics: Vec<u64>,
        offers: Vec<u64>,
    },

    #[error("variant `{variant_id}` costs {analytics} yen in the analytics, {offer} in its offer")]
    PriceMismatch {
        variant_id: u64,
        analytics: i64,
        offer: i64,
    },

    #[error(
        "variant `{variant_id}` has SKU {analytics:?} in the analytics, `{offer}` in its offer"
    )]
    SkuMismatch {
        variant_id: u64,
        analytics: Option<String>,
        offer: String,
    },

    #[error("offer availability `{value}` is neither in nor out of stock")]
    UnknownAvailability { value: String },

    #[error("rating `{value}` is not a number")]
    RatingNotANumber { value: String },
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

// The island states yen as a float, such as `2996.0`.
fn yen(amount: f64, currency: &str, field: &str) -> Result<Money, NormalizeError> {
    let currency = yen_currency(currency, field)?;
    if amount.fract() != 0.0 {
        return Err(NormalizeError::AmountNotWholeYen {
            field: field.to_string(),
            amount,
        });
    }

    Ok(Money::new(amount as i64, currency))
}

// Liquid renders every amount in hundredths of the shop currency, yen included.
fn whole_yen_from_shopify_cents(cents: i64, field: &str) -> Result<i64, NormalizeError> {
    if cents % 100 != 0 {
        return Err(NormalizeError::CentsNotWholeYen {
            field: field.to_string(),
            cents,
        });
    }

    Ok(cents / 100)
}

// The family identifies Shopify objects by their Admin API gid.
fn variant_gid(id: u64) -> String {
    format!("gid://shopify/ProductVariant/{id}")
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UgreenNasJpNormalizedDocument {
    pub product: product::Product,
}

impl UgreenNasJpNormalizedDocument {
    pub fn from_raw(
        raw: raw::UgreenNasJpRawDocument,
    ) -> Result<UgreenNasJpNormalizedDocument, NormalizeError> {
        Ok(UgreenNasJpNormalizedDocument {
            product: product::Product::from_raw(raw)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{NormalizeError, UgreenNasJpNormalizedDocument};
    use crate::ugreennasjp::offer_details::v1::raw::UgreenNasJpRawDocument;
    use crate::ugreennasjp::offer_details::v1::raw::tests::document;
    use html_extractor::serde_json::{self, Value, json};

    fn normalize(document: Value) -> Result<UgreenNasJpNormalizedDocument, NormalizeError> {
        let raw: UgreenNasJpRawDocument =
            serde_json::from_value(document).expect("the raw fixture deserializes");
        UgreenNasJpNormalizedDocument::from_raw(raw)
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

    #[test]
    fn the_product_is_assembled_from_every_island() {
        let product = &normalized(document())["product"];

        assert_eq!(product["id"], json!("gid://shopify/Product/15994687127633"));
        assert_eq!(product["handle"], json!("air-45w-usb-c-charger"));
        assert_eq!(
            product["url"],
            json!("https://store.ugreen.jp/products/air-45w-usb-c-charger")
        );
        assert_eq!(
            product["title"],
            json!("UGREEN Air 45W USB-C 当社最小充電器 GaNII 急速充電器")
        );
        assert_eq!(product["vendor"], json!("UGREEN-JP"));
        assert_eq!(product["product_type"], json!("Charger"));
        assert_eq!(
            product["images"],
            json!(["https://store.ugreen.jp/cdn/shop/files/519suz8uUOL.png?v=1788427694"])
        );
        assert_eq!(product["rating"], json!({"value": 4.67, "count": 3}));
    }

    #[test]
    fn the_description_keeps_its_text_lines_only() {
        assert_eq!(
            normalized(document())["product"]["description"],
            json!("なぜ「Air」なのか。\nそれは、空気のように薄く、軽いから。")
        );
    }

    #[test]
    fn variants_join_the_analytics_and_the_offers() {
        let variants = &normalized(document())["product"]["variants"];

        assert_eq!(
            variants[0],
            json!({
                "id": "gid://shopify/ProductVariant/60268391956561",
                "title": "グレー",
                "name": "UGREEN Air 45W USB-C 当社最小充電器 GaNII 急速充電器 - グレー",
                "sku": "35993",
                "gtin": "6941876268399",
                "price": {"amount": 2996, "currency": "JPY"},
                "available": true,
                "url": "https://store.ugreen.jp/products/air-45w-usb-c-charger?variant=60268391956561"
            })
        );
        assert_eq!(variants[1]["gtin"], Value::Null);
        assert_eq!(variants[1]["available"], json!(false));
    }

    #[test]
    fn a_page_without_reviews_has_no_rating() {
        let mut input = document();
        input["ld-json"].as_array_mut().expect("items").truncate(2);

        assert_eq!(normalized(input)["product"]["rating"], Value::Null);
    }

    #[test]
    fn a_page_without_a_theme_product_island_fails() {
        let mut input = document();
        input["ld-json"].as_array_mut().expect("items").remove(1);

        assert!(matches!(
            normalize(input),
            Err(NormalizeError::MissingProductIsland)
        ));
    }

    #[test]
    fn a_currency_other_than_yen_fails() {
        let mut input = document();
        island_mut(&mut input)["offers"][0]["priceCurrency"] = json!("USD");

        assert!(matches!(
            normalize(input),
            Err(NormalizeError::UnsupportedCurrency { currency, .. }) if currency == "USD"
        ));
    }

    #[test]
    fn hundredths_that_are_not_whole_yen_fail() {
        let mut input = document();
        analytics_variant_mut(&mut input, 0)["price"] = json!(299650);

        assert!(matches!(
            normalize(input),
            Err(NormalizeError::CentsNotWholeYen { cents: 299650, .. })
        ));
    }

    #[test]
    fn the_analytics_and_the_offer_must_agree_on_the_price() {
        let mut input = document();
        analytics_variant_mut(&mut input, 1)["price"] = json!(199600);

        assert!(matches!(
            normalize(input),
            Err(NormalizeError::PriceMismatch {
                analytics: 1996,
                offer: 2996,
                ..
            })
        ));
    }

    #[test]
    fn the_analytics_and_the_offer_must_agree_on_the_sku() {
        let mut input = document();
        analytics_variant_mut(&mut input, 0)["sku"] = json!("other");

        assert!(matches!(
            normalize(input),
            Err(NormalizeError::SkuMismatch { offer, .. }) if offer == "35993"
        ));
    }

    #[test]
    fn the_analytics_and_the_offers_must_list_the_same_variants() {
        let mut input = document();
        analytics_variant_mut(&mut input, 1)["id"] = json!(1);

        assert!(matches!(
            normalize(input),
            Err(NormalizeError::VariantIdsMismatch { analytics, .. })
                if analytics == [60268391956561, 1]
        ));
    }

    #[test]
    fn an_offer_url_without_a_variant_fails() {
        let mut input = document();
        island_mut(&mut input)["offers"][0]["url"] =
            json!("https://store.ugreen.jp/products/air-45w-usb-c-charger");

        assert!(matches!(
            normalize(input),
            Err(NormalizeError::OfferUrlWithoutVariant { .. })
        ));
    }

    #[test]
    fn the_island_url_must_be_the_handles() {
        let mut input = document();
        input["analytics"]["meta"]["product"]["handle"] = json!("other");

        assert!(matches!(
            normalize(input),
            Err(NormalizeError::ProductUrlMismatch { .. })
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
    fn an_empty_product_type_is_absent() {
        let mut input = document();
        input["analytics"]["meta"]["product"]["type"] = json!("");

        assert_eq!(normalized(input)["product"]["product_type"], Value::Null);
    }
}
