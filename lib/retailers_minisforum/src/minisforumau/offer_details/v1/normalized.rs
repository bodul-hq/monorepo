//! Expresses the storefront's product facts in the system's vocabulary. Liquid's product record
//! crosses as the product; the schema.org island restates its offers, so normalization checks
//! the two agree and keeps from the island only its `category`. What nothing needs stays behind
//! as the commented-out exclusion records in `product`.

mod island;
pub mod product;

use super::raw;
use money::{Currency, Money};
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// The storefront the pages are saved from; the record states only the product's handle.
const ORIGIN: &str = "https://au.minisforum.com";

#[derive(Debug, Clone, PartialEq, Error)]
pub enum NormalizeError {
    #[error("shop currency `{currency}` is not supported: {reason}")]
    UnsupportedCurrency { currency: String, reason: String },

    #[error("shop currency `{currency}` has no hundredths, the unit Liquid renders amounts in")]
    CurrencyWithoutHundredths { currency: String },

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

    #[error("offer `{offer}` price `{price}` is not a decimal amount")]
    OfferPriceNotDecimal { offer: String, price: String },

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
pub struct MinisforumAuNormalizedDocument {
    pub product: product::Product,
}

impl MinisforumAuNormalizedDocument {
    pub fn from_raw(
        raw: raw::MinisforumAuRawDocument,
    ) -> Result<MinisforumAuNormalizedDocument, NormalizeError> {
        let currency = shop_currency(&raw.paypal.currency)?;
        let island = island::Island::from_raw(raw.ld_json)?;

        Ok(MinisforumAuNormalizedDocument {
            product: product::Product::from_raw(raw.product_json.data, island, currency)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{MinisforumAuNormalizedDocument, NormalizeError};
    use crate::minisforumau::offer_details::v1::raw::MinisforumAuRawDocument;
    use crate::minisforumau::offer_details::v1::raw::tests::document;
    use html_extractor::serde_json::{self, Value, json};

    fn normalize(document: Value) -> Result<MinisforumAuNormalizedDocument, NormalizeError> {
        let raw: MinisforumAuRawDocument =
            serde_json::from_value(document).expect("the raw fixture deserializes");
        MinisforumAuNormalizedDocument::from_raw(raw)
    }

    fn normalized(document: Value) -> Value {
        serde_json::to_value(normalize(document).expect("the fixture normalizes"))
            .expect("serializes")
    }

    fn record_variant_mut(document: &mut Value, index: usize) -> &mut Value {
        &mut document["product_json"]["data"]["variants"][index]
    }

    fn island_offer_mut(document: &mut Value, index: usize) -> &mut Value {
        &mut document["ld-json"][1]["data"]["hasVariant"][index]["offers"]
    }

    fn without_product_island(mut document: Value) -> Value {
        document["ld-json"]
            .as_array_mut()
            .expect("islands")
            .truncate(1);
        document
    }

    #[test]
    fn the_product_is_the_record_with_the_islands_category() {
        let product = &normalized(document())["product"];

        assert_eq!(product["id"], json!("gid://shopify/Product/8313879462003"));
        assert_eq!(product["handle"], json!("minisforum-ms-03-workstation"));
        assert_eq!(
            product["url"],
            json!("https://au.minisforum.com/products/minisforum-ms-03-workstation")
        );
        assert_eq!(product["title"], json!("MINISFORUM MS-03"));
        assert_eq!(product["vendor"], json!("Au Minisforum"));
        assert_eq!(product["product_type"], json!("Work Station"));
        assert_eq!(product["category"], json!("Desktop Computers"));
        assert_eq!(product["options"], json!(["CPU", "Type"]));
        assert_eq!(
            product["images"],
            json!(["https://au.minisforum.com/cdn/shop/files/MinisforumMS-03.png?v=1784715204"])
        );
        assert_eq!(product["variants"].as_array().map(Vec::len), Some(2));
    }

    #[test]
    fn a_price_is_hundredths_of_the_shop_currency() {
        let variant = &normalized(document())["product"]["variants"][0];
        assert_eq!(
            variant["price"],
            json!({"amount": 114290, "currency": "AUD"})
        );
        assert_eq!(
            variant["compare_at_price"],
            json!({"amount": 142890, "currency": "AUD"})
        );
    }

    #[test]
    fn a_zero_price_is_no_price() {
        let mut input = without_product_island(document());
        record_variant_mut(&mut input, 0)["price"] = json!(0);
        record_variant_mut(&mut input, 0)["compare_at_price"] = json!(0);

        let variants = &normalized(input)["product"]["variants"];
        assert_eq!(variants[0]["price"], Value::Null);
        assert_eq!(variants[0]["compare_at_price"], Value::Null);
        assert_eq!(variants[1]["price"]["amount"], json!(202290));
    }

    #[test]
    fn a_page_without_a_product_island_has_no_category() {
        let product = &normalized(without_product_island(document()))["product"];

        assert_eq!(product["category"], Value::Null);
        assert_eq!(product["variants"].as_array().map(Vec::len), Some(2));
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
            "@id": "/products/minisforum-ms-03-workstation#product",
            "@type": "Product",
            "brand": group["brand"],
            "category": "Desktop Computers",
            "description": "",
            "image": group["hasVariant"][0]["image"],
            "name": group["name"],
            "offers": group["hasVariant"][0]["offers"],
            "sku": "MS-03-33600AU",
            "url": group["url"]
        });

        let product = &normalized(input)["product"];
        assert_eq!(product["category"], json!("Desktop Computers"));
        assert_eq!(product["variants"].as_array().map(Vec::len), Some(1));
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
    fn an_island_in_another_currency_fails() {
        let mut input = document();
        island_offer_mut(&mut input, 1)["priceCurrency"] = json!("NZD");

        assert!(matches!(
            normalize(input),
            Err(NormalizeError::IslandCurrencyMismatch { island, .. }) if island == "NZD"
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
    }

    #[test]
    fn the_island_and_the_record_must_offer_the_same_variants() {
        let mut input = document();
        island_offer_mut(&mut input, 1)["url"] =
            json!("https://au.minisforum.com/products/minisforum-ms-03-workstation?variant=1");

        assert!(matches!(
            normalize(input),
            Err(NormalizeError::VariantIdsMismatch { island, .. })
                if island == [45188354441331, 1]
        ));
    }

    #[test]
    fn the_island_and_the_record_must_agree_on_a_variant() {
        let mut price = document();
        island_offer_mut(&mut price, 0)["price"] = json!("1143.00");
        assert!(matches!(
            normalize(price),
            Err(NormalizeError::VariantFieldMismatch { field, .. }) if field == "price"
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

    /// As on `products-minisforum-n5-max-ai-nas.html`, whose second variant has neither.
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

    #[test]
    fn option_values_pair_with_the_products_names_by_position() {
        let output = normalized(document());
        assert_eq!(
            output["product"]["variants"][0]["options"],
            json!([
                {"name": "CPU", "value": "Intel® Core™ Ultra 5 336H"},
                {"name": "Type", "value": "Barebone"}
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
            "id": 1, "product_id": 8313879462003_u64, "position": 1,
            "created_at": "2026-07-08T11:12:15+08:00", "updated_at": "2026-07-08T11:12:15+08:00",
            "alt": null, "width": 1600, "height": 1600,
            "src": "//au.minisforum.com/cdn/shop/files/v.jpg?v=1", "variant_ids": [45188354441331_u64]
        });

        let variant = &normalized(input)["product"]["variants"][0];
        assert_eq!(
            variant["id"],
            json!("gid://shopify/ProductVariant/45188354441331")
        );
        assert_eq!(
            variant["image"],
            json!("https://au.minisforum.com/cdn/shop/files/v.jpg?v=1")
        );
    }
}
