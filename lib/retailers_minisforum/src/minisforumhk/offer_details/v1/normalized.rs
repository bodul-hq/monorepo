//! Expresses the storefront's product facts in the system's vocabulary. The page states the
//! product across several islands - the schema.org `Product`, the theme's variant list, the
//! selected variant, `seal_product` and the variant selector's labels - and they cross here as
//! one product. Where two islands state the same fact, normalization checks that they agree;
//! what nothing needs stays behind as the commented-out exclusion records in `product`.
//!
//! The variant list is the record of every variant, and `seal_product` the product's id and
//! title; the schema.org island supplies what neither has (brand, category, description, image,
//! path) and restates the offers, which are checked against the variants.

mod island;
pub mod product;

use super::raw;
use money::{Currency, Money};
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// The storefront the pages are saved from; the island states only the product's path.
const ORIGIN: &str = "https://minisforum.hk";

#[derive(Debug, Clone, PartialEq, Error)]
pub enum NormalizeError {
    #[error("shop currency `{currency}` is not supported: {reason}")]
    UnsupportedCurrency { currency: String, reason: String },

    #[error("shop currency `{currency}` has no hundredths, the unit Liquid renders amounts in")]
    CurrencyWithoutHundredths { currency: String },

    #[error("the document carries no product ld+json island")]
    MissingProductIsland,

    #[error("the document carries {count} product ld+json islands; exactly one is read")]
    SeveralProductIslands { count: usize },

    #[error("the island's product path `{path}` is not under `/products/`")]
    ProductPathNotUnderProducts { path: String },

    #[error("the island is for product `{island}`, `seal_product` for `{seal}`")]
    ProductIdMismatch { island: u64, seal: u64 },

    #[error("the island names the product `{island}`, `seal_product` `{seal}`")]
    ProductTitleMismatch { island: String, seal: String },

    #[error("the island's image is `{island}`, `seal_product`'s `{seal}`")]
    ProductImageMismatch { island: String, seal: String },

    #[error("offer `{offer}` is in `{island}`, the shop in `{shop}`")]
    IslandCurrencyMismatch {
        offer: String,
        island: String,
        shop: String,
    },

    #[error("offer URL `{url}` names no numeric `variant`")]
    OfferUrlWithoutVariant { url: String },

    #[error("offer URL `{url}` is not under the product's path `{product}`")]
    OfferUrlForAnotherProduct { url: String, product: String },

    #[error("offer availability `{value}` is neither schema.org InStock nor OutOfStock")]
    UnknownAvailability { value: String },

    #[error("the island offers variants {island:?}, the variant list {record:?}")]
    VariantIdsMismatch { island: Vec<u64>, record: Vec<u64> },

