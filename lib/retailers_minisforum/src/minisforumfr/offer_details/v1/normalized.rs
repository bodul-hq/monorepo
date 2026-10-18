//! Expresses the storefront's product facts in the system's vocabulary. Liquid's product record
//! crosses as the product; the storefront's schema.org island restates its offers, so
//! normalization checks the two agree and keeps from the island only its `category`. The reviews
//! app's island restates nothing the record lacks and is not read. What nothing needs stays
//! behind as the commented-out exclusion records in `product`.

mod island;
pub mod product;

use super::raw;
use money::{Currency, Money};
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// The storefront the pages are saved from; the record states only the product's handle.
const ORIGIN: &str = "https://minisforumpc.fr";

#[derive(Debug, Clone, PartialEq, Error)]
pub enum NormalizeError {
    #[error("shop currency `{currency}` is not supported: {reason}")]
    UnsupportedCurrency { currency: String, reason: String },

    #[error("shop currency `{currency}` has no hundredths, the unit Liquid renders amounts in")]
    CurrencyWithoutHundredths { currency: String },

    #[error("page locale `{locale}` is not supported; the storefront serves `fr_FR` and `en_US`")]
    UnsupportedLocale { locale: String },

    #[error("the document carries {count} storefront product islands; at most one is read")]
    SeveralProductIslands { count: usize },

    #[error("the island is for `{island}`, the record for `{record}`")]
    IslandProductMismatch { island: String, record: String },

    #[error("the island is at `{island}`, the page's product at `{record}`")]
    IslandUrlMismatch { island: String, record: String },

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

/// The page's language, which sets the path the storefront serves a product under. The record
/// states neither, so the PayPal tag's locale is read; the island's URL, where there is one,
/// is checked against it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Locale {
    /// `/products/<handle>`, on the 59 `fr_FR` pages.
    French,
    /// `/en/products/<handle>`, on the 59 `en_US` pages.
    English,
}

impl Locale {
    fn from_paypal(code: &str) -> Result<Locale, NormalizeError> {
        match code {
            "fr_FR" => Ok(Locale::French),
            "en_US" => Ok(Locale::English),
            _ => Err(NormalizeError::UnsupportedLocale {
                locale: code.to_string(),
            }),
        }
    }

