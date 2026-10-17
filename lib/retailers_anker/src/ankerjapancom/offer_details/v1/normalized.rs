//! Expresses the storefront's product facts in the system's vocabulary. The schema.org island and
//! Liquid's product record both cross, because choosing between them is structuralization's job;
//! what nothing needs stays behind as the commented-out exclusion records below.

pub mod ldjson;
pub mod record;

use super::raw;
use chrono::{DateTime, Utc};
use money::{Currency, Money};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Error)]
pub enum NormalizeError {
    #[error("the `Product` island carries no offer, so the page states no currency")]
    MissingOfferCurrency,

    #[error("the island's offers disagree on their currency: {currencies:?}")]
    MixedOfferCurrencies { currencies: Vec<String> },

    #[error("`{field}` is in `{currency}`; only JPY is read")]
    UnsupportedCurrency { field: String, currency: String },

    #[error("`{field}` of {cents} hundredths is not a whole number of yen")]
    CentsNotWholeYen { field: String, cents: i64 },

    #[error("`{field}` of {amount} is not a whole number of yen")]
    AmountNotWholeYen { field: String, amount: f64 },

    #[error("`{field}` `{value}` is not an RFC 3339 instant: {source}")]
    NotAnInstant {
        field: String,
        value: String,
        source: chrono::ParseError,
    },

    #[error("review `datePublished` `{value}` is not a calendar date: {source}")]
    NotADate {
        value: String,
        source: chrono::ParseError,
    },

    #[error("offer URL `{url}` names no numeric `variant`")]
    OfferUrlWithoutVariant { url: String },

    #[error("variant `{variant_id}` selects {selected} option values against {offered} options")]
    OptionCountMismatch {
        variant_id: u64,
        offered: usize,
        selected: usize,
    },
}

// `Currency::exponent` is unimplemented, so only the zero-decimal yen has a known minor unit here;
// `Money::from_raw` would store it multiplied by a hundred.
fn yen_currency(code: &str, field: &str) -> Result<Currency, NormalizeError> {
    match code {
        "JPY" => Ok(Currency::JPY),
        other => Err(NormalizeError::UnsupportedCurrency {
            field: field.to_string(),
            currency: other.to_string(),
        }),
    }
}

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
fn yen_from_shopify_cents(
    cents: i64,
    currency: Currency,
    field: &str,
) -> Result<Money, NormalizeError> {
    if cents % 100 != 0 {
        return Err(NormalizeError::CentsNotWholeYen {
            field: field.to_string(),
            cents,
        });
    }

    Ok(Money::new(cents / 100, currency))
}

fn instant(raw: &str, field: &str) -> Result<DateTime<Utc>, NormalizeError> {
    let parsed =
        DateTime::parse_from_rfc3339(raw).map_err(|source| NormalizeError::NotAnInstant {
            field: field.to_string(),
            value: raw.to_string(),
            source,
        })?;

    Ok(parsed.with_timezone(&Utc))
}

fn https_from_protocol_relative(url: String) -> String {
    match url.strip_prefix("//") {
        Some(rest) => format!("https://{rest}"),
        None => url,
    }
}

// Liquid gives bare numeric ids; the family identifies Shopify objects by their Admin API gid.
fn product_gid(id: u64) -> String {
    format!("gid://shopify/Product/{id}")
}

