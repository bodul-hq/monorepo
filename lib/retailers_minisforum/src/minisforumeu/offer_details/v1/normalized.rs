//! Expresses the storefront's product facts in the system's vocabulary. Liquid's product record
//! crosses as the product; the theme's schema.org island restates its offers, so normalization
//! checks the two agree and keeps from the island only its `category`. What nothing needs stays
//! behind as the commented-out exclusion records in `product`.
//!
//! The page's other islands (the merchant feed app's `Product`, the SEO app's `@graph`, the
//! `Organization` and the reviews app's `Product`) are neither checked nor kept. They are typed
//! in `raw`, so that a change in their shape fails the page, but each one restates the record or
//! says nothing about the offer: the feed's price is of one selected variant and malformed on 18
//! of the 274 saved pages, the graph's prices are floats, and the reviews carry no variants.
//!
//! The storefront serves every product in two locales, `/products/<handle>` and
//! `/de/products/<handle>`, from one record. The PayPal metadata tag names the page's locale, and
//! with it the URL prefix; see [`LOCALES`].

mod island;
pub mod product;

use super::raw;
use money::{Currency, Money};
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// The storefront the pages are saved from; the record states only the product's handle.
const ORIGIN: &str = "https://minisforumpc.eu";

/// The storefront's locales as the PayPal metadata tag names them, with the path prefix the
/// locale's pages have: `en_US` is the unprefixed default (137 saved pages), `de_DE` is under
/// `/de` (the other 137). The saved file names carry the same split (`de-products-*.html`).
const LOCALES: [(&str, &str); 2] = [("en_US", ""), ("de_DE", "/de")];

#[derive(Debug, Clone, PartialEq, Error)]
pub enum NormalizeError {
    #[error("shop currency `{currency}` is not supported: {reason}")]
    UnsupportedCurrency { currency: String, reason: String },

    #[error("shop currency `{currency}` has no hundredths, the unit Liquid renders amounts in")]
    CurrencyWithoutHundredths { currency: String },

    #[error("page locale `{locale}` is not one of the storefront's locales")]
    UnsupportedLocale { locale: String },

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

// The path prefix of the pages in `locale`, the PayPal tag's spelling of it.
fn locale_prefix(locale: &str) -> Result<&'static str, NormalizeError> {
    LOCALES
        .iter()
        .find(|(name, _)| *name == locale)
        .map(|(_, prefix)| *prefix)
        .ok_or_else(|| NormalizeError::UnsupportedLocale {
            locale: locale.to_string(),
        })
}

// Handles can be any text (`...-ryzen™-5-...` is one on the saved pages), and the storefront and
// its sitemaps spell the URL's path percent-encoded, in upper-case hex.
fn product_url(prefix: &str, handle: &str) -> String {
    let mut segment = String::with_capacity(handle.len());
    for byte in handle.bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'.' | b'_' | b'~') {
            segment.push(char::from(byte));
        } else {
            segment.push_str(&format!("%{byte:02X}"));
        }
    }

    format!("{ORIGIN}{prefix}/products/{segment}")
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
pub struct MinisforumEuNormalizedDocument {
    pub product: product::Product,
}

