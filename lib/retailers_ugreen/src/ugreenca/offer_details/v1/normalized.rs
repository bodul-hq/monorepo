//! Expresses the storefront's product facts in the system's vocabulary. The page states the
//! product across several islands - the `Product` ld+json, Shopify's analytics `meta`, the
//! theme's featured-product attribute and variant picker, the title and the gallery - and they
//! cross here as one product. Where two islands state the same fact, normalization checks that
//! they agree; what nothing needs stays behind as the commented-out exclusion records in
//! `product`.

pub mod product;

use super::raw;
use money::{Currency, Money};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Error)]
pub enum NormalizeError {
    #[error("no `Product` ld+json island on the document")]
    MissingProductIsland,

    #[error("shop currency `{currency}` is not supported: {reason}")]
    UnsupportedCurrency { currency: String, reason: String },

    #[error("shop currency `{currency}` has no hundredths, the unit Liquid renders amounts in")]
    CurrencyWithoutHundredths { currency: String },

    #[error("an offer is priced in `{offer}`, the shop in `{shop}`")]
    OfferCurrencyMismatch { offer: String, shop: String },

    #[error(
        "the islands name different products: analytics `{analytics}`, featured `{featured}`, \
         ld+json `{island}`"
    )]
    ProductIdMismatch {
        analytics: u64,
        featured: String,
        island: String,
    },

    #[error("the product URL `{url}` is not for the product's handle `{handle}`")]
    ProductUrlMismatch { url: String, handle: String },

    #[error("the variant picker renders its {present} island without its {missing} island")]
    IncompletePicker {
        present: &'static str,
        missing: &'static str,
    },

    #[error("the analytics list variants {analytics:?}, the variant picker {picker:?}")]
    VariantIdsMismatch {
        analytics: Vec<u64>,
        picker: Vec<u64>,
    },

    #[error("no variant picker, but the analytics list {count} variants rather than one")]
    NoVariantRecords { count: usize },

    #[error("the featured product is priced at {featured}, its only variant at {analytics}")]
    FeaturedPriceMismatch { featured: i64, analytics: i64 },

    #[error("the picker's option positions are {positions:?}, not one to the option count")]
    OptionPositionsOutOfOrder { positions: Vec<usize> },

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

// Liquid gives bare numeric ids; the family identifies Shopify objects by their Admin API gid.
fn variant_gid(id: u64) -> String {
    format!("gid://shopify/ProductVariant/{id}")
}

fn https_from_protocol_relative(url: &str) -> String {
    match url.strip_prefix("//") {
        Some(rest) => format!("https://{rest}"),
        None => url.to_string(),
    }
}

