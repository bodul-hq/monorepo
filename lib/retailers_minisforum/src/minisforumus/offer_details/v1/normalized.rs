//! Expresses the storefront's product facts in the system's vocabulary. The page states the
//! product twice - as the theme's schema.org `Product` island and as Liquid's product record - and
//! they cross here as one product: the record for the product and its variants, the island for
//! what only it states, the market's currency and address. Where both state the same fact,
//! normalization checks that they agree; what nothing needs stays behind as the commented-out
//! exclusion records in `product`.

pub mod product;

use super::raw;
use super::raw::ldjsonitem::{LdJson, Product as Island};
use money::{Currency, Money};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Error)]
pub enum NormalizeError {
    #[error("no `Product` ld+json island on the document")]
    MissingProductIsland,

    #[error("{count} `Product` ld+json islands on the document; the theme renders one")]
    SeveralProductIslands { count: usize },

    #[error("offer currency `{currency}` is not supported: {reason}")]
    UnsupportedCurrency { currency: String, reason: String },

    #[error("offer currency `{currency}` has no hundredths, the unit Liquid renders amounts in")]
    CurrencyWithoutHundredths { currency: String },

    #[error("the island names the product `{island}`, the record `{record}`")]
    ProductNameMismatch { island: String, record: String },

    #[error("the island's URL `{url}` is not the product's `/products/{handle}`")]
    ProductUrlMismatch { url: String, handle: String },

    #[error("the island counts {island} offers, the record {record} variants")]
    OfferCountMismatch { island: u64, record: usize },

    #[error("variant `{variant_id}` selects {selected} option values against {offered} options")]
    OptionCountMismatch {
        variant_id: u64,
        offered: usize,
        selected: usize,
    },

    #[error(
        "variant `{variant_id}` is allocated selling plan `{selling_plan_id}`, which the product does not offer"
    )]
    UnknownSellingPlan {
        variant_id: u64,
        selling_plan_id: u64,
    },
}