impl MinisforumEuNormalizedDocument {
    pub fn from_raw(
        raw: raw::MinisforumEuRawDocument,
    ) -> Result<MinisforumEuNormalizedDocument, NormalizeError> {
        let currency = shop_currency(&raw.paypal.currency)?;
        let prefix = locale_prefix(&raw.paypal.locale)?;
        let island = island::Island::from_raw(raw.ld_json)?;

        Ok(MinisforumEuNormalizedDocument {
            product: product::Product::from_raw(raw.product_json.data, island, currency, prefix)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{MinisforumEuNormalizedDocument, NormalizeError, locale_prefix, product_url};
    use crate::minisforumeu::offer_details::v1::raw::MinisforumEuRawDocument;
    use crate::minisforumeu::offer_details::v1::raw::tests::{document, reviews_script};
    use html_extractor::serde_json::{self, Value, json};

    /// Where the theme's product island sits in `document()`.
    const ISLAND: usize = 3;

    fn normalize(document: Value) -> Result<MinisforumEuNormalizedDocument, NormalizeError> {
        let raw: MinisforumEuRawDocument =
            serde_json::from_value(document).expect("the raw fixture deserializes");
        MinisforumEuNormalizedDocument::from_raw(raw)
    }

    fn normalized(document: Value) -> Value {
        serde_json::to_value(normalize(document).expect("the fixture normalizes"))
            .expect("serializes")
    }

    fn record_variant_mut(document: &mut Value, index: usize) -> &mut Value {
        &mut document["product_json"]["data"]["variants"][index]
    }

    fn island_mut(document: &mut Value) -> &mut Value {
        &mut document["ld-json"][ISLAND]["data"]
    }

    fn island_offer_mut(document: &mut Value, index: usize) -> &mut Value {
        &mut island_mut(document)["hasVariant"][index]["offers"]
    }

    fn without_product_island(mut document: Value) -> Value {
        document["ld-json"].as_array_mut().expect("islands").pop();
        document
    }

    #[test]
    fn the_product_is_the_record_with_the_islands_category() {
        let product = &normalized(document())["product"];

        assert_eq!(product["id"], json!("gid://shopify/Product/14959267250542"));
        assert_eq!(product["handle"], json!("minisforum-u700-refurbished"));
        assert_eq!(
            product["url"],
            json!("https://minisforumpc.eu/products/minisforum-u700-refurbished")
        );
        assert_eq!(product["title"], json!("MINISFORUM U700 Refurbished"));
        assert_eq!(product["vendor"], json!("Refurbished Mini PC"));
        assert_eq!(product["product_type"], json!("Refurbished"));
        assert_eq!(product["category"], json!("Hard Drives"));
        assert_eq!(product["options"], json!(["CPU", "Type"]));
        assert_eq!(
            product["images"],
            json!(["https://minisforumpc.eu/cdn/shop/files/image13.png?v=1770796122"])
        );
        assert_eq!(product["variants"].as_array().map(Vec::len), Some(2));
    }

    #[test]
    fn a_price_is_hundredths_of_the_shop_currency() {
        let variant = &normalized(document())["product"]["variants"][0];
        assert_eq!(
            variant["price"],
            json!({"amount": 12900, "currency": "EUR"})
        );
        assert_eq!(
            variant["compare_at_price"],
            json!({"amount": 19900, "currency": "EUR"})
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
        assert_eq!(variants[1]["price"]["amount"], json!(9900));
    }

    #[test]
    fn a_list_price_is_kept_only_above_the_price() {
        let mut input = document();
        record_variant_mut(&mut input, 0)["compare_at_price"] = Value::Null;
        // As on `products-minisforum-ms-a1.html`, whose list price is under its price.
        record_variant_mut(&mut input, 1)["compare_at_price"] = json!(5900);
        let variants = &normalized(input)["product"]["variants"];
        assert_eq!(variants[0]["compare_at_price"], Value::Null);
        assert_eq!(variants[1]["compare_at_price"], Value::Null);

        let mut equal = document();
        record_variant_mut(&mut equal, 1)["compare_at_price"] = json!(9900);
        assert_eq!(
            normalized(equal)["product"]["variants"][1]["compare_at_price"],
            Value::Null
        );
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
        *island_mut(&mut input) =
            crate::minisforumeu::offer_details::v1::raw::ldjson::tests::product();

        let product = &normalized(input)["product"];
        assert_eq!(product["category"], json!("Hard Drives"));
        assert_eq!(product["variants"].as_array().map(Vec::len), Some(1));
    }

    #[test]
    fn the_other_islands_are_not_read() {
        let mut input = document();
        // The feed app's malformed price, as on `products-minisforum-ms-s1-max-mini-pc.html`,
        // and a reviews island that restates the product at a price the record does not have.
        input["ld-json"][0]["data"]["offers"]["price"] = json!("4.159.00");
        input["ld-json"][0]["data"]["offers"]["priceCurrency"] = json!("USD");
        let mut reviews = reviews_script();
        reviews["data"]["offers"]["price"] = json!("1.00");
        input["ld-json"]
            .as_array_mut()
            .expect("islands")
            .push(reviews);

        assert_eq!(normalized(input), normalized(document()));
    }

    #[test]
    fn a_german_page_is_under_de_and_a_handle_is_percent_encoded() {
        let mut input = document();
        input["paypal"]["data-locale"] = json!("de_DE");
        let record = &mut input["product_json"]["data"];
        record["handle"] = json!("minisforum-ai-nas-n5-air-amd-ryzen\u{2122}-5-bay-nas");
        island_mut(&mut input)["url"] = json!(
            "https://minisforumpc.eu/de/products/minisforum-ai-nas-n5-air-amd-ryzen%E2%84%A2-5-bay-nas"
        );
        for index in 0..2 {
            let id = [53296398172526_u64, 53753234751854][index];
            island_offer_mut(&mut input, index)["url"] = json!(format!(
                "https://minisforumpc.eu/de/products/minisforum-ai-nas-n5-air-amd-ryzen%E2%84%A2-5-bay-nas?variant={id}"
            ));
        }

        let product = &normalized(input)["product"];
        assert_eq!(
            product["url"],
            json!(
                "https://minisforumpc.eu/de/products/minisforum-ai-nas-n5-air-amd-ryzen%E2%84%A2-5-bay-nas"
            )
        );
        assert_eq!(
            product["handle"],
            json!("minisforum-ai-nas-n5-air-amd-ryzen\u{2122}-5-bay-nas")
        );
    }

    /// As on `de-products-kabeltasche.html`, a `de_DE` page whose island names the unprefixed
    /// URL: the page's URL still follows its locale.
    #[test]
    fn an_island_may_name_the_other_locales_url() {
        let mut input = document();
        input["paypal"]["data-locale"] = json!("de_DE");

        assert_eq!(
            normalized(input)["product"]["url"],
            json!("https://minisforumpc.eu/de/products/minisforum-u700-refurbished")
        );
    }

    #[test]
    fn an_island_under_an_unknown_prefix_fails() {
        let mut input = document();
        island_mut(&mut input)["url"] =
            json!("https://minisforumpc.eu/fr/products/minisforum-u700-refurbished");

        assert!(matches!(
            normalize(input),
            Err(NormalizeError::IslandProductMismatch { .. })
        ));
    }

    #[test]
    fn the_locale_decides_the_prefix() {
        assert_eq!(locale_prefix("en_US"), Ok(""));
        assert_eq!(locale_prefix("de_DE"), Ok("/de"));
        assert_eq!(
            locale_prefix("fr_FR"),
            Err(NormalizeError::UnsupportedLocale {
                locale: "fr_FR".to_string()
            })
        );
    }

    #[test]
    fn a_url_keeps_plain_handles_and_encodes_the_rest() {
        assert_eq!(
            product_url("", "a-b.c_d~e1"),
            "https://minisforumpc.eu/products/a-b.c_d~e1"
        );
        assert_eq!(
            product_url("/de", "a b™/"),
            "https://minisforumpc.eu/de/products/a%20b%E2%84%A2%2F"
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
        island_mut(&mut input)["productGroupID"] = json!("1");

        assert!(matches!(
            normalize(input),
            Err(NormalizeError::IslandProductMismatch { .. })
        ));
    }

    #[test]
    fn the_island_and_the_record_must_offer_the_same_variants() {
        let mut input = document();
        island_offer_mut(&mut input, 1)["url"] =
            json!("https://minisforumpc.eu/products/minisforum-u700-refurbished?variant=1");

        assert!(matches!(
            normalize(input),
            Err(NormalizeError::VariantIdsMismatch { island, .. })
                if island == [53296398172526, 1]
        ));
    }

    #[test]
    fn the_island_and_the_record_must_agree_on_a_variant() {
        let mut price = document();
        island_offer_mut(&mut price, 0)["price"] = json!("129.10");
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
        island_mut(&mut sku)["hasVariant"][0]
            .as_object_mut()
            .expect("a variant")
            .remove("sku");
        assert!(matches!(
            normalize(sku),
            Err(NormalizeError::VariantFieldMismatch { field, .. }) if field == "sku"
        ));
    }

    /// As on `products-minisforum-ms-a1-refurbished.html`, whose second variant has neither.
    #[test]
    fn a_missing_island_sku_matches_a_missing_record_sku() {
        let mut input = document();
        island_mut(&mut input)["hasVariant"][0]
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
        island_mut(&mut input)["hasVariant"][0]
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
                {"name": "CPU", "value": "Intel® Core™ i5-5257U Processor"},
                {"name": "Type", "value": "8GB RAM + 256GB SSD"}
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
        let island = input["ld-json"][ISLAND].clone();
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
            "id": 1, "product_id": 14959267250542_u64, "position": 1,
            "created_at": "2026-02-11T15:45:22+08:00", "updated_at": "2026-02-11T15:45:22+08:00",
            "alt": null, "width": 1600, "height": 1600,
            "src": "//minisforumpc.eu/cdn/shop/files/v.jpg?v=1", "variant_ids": [53296398172526_u64]
        });

        let variant = &normalized(input)["product"]["variants"][0];
        assert_eq!(
            variant["id"],
            json!("gid://shopify/ProductVariant/53296398172526")
        );
        assert_eq!(
            variant["image"],
            json!("https://minisforumpc.eu/cdn/shop/files/v.jpg?v=1")
        );
    }
}