fn non_empty(value: String) -> Option<String> {
    (!value.is_empty()).then_some(value)
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UgreenCaNormalizedDocument {
    pub product: product::Product,
}

impl UgreenCaNormalizedDocument {
    pub fn from_raw(
        raw: raw::UgreenCaRawDocument,
    ) -> Result<UgreenCaNormalizedDocument, NormalizeError> {
        Ok(UgreenCaNormalizedDocument {
            product: product::Product::from_raw(raw)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{NormalizeError, UgreenCaNormalizedDocument};
    use crate::ugreenca::offer_details::v1::raw::UgreenCaRawDocument;
    use crate::ugreenca::offer_details::v1::raw::tests::document;
    use html_extractor::serde_json::{self, Value, json};

    fn normalize(document: Value) -> Result<UgreenCaNormalizedDocument, NormalizeError> {
        let raw: UgreenCaRawDocument =
            serde_json::from_value(document).expect("the raw fixture deserializes");
        UgreenCaNormalizedDocument::from_raw(raw)
    }

    fn normalized(document: Value) -> Value {
        serde_json::to_value(normalize(document).expect("the fixture normalizes"))
            .expect("serializes")
    }

    fn picker_variant_mut(document: &mut Value, index: usize) -> &mut Value {
        &mut document["variants"]["data"][index]
    }

    fn product_island_mut(document: &mut Value) -> &mut Value {
        &mut document["ld-json"][3]["data"]
    }

    /// The fixture reduced to a `Default Title` product: no picker, one variant.
    fn default_title_document() -> Value {
        let mut input = document();
        let object = input.as_object_mut().expect("an object");
        object.remove("variants");
        object.remove("options");

        let variants = input["analytics"]["meta"]["product"]["variants"]
            .as_array_mut()
            .expect("variants");
        variants.truncate(1);
        variants[0]["public_title"] = Value::Null;
        input
    }

    #[test]
    fn the_product_is_assembled_from_every_island() {
        let product = &normalized(document())["product"];

        assert_eq!(product["id"], json!("gid://shopify/Product/8006139642144"));
        assert_eq!(product["handle"], json!("100w-3c1a-gan-fast-charger"));
        assert_eq!(
            product["url"],
            json!("https://ca.ugreen.com/products/100w-3c1a-gan-fast-charger")
        );
        assert_eq!(product["title"], json!("UGREEN Nexode 100W USB C Charger"));
        assert_eq!(product["description"], json!("Fast GaN charger"));
        assert_eq!(product["vendor"], json!("UGREEN STORE"));
        assert_eq!(product["product_type"], json!("GaN Charger"));
        assert_eq!(product["options"], json!(["Color"]));
        assert_eq!(product["variants"].as_array().map(Vec::len), Some(2));
    }

    #[test]
    fn a_picker_variant_carries_its_own_record() {
        let mut input = document();
        picker_variant_mut(&mut input, 0)["compare_at_price"] = json!(7999);

        let variant = &normalized(input)["product"]["variants"][0];
        assert_eq!(
            variant["id"],
            json!("gid://shopify/ProductVariant/44061363634464")
        );
        assert_eq!(variant["title"], json!("Black"));
        assert_eq!(variant["price"], json!({"amount": 5999, "currency": "CAD"}));
        assert_eq!(
            variant["compare_at_price"],
            json!({"amount": 7999, "currency": "CAD"})
        );
        assert_eq!(variant["barcode"], json!("6957303813346"));
        assert_eq!(variant["requires_shipping"], json!(true));
        assert_eq!(variant["weight"], json!(265));
        assert_eq!(
            variant["options"],
            json!([{"name": "Color", "value": "Black"}])
        );
    }

    #[test]
    fn a_default_title_variant_is_read_from_the_analytics_and_the_featured_product() {
        let output = normalized(default_title_document());
        let product = &output["product"];

        assert_eq!(product["options"], json!([]));
        assert_eq!(
            product["variants"],
            json!([{
                "id": "gid://shopify/ProductVariant/44061363634464",
                "title": null,
                "name": "UGREEN Nexode 100W USB C Charger - Black",
                "sku": "40737",
                "barcode": "6957303813346",
                "price": {"amount": 5999, "currency": "CAD"},
                "compare_at_price": {"amount": 7999, "currency": "CAD"},
                "available": true,
                "requires_shipping": null,
                "weight": null,
                "image": null,
                "options": []
            }])
        );
    }

    #[test]
    fn a_default_title_price_must_match_the_featured_product() {
        let mut input = default_title_document();
        input["featured"]["data"]["price"] = json!(1);

        assert!(matches!(
            normalize(input),
            Err(NormalizeError::FeaturedPriceMismatch {
                featured: 1,
                analytics: 5999
            })
        ));
    }

    #[test]
    fn without_a_picker_the_analytics_must_list_one_variant() {
        let mut input = document();
        let object = input.as_object_mut().expect("an object");
        object.remove("variants");
        object.remove("options");

        assert!(matches!(
            normalize(input),
            Err(NormalizeError::NoVariantRecords { count: 2 })
        ));
    }

    #[test]
    fn half_a_picker_fails() {
        let mut input = document();
        input.as_object_mut().expect("an object").remove("options");

        assert!(matches!(
            normalize(input),
            Err(NormalizeError::IncompletePicker {
                missing: "options",
                ..
            })
        ));
    }

    #[test]
    fn the_analytics_and_the_picker_must_list_the_same_variants() {
        let mut input = document();
        picker_variant_mut(&mut input, 1)["id"] = json!(1);

        assert!(matches!(
            normalize(input),
            Err(NormalizeError::VariantIdsMismatch { picker, .. }) if picker == [44061363634464, 1]
        ));
    }

    #[test]
    fn the_islands_must_name_the_same_product() {
        let mut featured = document();
        featured["featured"]["data"]["id"] = json!("1");
        assert!(matches!(
            normalize(featured),
            Err(NormalizeError::ProductIdMismatch { .. })
        ));

        let mut island = document();
        product_island_mut(&mut island)["productID"] = json!("1");
        assert!(matches!(
            normalize(island),
            Err(NormalizeError::ProductIdMismatch { .. })
        ));

        let mut url = document();
        product_island_mut(&mut url)["url"] = json!("https://ca.ugreen.com/products/other");
        assert!(matches!(
            normalize(url),
            Err(NormalizeError::ProductUrlMismatch { .. })
        ));
    }

    #[test]
    fn a_french_page_keeps_its_localized_url() {
        let mut input = document();
        product_island_mut(&mut input)["url"] =
            json!("https://ca.ugreen.com/fr/products/100w-3c1a-gan-fast-charger");

        assert_eq!(
            normalized(input)["product"]["url"],
            json!("https://ca.ugreen.com/fr/products/100w-3c1a-gan-fast-charger")
        );
    }

    #[test]
    fn an_offer_in_another_currency_fails() {
        let mut input = document();
        product_island_mut(&mut input)["offers"]["priceCurrency"] = json!("USD");

        assert!(matches!(
            normalize(input),
            Err(NormalizeError::OfferCurrencyMismatch { offer, shop })
                if offer == "USD" && shop == "CAD"
        ));
    }

    #[test]
    fn a_currency_without_hundredths_fails() {
        let mut input = document();
        input["shop"]["data"]["currencyCode"] = json!("JPY");

        assert!(matches!(
            normalize(input),
            Err(NormalizeError::CurrencyWithoutHundredths { currency }) if currency == "JPY"
        ));
    }

    #[test]
    fn a_page_without_a_product_island_fails() {
        let mut input = document();
        input["ld-json"]
            .as_array_mut()
            .expect("islands")
            .truncate(3);

        assert!(matches!(
            normalize(input),
            Err(NormalizeError::MissingProductIsland)
        ));
    }

    #[test]
    fn options_are_ordered_by_position_and_paired_by_it() {
        let mut input = document();
        input["options"]["data"] = json!([
            {"name": "Size", "position": 2, "values": ["3FT"]},
            {"name": "Color", "position": 1, "values": ["Black", "White"]}
        ]);
        for index in 0..2 {
            let variant = picker_variant_mut(&mut input, index);
            let color = variant["option1"].clone();
            variant["option2"] = json!("3FT");
            variant["options"] = json!([color, "3FT"]);
        }

        let output = normalized(input);
        assert_eq!(output["product"]["options"], json!(["Color", "Size"]));
        assert_eq!(
            output["product"]["variants"][0]["options"],
            json!([{"name": "Color", "value": "Black"}, {"name": "Size", "value": "3FT"}])
        );

        let mut gap = document();
        gap["options"]["data"][0]["position"] = json!(2);
        assert!(matches!(
            normalize(gap),
            Err(NormalizeError::OptionPositionsOutOfOrder { .. })
        ));

        let mut count = document();
        picker_variant_mut(&mut count, 0)["options"] = json!(["Black", "3FT"]);
        assert!(matches!(
            normalize(count),
            Err(NormalizeError::OptionCountMismatch {
                offered: 1,
                selected: 2,
                ..
            })
        ));
    }

    #[test]
    fn an_empty_barcode_or_product_type_is_absent() {
        let mut input = document();
        input["analytics"]["meta"]["product"]["type"] = json!("");

        let product = &normalized(input)["product"];
        assert_eq!(product["product_type"], Value::Null);
        assert_eq!(product["variants"][1]["barcode"], Value::Null);
    }

    #[test]
    fn gallery_images_become_https() {
        assert_eq!(
            normalized(document())["product"]["images"],
            json!(["https://ca.ugreen.com/cdn/shop/files/charger.png?v=1746739856"])
        );
    }
}