// Liquid renders every amount in hundredths of the market's currency, so only a currency whose
// minor unit is a hundredth takes them as they are.
fn offer_currency(code: &str) -> Result<Currency, NormalizeError> {
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
fn product_gid(id: u64) -> String {
    format!("gid://shopify/Product/{id}")
}

fn variant_gid(id: u64) -> String {
    format!("gid://shopify/ProductVariant/{id}")
}

fn https_from_protocol_relative(url: String) -> String {
    match url.strip_prefix("//") {
        Some(rest) => format!("https://{rest}"),
        None => url,
    }
}

// The theme renders exactly one `Product` island; the organization, breadcrumb and website
// islands carry nothing about the offer.
fn product_island(items: Vec<raw::ldjsonitem::LdJsonItem>) -> Result<Island, NormalizeError> {
    let mut products: Vec<Island> = items
        .into_iter()
        .filter_map(|item| match item.data {
            LdJson::Product(product) => Some(product),
            LdJson::Organization(_) | LdJson::BreadcrumbList(_) | LdJson::WebSite(_) => None,
        })
        .collect();

    match products.len() {
        0 => Err(NormalizeError::MissingProductIsland),
        1 => Ok(products.remove(0)),
        count => Err(NormalizeError::SeveralProductIslands { count }),
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MinisforumUsNormalizedDocument {
    pub product: product::Product,
}

impl MinisforumUsNormalizedDocument {
    pub fn from_raw(
        raw: raw::MinisforumUsRawDocument,
    ) -> Result<MinisforumUsNormalizedDocument, NormalizeError> {
        let island = product_island(raw.ld_json)?;

        Ok(MinisforumUsNormalizedDocument {
            product: product::Product::from_raw(raw.product_json.data, island)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{MinisforumUsNormalizedDocument, NormalizeError};
    use crate::minisforumus::offer_details::v1::raw::MinisforumUsRawDocument;
    use crate::minisforumus::offer_details::v1::raw::ldjsonitem::tests::product as island;
    use crate::minisforumus::offer_details::v1::raw::productjson::tests::preorder_group;
    use crate::minisforumus::offer_details::v1::raw::tests::document;
    use html_extractor::serde_json::{self, Value, json};

    fn normalize(document: Value) -> Result<MinisforumUsNormalizedDocument, NormalizeError> {
        let raw: MinisforumUsRawDocument =
            serde_json::from_value(document).expect("the raw fixture deserializes");
        MinisforumUsNormalizedDocument::from_raw(raw)
    }

    fn normalized(document: Value) -> Value {
        serde_json::to_value(normalize(document).expect("the fixture normalizes"))
            .expect("serializes")
    }

    fn record_mut(document: &mut Value) -> &mut Value {
        &mut document["product_json"]["data"]
    }

    fn island_mut(document: &mut Value) -> &mut Value {
        &mut document["ld-json"][3]["data"]
    }

    fn variant_mut(document: &mut Value, index: usize) -> &mut Value {
        &mut record_mut(document)["variants"][index]
    }

    #[test]
    fn the_product_is_assembled_from_both_renderings() {
        let product = &normalized(document())["product"];

        assert_eq!(product["id"], json!("gid://shopify/Product/9093286985973"));
        assert_eq!(product["handle"], json!("minisforum-ms-01-se-workstation"));
        assert_eq!(
            product["url"],
            json!("https://store.minisforum.com/en-ca/products/minisforum-ms-01-se-workstation")
        );
        assert_eq!(product["title"], json!("MINISFORUM MS-01 SE"));
        assert_eq!(product["vendor"], json!("MINISFORUM US"));
        assert_eq!(product["brand"], json!("MINISFORUM"));
        assert_eq!(product["product_type"], json!("Work Station"));
        assert_eq!(product["options"], json!(["CPU", "Type", "Adapter"]));
        assert_eq!(product["variants"].as_array().map(Vec::len), Some(2));
    }

    #[test]
    fn a_price_is_hundredths_of_the_islands_currency() {
        let variant = &normalized(document())["product"]["variants"][0];
        assert_eq!(
            variant["price"],
            json!({"amount": 54343, "currency": "CAD"})
        );
        assert_eq!(
            variant["compare_at_price"],
            json!({"amount": 125930, "currency": "CAD"})
        );

        let mut usd = document();
        island_mut(&mut usd)["offers"]["priceCurrency"] = json!("USD");
        assert_eq!(
            normalized(usd)["product"]["variants"][0]["price"],
            json!({"amount": 54343, "currency": "USD"})
        );
    }

    #[test]
    fn a_zero_price_or_compare_at_price_is_absent() {
        let mut input = document();
        variant_mut(&mut input, 0)["price"] = json!(0);
        variant_mut(&mut input, 0)["compare_at_price"] = json!(0);

        let output = normalized(input);
        let variants = &output["product"]["variants"];
        assert_eq!(variants[0]["price"], Value::Null);
        assert_eq!(variants[0]["compare_at_price"], Value::Null);
        assert_eq!(variants[1]["price"]["amount"], json!(112308));
    }

    #[test]
    fn a_currency_without_hundredths_fails() {
        let mut input = document();
        island_mut(&mut input)["offers"]["priceCurrency"] = json!("JPY");

        assert!(matches!(
            normalize(input),
            Err(NormalizeError::CurrencyWithoutHundredths { currency }) if currency == "JPY"
        ));
    }

    #[test]
    fn an_unknown_currency_fails() {
        let mut input = document();
        island_mut(&mut input)["offers"]["priceCurrency"] = json!("SGD");

        assert!(matches!(
            normalize(input),
            Err(NormalizeError::UnsupportedCurrency { currency, .. }) if currency == "SGD"
        ));
    }

    #[test]
    fn exactly_one_product_island_is_read() {
        let mut none = document();
        none["ld-json"].as_array_mut().expect("islands").pop();
        assert!(matches!(
            normalize(none),
            Err(NormalizeError::MissingProductIsland)
        ));

        let mut two = document();
        two["ld-json"]
            .as_array_mut()
            .expect("islands")
            .push(json!({"type": "application/ld+json", "data": island()}));
        assert!(matches!(
            normalize(two),
            Err(NormalizeError::SeveralProductIslands { count: 2 })
        ));
    }

    #[test]
    fn the_island_and_the_record_must_describe_the_same_product() {
        let mut name = document();
        island_mut(&mut name)["name"] = json!("MINISFORUM MS-01");
        assert!(matches!(
            normalize(name),
            Err(NormalizeError::ProductNameMismatch { .. })
        ));

        let mut url = document();
        island_mut(&mut url)["url"] = json!("https://store.minisforum.com/en-ca/products/other");
        assert!(matches!(
            normalize(url),
            Err(NormalizeError::ProductUrlMismatch { .. })
        ));

        let mut offer_url = document();
        island_mut(&mut offer_url)["offers"]["url"] =
            json!("https://store.minisforum.com/en-ca/products/other");
        assert!(matches!(
            normalize(offer_url),
            Err(NormalizeError::ProductUrlMismatch { .. })
        ));

        let mut count = document();
        island_mut(&mut count)["offers"]["offerCount"] = json!(3);
        assert!(matches!(
            normalize(count),
            Err(NormalizeError::OfferCountMismatch {
                island: 3,
                record: 2
            })
        ));
    }

    #[test]
    fn option_values_pair_with_the_records_names_by_position() {
        let output = normalized(document());
        assert_eq!(
            output["product"]["variants"][0]["options"],
            json!([
                {"name": "CPU", "value": "Intel® Core™ i9-12900H"},
                {"name": "Type", "value": "Barebone"},
                {"name": "Adapter", "value": "US"}
            ])
        );

        let mut input = document();
        record_mut(&mut input)["options"] = json!(["CPU"]);
        assert!(matches!(
            normalize(input),
            Err(NormalizeError::OptionCountMismatch {
                offered: 1,
                selected: 3,
                ..
            })
        ));
    }

    #[test]
    fn shopifys_default_title_is_no_option() {
        let mut input = document();
        record_mut(&mut input)["options"] = json!(["Title"]);
        for index in 0..2 {
            variant_mut(&mut input, index)["options"] = json!(["Default Title"]);
        }

        let product = &normalized(input)["product"];
        assert_eq!(product["options"], json!([]));
        assert_eq!(product["variants"][0]["options"], json!([]));
    }

    #[test]
    fn an_empty_or_null_sku_or_barcode_is_absent() {
        for value in [json!(""), Value::Null] {
            let mut input = document();
            variant_mut(&mut input, 0)["sku"] = value.clone();
            variant_mut(&mut input, 0)["barcode"] = value.clone();

            let variant = &normalized(input)["product"]["variants"][0];
            assert_eq!(variant["sku"], Value::Null, "sku {value}");
            assert_eq!(variant["barcode"], Value::Null, "barcode {value}");
        }
    }

    #[test]
    fn a_selling_plan_allocation_names_its_plan() {
        let mut input = document();
        record_mut(&mut input)["selling_plan_groups"] = json!([preorder_group(10215063797)]);
        variant_mut(&mut input, 0)["selling_plan_allocations"] = json!([{
            "price_adjustments": [], "price": 54343, "compare_at_price": 125930,
            "per_delivery_price": 54343, "selling_plan_id": 10215063797_u64,
            "selling_plan_group_id": "6c17fabdef8f8d1808aafff6115bf384c07f7045"
        }]);

        let variants = &normalized(input.clone())["product"]["variants"];
        assert_eq!(
            variants[0]["selling_plans"],
            json!([{
                "group": "Preorder",
                "name": "Preorder",
                "price": {"amount": 54343, "currency": "CAD"},
                "compare_at_price": {"amount": 125930, "currency": "CAD"}
            }])
        );
        assert_eq!(variants[1]["selling_plans"], json!([]));

        variant_mut(&mut input, 0)["selling_plan_allocations"][0]["selling_plan_id"] = json!(1);
        assert!(matches!(
            normalize(input),
            Err(NormalizeError::UnknownSellingPlan {
                selling_plan_id: 1,
                ..
            })
        ));
    }

    #[test]
    fn ids_become_gids_and_images_become_https() {
        let product = &normalized(document())["product"];

        assert_eq!(
            product["variants"][0]["id"],
            json!("gid://shopify/ProductVariant/46589796385013")
        );
        assert_eq!(
            product["images"],
            json!([
                "https://store.minisforum.com/cdn/shop/files/MS-01.png?v=1",
                "https://store.minisforum.com/cdn/shop/files/MS-01-2.png?v=1"
            ])
        );
    }
}