    fn product_url(self, handle: &str) -> String {
        match self {
            Locale::French => format!("{ORIGIN}/products/{handle}"),
            Locale::English => format!("{ORIGIN}/en/products/{handle}"),
        }
    }
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
pub struct MinisforumFrNormalizedDocument {
    pub product: product::Product,
}

impl MinisforumFrNormalizedDocument {
    pub fn from_raw(
        raw: raw::MinisforumFrRawDocument,
    ) -> Result<MinisforumFrNormalizedDocument, NormalizeError> {
        let currency = shop_currency(&raw.paypal.currency)?;
        let locale = Locale::from_paypal(&raw.paypal.locale)?;
        let island = island::Island::from_raw(raw.ld_json)?;

        Ok(MinisforumFrNormalizedDocument {
            product: product::Product::from_raw(raw.product_json.data, island, currency, locale)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{MinisforumFrNormalizedDocument, NormalizeError};
    use crate::minisforumfr::offer_details::v1::raw::MinisforumFrRawDocument;
    use crate::minisforumfr::offer_details::v1::raw::tests::document;
    use html_extractor::serde_json::{self, Value, json};

    const IMAGE: &str =
        "https://minisforumpc.fr/cdn/shop/files/ms-03-local-ai-edge-workstation-1.png?v=1788346698";

    fn normalize(document: Value) -> Result<MinisforumFrNormalizedDocument, NormalizeError> {
        let raw: MinisforumFrRawDocument =
            serde_json::from_value(document).expect("the raw fixture deserializes");
        MinisforumFrNormalizedDocument::from_raw(raw)
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

    /// As on the two pages whose product is announced without a price: the Organization island
    /// and the reviews app's remain.
    fn without_product_island(mut document: Value) -> Value {
        document["ld-json"]
            .as_array_mut()
            .expect("islands")
            .remove(1);
        document
    }

    /// The fixture as its English page: the locale, and the island's URLs and category.
    fn english(mut document: Value) -> Value {
        document["paypal"]["data-locale"] = json!("en_US");
        let island = &mut document["ld-json"][1]["data"];
        island["@id"] = json!("/en/products/minisforum-ms-03-workstation#product");
        island["category"] = json!("Desktop Computers");
        island["url"] = json!("https://minisforumpc.fr/en/products/minisforum-ms-03-workstation");
        for variant in island["hasVariant"].as_array_mut().expect("variants") {
            let id = variant["offers"]["url"]
                .as_str()
                .and_then(|url| url.rsplit_once("variant="))
                .map(|(_, id)| id.to_string())
                .expect("a variant id");
            let path = format!("/en/products/minisforum-ms-03-workstation?variant={id}");
            variant["@id"] = json!(format!("{path}#variant"));
            variant["offers"]["@id"] = json!(format!("{path}#offer"));
            variant["offers"]["url"] = json!(format!("https://minisforumpc.fr{path}"));
        }
        document
    }

    #[test]
    fn the_product_is_the_record_with_the_islands_category() {
        let product = &normalized(document())["product"];

        assert_eq!(product["id"], json!("gid://shopify/Product/15885734478193"));
        assert_eq!(product["handle"], json!("minisforum-ms-03-workstation"));
        assert_eq!(
            product["url"],
            json!("https://minisforumpc.fr/products/minisforum-ms-03-workstation")
        );
        assert_eq!(
            product["title"],
            json!("Minisforum MS-03 Workstation Locale IA Edge")
        );
        assert_eq!(product["vendor"], json!("Minisforum FR"));
        assert_eq!(product["product_type"], json!("Mini Workstation"));
        assert_eq!(product["category"], json!("Ordinateurs de bureau"));
        assert_eq!(product["tags"], json!(["INTEL"]));
        assert_eq!(product["options"], json!(["CPU", "Type"]));
        assert_eq!(product["images"], json!([IMAGE]));
        assert_eq!(product["variants"].as_array().map(Vec::len), Some(2));
    }

    #[test]
    fn an_english_page_is_served_under_en() {
        let product = &normalized(english(document()))["product"];

        assert_eq!(
            product["url"],
            json!("https://minisforumpc.fr/en/products/minisforum-ms-03-workstation")
        );
        assert_eq!(product["category"], json!("Desktop Computers"));
        assert_eq!(product["id"], json!("gid://shopify/Product/15885734478193"));
    }

    #[test]
    fn an_english_page_without_a_product_island_is_served_under_en() {
        let mut input = without_product_island(document());
        input["paypal"]["data-locale"] = json!("en_US");

        let product = &normalized(input)["product"];
        assert_eq!(
            product["url"],
            json!("https://minisforumpc.fr/en/products/minisforum-ms-03-workstation")
        );
    }

    #[test]
    fn a_price_is_hundredths_of_the_shop_currency() {
        let variant = &normalized(document())["product"]["variants"][0];
        assert_eq!(
            variant["price"],
            json!({"amount": 79900, "currency": "EUR"})
        );
        assert_eq!(
            variant["compare_at_price"],
            json!({"amount": 99900, "currency": "EUR"})
        );
    }

    #[test]
    fn a_zero_price_is_no_price() {
        // As on `products-minisforum-ai-x1.html`, whose island states it as `0.00`.
        let mut input = document();
        record_variant_mut(&mut input, 0)["price"] = json!(0);
        record_variant_mut(&mut input, 0)["compare_at_price"] = Value::Null;
        island_offer_mut(&mut input, 0)["price"] = json!("0.00");

        let variants = &normalized(input)["product"]["variants"];
        assert_eq!(variants[0]["price"], Value::Null);
        assert_eq!(variants[0]["compare_at_price"], Value::Null);
        assert_eq!(variants[1]["price"]["amount"], json!(139900));
    }

    #[test]
    fn a_zero_price_without_an_island_is_no_price() {
        let mut input = without_product_island(document());
        record_variant_mut(&mut input, 0)["price"] = json!(0);
        record_variant_mut(&mut input, 0)["compare_at_price"] = json!(0);

        let variants = &normalized(input)["product"]["variants"];
        assert_eq!(variants[0]["price"], Value::Null);
        assert_eq!(variants[0]["compare_at_price"], Value::Null);
        assert_eq!(variants[1]["price"]["amount"], json!(139900));
    }

    #[test]
    fn a_page_without_a_product_island_has_no_category() {
        let product = &normalized(without_product_island(document()))["product"];

        assert_eq!(product["category"], Value::Null);
        assert_eq!(product["variants"].as_array().map(Vec::len), Some(2));
    }

    #[test]
    fn the_reviews_apps_island_is_not_read() {
        let mut input = document();
        input["ld-json"][2]["data"]["offers"]["price"] = json!("1.00");
        input["ld-json"][2]["data"]["offers"]["url"] =
            json!("https://minisforumpc.fr/products/another?variant=1");

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
        input["ld-json"][1]["data"] =
            crate::minisforumfr::offer_details::v1::raw::ldjson::tests::native_product();

        let product = &normalized(input)["product"];
        assert_eq!(product["category"], json!("Ordinateurs de bureau"));
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
    fn an_unknown_locale_fails() {
        let mut input = document();
        input["paypal"]["data-locale"] = json!("de_DE");

        assert!(matches!(
            normalize(input),
            Err(NormalizeError::UnsupportedLocale { locale }) if locale == "de_DE"
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
        input["ld-json"][1]["data"]["productGroupID"] = json!("1");

        assert!(matches!(
            normalize(input),
            Err(NormalizeError::IslandProductMismatch { island, .. }) if island == "1"
        ));
    }

    #[test]
    fn an_island_in_another_language_than_the_page_fails() {
        // The page says it is English while its island is at the French URL.
        let mut input = document();
        input["paypal"]["data-locale"] = json!("en_US");

        assert!(matches!(
            normalize(input),
            Err(NormalizeError::IslandUrlMismatch { island, record })
                if island == "https://minisforumpc.fr/products/minisforum-ms-03-workstation"
                    && record == "https://minisforumpc.fr/en/products/minisforum-ms-03-workstation"
        ));
    }

    #[test]
    fn the_island_and_the_record_must_offer_the_same_variants() {
        let mut input = document();
        island_offer_mut(&mut input, 1)["url"] =
            json!("https://minisforumpc.fr/products/minisforum-ms-03-workstation?variant=1");

        assert!(matches!(
            normalize(input),
            Err(NormalizeError::VariantIdsMismatch { island, .. })
                if island == [64248383471985, 1]
        ));
    }

    #[test]
    fn the_island_and_the_record_must_agree_on_a_variant() {
        let mut price = document();
        island_offer_mut(&mut price, 0)["price"] = json!("799.01");
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

    #[test]
    fn a_price_with_a_decimal_comma_fails() {
        let mut input = document();
        island_offer_mut(&mut input, 0)["price"] = json!("799,00");

        assert!(matches!(
            normalize(input),
            Err(NormalizeError::OfferPriceNotDecimal { price, .. }) if price == "799,00"
        ));
    }

    /// As on `products-gift-card.html`, whose variants have none.
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

    /// As on `products-price-difference.html`.
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
                {"name": "Type", "value": "Barebone (Licence OS non incluse)"}
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
            "id": 1, "product_id": 15885734478193_u64, "position": 1,
            "created_at": "2026-09-02T19:44:32+08:00", "updated_at": "2026-09-02T19:44:34+08:00",
            "alt": null, "width": 1600, "height": 1600,
            "src": "//minisforumpc.fr/cdn/shop/files/v.jpg?v=1", "variant_ids": [64248383471985_u64]
        });

        let variant = &normalized(input)["product"]["variants"][0];
        assert_eq!(
            variant["id"],
            json!("gid://shopify/ProductVariant/64248383471985")
        );
        assert_eq!(
            variant["image"],
            json!("https://minisforumpc.fr/cdn/shop/files/v.jpg?v=1")
        );
    }

    #[test]
    fn empty_description_type_and_barcode_are_none() {
        let mut input = document();
        input["product_json"]["data"]["description"] = json!("");
        input["product_json"]["data"]["type"] = json!("");
        record_variant_mut(&mut input, 0)["barcode"] = json!("");

        let product = &normalized(input)["product"];
        assert_eq!(product["description"], Value::Null);
        assert_eq!(product["product_type"], Value::Null);
        assert_eq!(product["variants"][0]["barcode"], Value::Null);
    }
}