fn variant_gid(id: u64) -> String {
    format!("gid://shopify/ProductVariant/{id}")
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AnkerJapanComNormalizedDocument {
    pub ld_json: ldjson::LdJson,
    pub product: record::ProductRecord,
}

impl AnkerJapanComNormalizedDocument {
    pub fn from_raw(
        raw: raw::AnkerJapanComRawDocument,
    ) -> Result<AnkerJapanComNormalizedDocument, NormalizeError> {
        let ld_json = ldjson::LdJson::from_raw(raw.ld_json)?;
        // Offers state the currency when present. The older PowerBag pages omit Product
        // JSON-LD; their og:price:currency and Shopify.currency still identify this JPY shop.
        let currency = ld_json
            .product
            .as_ref()
            .map_or(Currency::JPY, |p| p.currency);
        let product = record::ProductRecord::from_raw(raw.product_json.data, currency)?;

        Ok(AnkerJapanComNormalizedDocument { ld_json, product })
    }
}

#[cfg(test)]
mod tests {
    use super::{AnkerJapanComNormalizedDocument, NormalizeError};
    use crate::ankerjapancom::offer_details::v1::raw::AnkerJapanComRawDocument;
    use crate::ankerjapancom::offer_details::v1::raw::productjson::tests::PRODUCT;
    use html_extractor::serde_json::{self, Value, json};

    fn island(currency: &str) -> Value {
        json!({"data": {"@type": "Product", "@context": "http://schema.org/",
            "name": "【公式】Anker Hub", "description": "d",
            "image": ["https://www.ankerjapan.com/cdn/shop/products/a.jpg?v=1&width=1500"],
            "url": "https://www.ankerjapan.com/products/hub",
            "brand": {"@type": "Brand", "name": "Anker"},
            "offers": [{"@type": "Offer", "availability": "http://schema.org/OutOfStock",
                "price": 6199.0, "priceCurrency": currency, "sku": "68ANHUB-B14JA",
                "url": "https://www.ankerjapan.com/products/hub?variant=40082645057697",
                "hasMerchantReturnPolicy": {"@type": "MerchantReturnPolicy",
                    "applicableCountry": "JP", "returnPolicyCountry": "JP",
                    "returnPolicyCategory": "https://schema.org/MerchantReturnFiniteReturnWindow",
                    "merchantReturnDays": 30, "returnMethod": "https://schema.org/ReturnByMail",
                    "returnFees": "https://schema.org/FreeReturn"},
                "shippingDetails": {"@type": "OfferShippingDetails", "shippingRate":
                    {"@type": "MonetaryAmount", "value": 540, "currency": "JPY"}}}]}})
    }

    fn document() -> Value {
        json!({
            "ld-json": [island("JPY")],
            "product_json": {"data": serde_json::from_str::<Value>(PRODUCT).expect("fixture")}
        })
    }

    fn normalize(document: Value) -> Result<AnkerJapanComNormalizedDocument, NormalizeError> {
        let raw: AnkerJapanComRawDocument =
            serde_json::from_value(document).expect("the raw fixture deserializes");
        AnkerJapanComNormalizedDocument::from_raw(raw)
    }

    fn normalized(document: Value) -> Value {
        serde_json::to_value(normalize(document).expect("the fixture normalizes"))
            .expect("serializes")
    }

    fn variant_mut(document: &mut Value) -> &mut Value {
        &mut document["product_json"]["data"]["variants"][0]
    }

    #[test]
    fn a_record_price_is_hundredths_of_a_yen_in_the_islands_currency() {
        let output = normalized(document());

        assert_eq!(
            output["product"]["variants"][0]["price"],
            json!({"amount": 6199, "currency": "JPY"})
        );
        assert_eq!(output["ld_json"]["product"]["currency"], json!("JPY"));
    }

    #[test]
    fn hundredths_that_are_not_whole_yen_fail() {
        let mut input = document();
        variant_mut(&mut input)["price"] = json!(619950);

        assert!(matches!(
            normalize(input),
            Err(NormalizeError::CentsNotWholeYen { cents: 619950, .. })
        ));
    }

    #[test]
    fn a_currency_other_than_yen_fails() {
        let mut input = document();
        input["ld-json"] = json!([island("USD")]);

        assert!(matches!(
            normalize(input),
            Err(NormalizeError::UnsupportedCurrency { currency, .. }) if currency == "USD"
        ));
    }

    #[test]
    fn a_zero_price_is_no_price_while_a_sibling_keeps_its_own() {
        let mut input = document();
        let mut sibling = variant_mut(&mut input).clone();
        sibling["id"] = json!(2);
        sibling["price"] = json!(0);
        input["product_json"]["data"]["variants"]
            .as_array_mut()
            .expect("variants")
            .push(sibling);

        let output = normalized(input);
        assert_eq!(
            output["product"]["variants"][0]["price"]["amount"],
            json!(6199)
        );
        assert_eq!(output["product"]["variants"][1]["price"], Value::Null);
    }

    #[test]
    fn an_empty_or_null_sku_is_absent() {
        for sku in [json!(""), Value::Null] {
            let mut input = document();
            variant_mut(&mut input)["sku"] = sku.clone();

            assert_eq!(
                normalized(input)["product"]["variants"][0]["sku"],
                Value::Null,
                "sku {sku}"
            );
        }
    }

    #[test]
    fn a_barcode_crosses_verbatim_unless_empty() {
        let mut junk = document();
        variant_mut(&mut junk)["barcode"] = json!("a1233,");
        assert_eq!(
            normalized(junk)["product"]["variants"][0]["barcode"],
            json!("a1233,")
        );

        let mut empty = document();
        variant_mut(&mut empty)["barcode"] = json!("");
        assert_eq!(
            normalized(empty)["product"]["variants"][0]["barcode"],
            Value::Null
        );
    }

    #[test]
    fn an_unweighed_variant_has_no_weight() {
        let output = normalized(document());
        assert_eq!(output["product"]["variants"][0]["weight"], Value::Null);

        let mut weighed = document();
        variant_mut(&mut weighed)["weight"] = json!(26);
        assert_eq!(
            normalized(weighed)["product"]["variants"][0]["weight"],
            json!(26)
        );
    }

    #[test]
    fn a_page_without_a_product_island_keeps_its_liquid_record_in_yen() {
        let mut input = document();
        input["ld-json"] = json!([]);

        let output = normalized(input);
        assert!(output["ld_json"]["product"].is_null());
        assert_eq!(output["product"]["variants"][0]["price"]["currency"], "JPY");
        assert_eq!(output["product"]["variants"][0]["price"]["amount"], 6199);
    }

    #[test]
    fn option_values_pair_with_the_products_names_by_position() {
        let output = normalized(document());
        assert_eq!(
            output["product"]["variants"][0]["options"],
            json!([{"name": "カラー", "value": "ブラック"}])
        );

        let mut input = document();
        variant_mut(&mut input)["options"] = json!(["ブラック", "USB-C"]);
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
    fn ids_become_gids_and_images_become_https() {
        let output = normalized(document());
        let product = &output["product"];

        assert_eq!(product["id"], json!("gid://shopify/Product/1"));
        assert_eq!(
            product["variants"][0]["id"],
            json!("gid://shopify/ProductVariant/40082645057697")
        );
        assert_eq!(
            product["images"],
            json!(["https://www.ankerjapan.com/cdn/shop/products/a.jpg?v=1"])
        );
        assert_eq!(
            output["ld_json"]["product"]["offers"][0]["variant_id"],
            json!("gid://shopify/ProductVariant/40082645057697")
        );
    }
}