    #[error(
        "variant `{variant_id}` `{field}` is `{island}` on the island, `{record}` in the variant list"
    )]
    VariantFieldMismatch {
        variant_id: u64,
        field: String,
        island: String,
        record: String,
    },

    #[error("the selected variant `{variant_id}` is not among the listed variants")]
    UnknownSelectedVariant { variant_id: u64 },

    #[error("the selected variant `{variant_id}` differs from its entry in the variant list")]
    SelectedVariantDiffers { variant_id: u64 },

    #[error("the island's SKU is {island:?}, the selected variant's {selected:?}")]
    SelectedSkuMismatch {
        island: Option<String>,
        selected: Option<String>,
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
pub struct MinisforumHkNormalizedDocument {
    pub product: product::Product,
}

impl MinisforumHkNormalizedDocument {
    pub fn from_raw(
        raw: raw::MinisforumHkRawDocument,
    ) -> Result<MinisforumHkNormalizedDocument, NormalizeError> {
        let currency = shop_currency(&raw.paypal.currency)?;
        let island = island::Island::from_raw(raw.ld_json)?;

        Ok(MinisforumHkNormalizedDocument {
            product: product::Product::from_raw(
                product::Sources {
                    variants: raw.variants.data,
                    selected: raw.selected_variant.data,
                    seal: raw.seal_product.data,
                    option_names: raw.option_names,
                    island,
                },
                currency,
            )?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{MinisforumHkNormalizedDocument, NormalizeError};
    use crate::minisforumhk::offer_details::v1::raw::MinisforumHkRawDocument;
    use crate::minisforumhk::offer_details::v1::raw::tests::document;
    use html_extractor::serde_json::{self, Value, json};

    fn normalize(document: Value) -> Result<MinisforumHkNormalizedDocument, NormalizeError> {
        let raw: MinisforumHkRawDocument =
            serde_json::from_value(document).expect("the raw fixture deserializes");
        MinisforumHkNormalizedDocument::from_raw(raw)
    }

    fn normalized(document: Value) -> Value {
        serde_json::to_value(normalize(document).expect("the fixture normalizes"))
            .expect("serializes")
    }

    fn record_variant_mut(document: &mut Value, index: usize) -> &mut Value {
        &mut document["variants"]["data"][index]
    }

    /// The selected variant is the variant list's entry; a test that edits the first variant
    /// edits both.
    fn sync_selected(document: &mut Value) {
        document["selected_variant"]["data"] = document["variants"]["data"][0].clone();
    }

    fn island_mut(document: &mut Value) -> &mut Value {
        &mut document["ld-json"][0]["data"]
    }

    fn island_offer_mut(document: &mut Value, index: usize) -> &mut Value {
        &mut document["ld-json"][0]["data"]["offers"][index]
    }

    /// The page of a product without options, as `products-minisforum-g7-pro.html`: one
    /// `Default Title` variant, no selector labels.
    fn default_title_document() -> Value {
        let mut input = document();
        input["variants"]["data"]
            .as_array_mut()
            .expect("variants")
            .truncate(1);
        input["option_names"] = json!([]);
        let variant = record_variant_mut(&mut input, 0);
        variant["title"] = json!("Default Title");
        variant["option1"] = json!("Default Title");
        variant["option2"] = Value::Null;
        variant["options"] = json!(["Default Title"]);
        variant["public_title"] = Value::Null;
        variant["name"] = json!("MS-S1 Max");
        sync_selected(&mut input);
        let island = island_mut(&mut input);
        island["offers"].as_array_mut().expect("offers").truncate(1);
        island["offers"][0]["name"] = json!("Default Title");
        input
    }

    #[test]
    fn the_product_is_assembled_from_the_record_the_island_and_the_seal() {
        let product = &normalized(document())["product"];

        assert_eq!(product["id"], json!("gid://shopify/Product/8611751919794"));
        assert_eq!(product["handle"], json!("minisforum-ms-s1-max"));
        assert_eq!(
            product["url"],
            json!("https://minisforum.hk/products/minisforum-ms-s1-max")
        );
        assert_eq!(product["title"], json!("MS-S1 Max"));
        assert_eq!(product["brand"], json!("Minisforum HK"));
        assert_eq!(product["category"], json!("Mini PC"));
        assert_eq!(
            product["description"],
            json!("MINISFORUM MS-S1 Max MINI WORKSTATION\n\nRAM：LPDDR5x Unified Memory")
        );
        assert_eq!(product["options"], json!(["CPU", "規格"]));
        assert_eq!(
            product["images"],
            json!(["https://minisforum.hk/cdn/shop/files/MS-S1-Max.png?v=1758849971"])
        );
        assert_eq!(product["variants"].as_array().map(Vec::len), Some(2));
    }

    #[test]
    fn a_price_is_hundredths_of_the_shop_currency() {
        let variant = &normalized(document())["product"]["variants"][0];
        assert_eq!(
            variant["price"],
            json!({"amount": 3299900, "currency": "HKD"})
        );
        assert_eq!(
            variant["compare_at_price"],
            json!({"amount": 3499900, "currency": "HKD"})
        );
    }

    /// As on `products-minisforum-deg1.html`, whose compare-at price equals its price.
    #[test]
    fn a_compare_at_price_that_is_no_markdown_is_no_price() {
        for compare_at in [json!(null), json!(3299900), json!(0)] {
            let mut input = document();
            record_variant_mut(&mut input, 0)["compare_at_price"] = compare_at;
            sync_selected(&mut input);

            let variants = &normalized(input)["product"]["variants"];
            assert_eq!(variants[0]["compare_at_price"], Value::Null);
            assert_eq!(variants[1]["compare_at_price"]["amount"], json!(5699900));
        }
    }

    #[test]
    fn a_zero_price_is_no_price() {
        let mut input = document();
        record_variant_mut(&mut input, 0)["price"] = json!(0);
        sync_selected(&mut input);
        island_offer_mut(&mut input, 0)["price"] = json!(0.0);

        let variants = &normalized(input)["product"]["variants"];
        assert_eq!(variants[0]["price"], Value::Null);
        assert_eq!(variants[0]["compare_at_price"], Value::Null);
        assert_eq!(variants[1]["price"]["amount"], json!(5499900));
    }

    #[test]
    fn a_blank_description_and_category_are_none() {
        let mut input = document();
        island_mut(&mut input)["description"] = json!("   \n\u{a0}");
        island_mut(&mut input)["category"] = json!("");

        let product = &normalized(input)["product"];
        assert_eq!(product["description"], Value::Null);
        assert_eq!(product["category"], Value::Null);
    }

    #[test]
    fn a_non_ascii_handle_stays_as_the_url_spells_it() {
        let mut input = document();
        island_mut(&mut input)["url"] = json!("/products/%E9%81%8B%E8%B2%BB");
        for index in 0..2 {
            let id = record_variant_mut(&mut input, index)["id"].clone();
            island_offer_mut(&mut input, index)["url"] =
                json!(format!("/products/%E9%81%8B%E8%B2%BB?variant={id}"));
        }

        let product = &normalized(input)["product"];
        assert_eq!(product["handle"], json!("%E9%81%8B%E8%B2%BB"));
        assert_eq!(
            product["url"],
            json!("https://minisforum.hk/products/%E9%81%8B%E8%B2%BB")
        );
    }

    #[test]
    fn a_path_outside_products_fails() {
        let mut input = document();
        island_mut(&mut input)["url"] = json!("/collections/mini-pc");
        for index in 0..2 {
            let id = record_variant_mut(&mut input, index)["id"].clone();
            island_offer_mut(&mut input, index)["url"] =
                json!(format!("/collections/mini-pc?variant={id}"));
        }

        assert!(matches!(
            normalize(input),
            Err(NormalizeError::ProductPathNotUnderProducts { path })
                if path == "/collections/mini-pc"
        ));
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
    fn an_unknown_currency_fails() {
        let mut input = document();
        input["paypal"]["data-currency"] = json!("XXX");

        assert!(matches!(
            normalize(input),
            Err(NormalizeError::UnsupportedCurrency { currency, .. }) if currency == "XXX"
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
    fn the_product_islands_are_counted() {
        let mut none = document();
        none["ld-json"].as_array_mut().expect("islands").remove(0);
        assert!(matches!(
            normalize(none),
            Err(NormalizeError::MissingProductIsland)
        ));

        let mut several = document();
        let island = several["ld-json"][0].clone();
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
    fn the_island_and_the_seal_must_describe_the_same_product() {
        let mut id = document();
        id["seal_product"]["data"]["id"] = json!(1);
        assert!(matches!(
            normalize(id),
            Err(NormalizeError::ProductIdMismatch { seal: 1, .. })
        ));

        let mut title = document();
        title["seal_product"]["data"]["title"] = json!("MS-A2");
        assert!(matches!(
            normalize(title),
            Err(NormalizeError::ProductTitleMismatch { seal, .. }) if seal == "MS-A2"
        ));

        let mut image = document();
        image["seal_product"]["data"]["image_url"] =
            json!("//minisforum.hk/cdn/shop/files/Other.png?v=1&width=300");
        assert!(matches!(
            normalize(image),
            Err(NormalizeError::ProductImageMismatch { .. })
        ));
    }

    #[test]
    fn the_island_and_the_record_must_offer_the_same_variants() {
        let mut input = document();
        island_offer_mut(&mut input, 1)["url"] = json!("/products/minisforum-ms-s1-max?variant=1");

        assert!(matches!(
            normalize(input),
            Err(NormalizeError::VariantIdsMismatch { island, .. })
                if island == [46559370805426, 1]
        ));
    }

    #[test]
    fn an_offer_url_must_name_a_variant_of_this_product() {
        let mut without = document();
        island_offer_mut(&mut without, 0)["url"] = json!("/products/minisforum-ms-s1-max");
        assert!(matches!(
            normalize(without),
            Err(NormalizeError::OfferUrlWithoutVariant { .. })
        ));

        let mut other = document();
        island_offer_mut(&mut other, 0)["url"] = json!("/products/ms-a2?variant=46559370805426");
        assert!(matches!(
            normalize(other),
            Err(NormalizeError::OfferUrlForAnotherProduct { .. })
        ));
    }

    #[test]
    fn the_island_and_the_record_must_agree_on_a_variant() {
        let mut price = document();
        island_offer_mut(&mut price, 0)["price"] = json!(32999.5);
        assert!(matches!(
            normalize(price),
            Err(NormalizeError::VariantFieldMismatch { field, .. }) if field == "price"
        ));

        let mut available = document();
        island_offer_mut(&mut available, 0)["availability"] =
            json!("https://schema.org/OutOfStock");
        assert!(matches!(
            normalize(available),
            Err(NormalizeError::VariantFieldMismatch { field, .. }) if field == "available"
        ));

        let mut sku = document();
        island_offer_mut(&mut sku, 0)
            .as_object_mut()
            .expect("an offer")
            .remove("sku");
        assert!(matches!(
            normalize(sku),
            Err(NormalizeError::VariantFieldMismatch { field, .. }) if field == "sku"
        ));
    }

    #[test]
    fn an_unseen_availability_fails() {
        for value in ["http://schema.org/InStock", "https://schema.org/PreOrder"] {
            let mut input = document();
            island_offer_mut(&mut input, 0)["availability"] = json!(value);

            assert!(matches!(
                normalize(input),
                Err(NormalizeError::UnknownAvailability { value: seen }) if seen == value
            ));
        }
    }

    /// As on `products-運費補價.html`, whose only variant has an empty SKU and whose offer
    /// renders none.
    #[test]
    fn a_missing_offer_sku_matches_an_empty_record_sku() {
        let mut input = default_title_document();
        record_variant_mut(&mut input, 0)["sku"] = json!("");
        sync_selected(&mut input);
        island_mut(&mut input)["sku"] = json!("");
        island_offer_mut(&mut input, 0)
            .as_object_mut()
            .expect("an offer")
            .remove("sku");

        assert_eq!(
            normalized(input)["product"]["variants"][0]["sku"],
            Value::Null
        );
    }

    #[test]
    fn the_selected_variant_must_be_a_listed_one() {
        let mut unknown = document();
        unknown["selected_variant"]["data"]["id"] = json!(1);
        assert!(matches!(
            normalize(unknown),
            Err(NormalizeError::UnknownSelectedVariant { variant_id: 1 })
        ));

        let mut differs = document();
        differs["selected_variant"]["data"]["price"] = json!(1);
        assert!(matches!(
            normalize(differs),
            Err(NormalizeError::SelectedVariantDiffers {
                variant_id: 46559370805426
            })
        ));
    }

    #[test]
    fn the_islands_sku_is_the_selected_variants() {
        let mut input = document();
        island_mut(&mut input)["sku"] = json!("CS-MMSS14D");

        assert!(matches!(
            normalize(input),
            Err(NormalizeError::SelectedSkuMismatch { island, selected })
                if island.as_deref() == Some("CS-MMSS14D")
                    && selected.as_deref() == Some("CS-MMSS1MD")
        ));
    }

    #[test]
    fn option_values_pair_with_the_selectors_labels_by_position() {
        let output = normalized(document());
        assert_eq!(
            output["product"]["variants"][0]["options"],
            json!([
                {"name": "CPU", "value": "AMD Ryzen™ AI Max+ 395"},
                {"name": "規格", "value": "128GB + 2TB SSD"}
            ])
        );

        let mut input = document();
        input["option_names"] = json!([{"text": "CPU:"}]);
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
    fn a_product_without_options_has_neither_option_names_nor_values() {
        let product = &normalized(default_title_document())["product"];

        assert_eq!(product["options"], json!([]));
        let variant = &product["variants"][0];
        assert_eq!(variant["options"], json!([]));
        assert_eq!(variant["title"], Value::Null);
        assert_eq!(variant["name"], json!("MS-S1 Max"));
    }

    #[test]
    fn ids_become_gids_and_a_variant_image_becomes_https() {
        let mut input = document();
        record_variant_mut(&mut input, 0)["featured_image"] = json!({
            "id": 1, "product_id": 8611751919794_u64, "position": 1,
            "created_at": "2026-07-08T11:12:15+08:00", "updated_at": "2026-07-08T11:12:15+08:00",
            "alt": null, "width": 1600, "height": 1600,
            "src": "//minisforum.hk/cdn/shop/files/v.jpg?v=1", "variant_ids": [46559370805426_u64]
        });
        sync_selected(&mut input);

        let variant = &normalized(input)["product"]["variants"][0];
        assert_eq!(
            variant["id"],
            json!("gid://shopify/ProductVariant/46559370805426")
        );
        assert_eq!(
            variant["image"],
            json!("https://minisforum.hk/cdn/shop/files/v.jpg?v=1")
        );
    }
}
