//! Expresses the storefront's product facts in the system's vocabulary. The page states the
//! product across several islands - the theme's ld+json, Shopify's analytics `meta`, and the
//! main section's price block, product form, variant picker and gallery - and they cross here as
//! one product. Where two islands state the same fact, normalization checks that they agree;
//! what nothing needs stays behind as the commented-out exclusion records in `product`.
//!
//! A variant's `price` is the price customers see. On the 40 pages under an automatic discount
//! that is the discounted price for the selected variant, whose Shopify price becomes its
//! compare-at price; the other variants keep their Shopify price.

mod island;
pub mod product;

use super::raw;
use money::{Currency, Money};
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// The storefront the pages are saved from; the product form states its address only as a path.
const ORIGIN: &str = "https://fr.ugreen.com";

#[derive(Debug, Clone, PartialEq, Error)]
pub enum NormalizeError {
    #[error("shop currency `{currency}` is not supported: {reason}")]
    UnsupportedCurrency { currency: String, reason: String },

    #[error("shop currency `{currency}` has no hundredths, the unit Liquid renders amounts in")]
    CurrencyWithoutHundredths { currency: String },

    #[error("`{field}` `{value}` is not a whole number")]
    NotAWholeNumber { field: String, value: String },

    #[error("the document carries no theme product island")]
    MissingProductIsland,

    #[error("the document carries {count} theme product islands; exactly one is read")]
    SeveralProductIslands { count: usize },

    #[error("a {island} ld+json island carries class {class:?}")]
    IslandClassMismatch {
        island: String,
        class: Option<String>,
    },

    #[error("offer URL `{url}` names no numeric `variant`")]
    OfferUrlWithoutVariant { url: String },

    #[error("offer `{offer}` price `{price}` is not a decimal amount")]
    OfferPriceNotDecimal { offer: String, price: String },

    #[error("offer `{offer}` availability `{availability}` is not one the survey saw")]
    UnknownAvailability { offer: String, availability: String },

    #[error("offer `{offer}` is in `{island}`, the shop in `{shop}`")]
    OfferCurrencyMismatch {
        offer: String,
        island: String,
        shop: String,
    },

    #[error("the {island} is for product `{id}`, the analytics for `{analytics}`")]
    ProductIdMismatch {
        island: String,
        id: String,
        analytics: u64,
    },

    #[error("the {island}'s URL `{url}` is not the product's `{expected}`")]
    ProductUrlMismatch {
        island: String,
        url: String,
        expected: String,
    },

    #[error("the island offers variants {island:?}, the analytics {analytics:?}")]
    VariantIdsMismatch {
        island: Vec<u64>,
        analytics: Vec<u64>,
    },

    #[error(
        "variant `{variant_id}` `{field}` is `{island}` on the island, `{analytics}` in the \
         analytics"
    )]
    VariantFieldMismatch {
        variant_id: u64,
        field: String,
        island: String,
        analytics: String,
    },

    #[error("the selected variant `{id}` is not among the analytics variants")]
    SelectedVariantNotListed { id: u64 },

    #[error("the variant picker renders no record of the selected variant")]
    PickerWithoutSelectedVariant,

    #[error("the picker's selected variant disagrees with the page on `{field}`")]
    SelectedVariantMismatch { field: String },

    #[error("the displayed price {displayed} is above the variant's price {listed}")]
    DisplayedPriceAboveListed { displayed: i64, listed: i64 },

    #[error(
        "the displayed price {displayed} is discounted from {listed}, but the struck-through \
         price is {compare}"
    )]
    DiscountedFromAnotherPrice {
        displayed: i64,
        listed: i64,
        compare: i64,
    },

    #[error("the product form's `auto_discount` is `{value}`, not `true`")]
    AutomaticDiscountFlag { value: String },

    #[error(
        "the automatic-discount flag is {flagged}, but the displayed price is {displayed} \
         against the variant's {listed}"
    )]
    AutomaticDiscountMismatch {
        flagged: bool,
        displayed: i64,
        listed: i64,
    },

    #[error("the automatic-discount flag is {flagged}, but the discount code is {code:?}")]
    DiscountCodeMismatch { flagged: bool, code: Option<String> },

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
pub struct UgreenFrNormalizedDocument {
    pub product: product::Product,
}

impl UgreenFrNormalizedDocument {
    pub fn from_raw(
        raw: raw::UgreenFrRawDocument,
    ) -> Result<UgreenFrNormalizedDocument, NormalizeError> {
        Ok(UgreenFrNormalizedDocument {
            product: product::Product::from_raw(raw)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{NormalizeError, UgreenFrNormalizedDocument};
    use crate::ugreenfr::offer_details::v1::raw::UgreenFrRawDocument;
    use crate::ugreenfr::offer_details::v1::raw::tests::{discounted_document, document};
    use html_extractor::serde_json::{self, Value, json};

    fn normalize(document: Value) -> Result<UgreenFrNormalizedDocument, NormalizeError> {
        let raw: UgreenFrRawDocument =
            serde_json::from_value(document).expect("the raw fixture deserializes");
        UgreenFrNormalizedDocument::from_raw(raw)
    }

    fn normalized(document: Value) -> Value {
        serde_json::to_value(normalize(document).expect("the fixture normalizes"))
            .expect("serializes")
    }

    fn eur(amount: i64) -> Value {
        json!({"amount": amount, "currency": "EUR"})
    }

    fn group_variant_mut(document: &mut Value, index: usize) -> &mut Value {
        &mut document["ld-json"][1]["data"]["hasVariant"][index]
    }

    fn analytics_variant_mut(document: &mut Value, index: usize) -> &mut Value {
        &mut document["analytics"]["meta"]["product"]["variants"][index]
    }

    /// The discounted fixture without its discount: the price block shows the Shopify price.
    fn undiscounted_single_variant() -> Value {
        let mut input = discounted_document();
        input["product_form"] = json!({
            "product_id": "8986551157056",
            "product_url": "/products/15532"
        });
        input["price"]["base_price_cents"] = json!("6199");
        input["price"]["base_compare_cents"] = json!("6199");
        input
    }

    #[test]
    fn the_product_is_assembled_from_every_island() {
        let product = &normalized(document())["product"];

        assert_eq!(product["id"], json!("gid://shopify/Product/9526794977600"));
        assert_eq!(product["handle"], json!("45699"));
        assert_eq!(
            product["url"],
            json!("https://fr.ugreen.com/products/45699")
        );
        assert_eq!(product["title"], json!("UGREEN 300W Nexode Chargeur"));
        assert_eq!(product["vendor"], json!("UGREEN FR"));
        assert_eq!(product["product_type"], json!("Nexode Charger"));
        assert_eq!(
            product["category"],
            json!("Adaptateurs secteur et chargeurs")
        );
        assert_eq!(product["options"], json!(["Pack"]));
        assert_eq!(
            product["images"],
            json!([
                "https://fr.ugreen.com/cdn/shop/files/charger.png?v=1738861172",
                "https://fr.ugreen.com/cdn/shop/files/61kMB5x0AfL.jpg?v=1738834470"
            ])
        );
        assert_eq!(product["automatic_discount_code"], Value::Null);
        assert_eq!(
            product["selected_variant"],
            json!({
                "id": "gid://shopify/ProductVariant/50354801180992",
                "options": [{"name": "Pack", "value": "Chargeur"}]
            })
        );
    }

    #[test]
    fn every_variant_carries_its_price_identifiers_and_availability() {
        let variants = &normalized(document())["product"]["variants"];

        assert_eq!(
            variants,
            &json!([
                {"id": "gid://shopify/ProductVariant/50354801180992", "title": "Chargeur",
                    "name": "UGREEN 300W Nexode Chargeur - Chargeur",
                    "sku": "45699", "gtin": "6941876246991",
                    "price": eur(13499), "compare_at_price": eur(17999), "available": true},
                {"id": "gid://shopify/ProductVariant/50354801213760",
                    "title": "+ 2 Câble USB C 100W 1M",
                    "name": "UGREEN 300W Nexode Chargeur - + 2 Câble USB C 100W 1M",
                    "sku": "45699+70427P", "gtin": null,
                    "price": eur(18799), "compare_at_price": null, "available": true}
            ])
        );
    }

    #[test]
    fn an_automatic_discount_prices_the_selected_variant_from_its_shopify_price() {
        let product = &normalized(discounted_document())["product"];

        assert_eq!(product["automatic_discount_code"], json!("DL15532"));
        assert_eq!(product["variants"][0]["price"], eur(5299));
        assert_eq!(product["variants"][0]["compare_at_price"], eur(6199));
        assert_eq!(product["options"], json!([]));
        assert_eq!(product["selected_variant"]["options"], json!([]));
    }

    #[test]
    fn an_automatic_discount_leaves_the_other_variants_at_their_shopify_price() {
        let mut input = document();
        input["price"]["base_price_cents"] = json!("11999");
        input["price"]["base_compare_cents"] = json!("13499");
        input["product_form"]["auto_discount"] = json!("true");
        input["product_form"]["discount_code"] = json!("DL45699");

        let variants = &normalized(input)["product"]["variants"];
        assert_eq!(variants[0]["price"], eur(11999));
        assert_eq!(variants[0]["compare_at_price"], eur(13499));
        assert_eq!(variants[1]["price"], eur(18799));
        assert_eq!(variants[1]["compare_at_price"], Value::Null);
    }

    #[test]
    fn a_struck_through_price_equal_to_the_price_is_no_markdown() {
        let variant = &normalized(undiscounted_single_variant())["product"]["variants"][0];

        assert_eq!(variant["price"], eur(6199));
        assert_eq!(variant["compare_at_price"], Value::Null);
    }

    #[test]
    fn a_struck_through_price_above_the_price_is_the_compare_at_price() {
        let mut input = undiscounted_single_variant();
        input["price"]["base_compare_cents"] = json!("7999");

        let variant = &normalized(input)["product"]["variants"][0];
        assert_eq!(variant["price"], eur(6199));
        assert_eq!(variant["compare_at_price"], eur(7999));
    }

    #[test]
    fn a_displayed_price_above_the_variant_price_fails() {
        let mut input = undiscounted_single_variant();
        input["price"]["base_price_cents"] = json!("7000");

        assert!(matches!(
            normalize(input),
            Err(NormalizeError::DisplayedPriceAboveListed {
                displayed: 7000,
                listed: 6199
            })
        ));
    }

    #[test]
    fn an_automatic_discount_must_strike_through_the_variant_price() {
        let mut input = discounted_document();
        input["price"]["base_compare_cents"] = json!("7999");

        assert!(matches!(
            normalize(input),
            Err(NormalizeError::DiscountedFromAnotherPrice { compare: 7999, .. })
        ));
    }

    #[test]
    fn the_discount_flag_must_agree_with_the_displayed_price() {
        let mut unflagged = discounted_document();
        unflagged["product_form"] = json!({
            "product_id": "8986551157056",
            "product_url": "/products/15532"
        });
        assert!(matches!(
            normalize(unflagged),
            Err(NormalizeError::AutomaticDiscountMismatch { flagged: false, .. })
        ));

        let mut flagged = undiscounted_single_variant();
        flagged["product_form"]["auto_discount"] = json!("true");
        flagged["product_form"]["discount_code"] = json!("DL15532");
        assert!(matches!(
            normalize(flagged),
            Err(NormalizeError::AutomaticDiscountMismatch { flagged: true, .. })
        ));

        let mut other = discounted_document();
        other["product_form"]["auto_discount"] = json!("false");
        assert!(matches!(
            normalize(other),
            Err(NormalizeError::AutomaticDiscountFlag { value }) if value == "false"
        ));
    }

    #[test]
    fn the_discount_code_comes_with_the_flag() {
        let mut without_code = discounted_document();
        without_code["product_form"]
            .as_object_mut()
            .expect("an object")
            .remove("discount_code");
        assert!(matches!(
            normalize(without_code),
            Err(NormalizeError::DiscountCodeMismatch {
                flagged: true,
                code: None
            })
        ));

        let mut without_flag = undiscounted_single_variant();
        without_flag["product_form"]["discount_code"] = json!("DL15532");
        assert!(matches!(
            normalize(without_flag),
            Err(NormalizeError::DiscountCodeMismatch { flagged: false, .. })
        ));
    }

    /// As on `products-ugreen-cable-usb-c-vers-usb-c-pd-100w-5a-20v-cable-nylon-tresse.html`.
    #[test]
    fn a_barcode_ending_in_a_no_break_space_is_trimmed() {
        let mut input = document();
        group_variant_mut(&mut input, 0)["gtin"] = json!("6941876246991\u{a0}");
        input["selected_variant"]["data"]["barcode"] = json!("6941876246991\u{a0}");

        assert_eq!(
            normalized(input)["product"]["variants"][0]["gtin"],
            json!("6941876246991")
        );
    }

    #[test]
    fn a_barcode_that_is_not_digits_is_no_gtin() {
        let mut input = document();
        group_variant_mut(&mut input, 1)["gtin"] = json!("N/A");

        assert_eq!(
            normalized(input)["product"]["variants"][1]["gtin"],
            Value::Null
        );
    }

    /// As on the gift cards (`products-ugreen-carte-cadeau-*.html`).
    #[test]
    fn a_variant_without_a_sku_has_none() {
        let mut input = document();
        group_variant_mut(&mut input, 1)
            .as_object_mut()
            .expect("a variant")
            .remove("sku");
        analytics_variant_mut(&mut input, 1)["sku"] = json!("");

        assert_eq!(
            normalized(input)["product"]["variants"][1]["sku"],
            Value::Null
        );
    }

    #[test]
    fn an_out_of_stock_offer_is_an_unavailable_variant() {
        let mut input = document();
        group_variant_mut(&mut input, 1)["offers"]["availability"] =
            json!("http://schema.org/OutOfStock");

        assert_eq!(
            normalized(input)["product"]["variants"][1]["available"],
            json!(false)
        );
    }

    #[test]
    fn an_unknown_availability_fails() {
        let mut input = document();
        group_variant_mut(&mut input, 1)["offers"]["availability"] =
            json!("http://schema.org/PreOrder");

        assert!(matches!(
            normalize(input),
            Err(NormalizeError::UnknownAvailability { availability, .. })
                if availability == "http://schema.org/PreOrder"
        ));
    }

    #[test]
    fn the_review_island_is_ignored_and_only_by_its_class() {
        let mut without_reviews = document();
        without_reviews["ld-json"]
            .as_array_mut()
            .expect("islands")
            .truncate(2);
        assert_eq!(
            normalized(without_reviews)["product"],
            normalized(document())["product"]
        );

        let mut unclassed = document();
        unclassed["ld-json"][2]
            .as_object_mut()
            .expect("a script")
            .remove("class");
        assert!(matches!(
            normalize(unclassed),
            Err(NormalizeError::IslandClassMismatch { island, class: None }) if island == "review"
        ));

        let mut classed = document();
        classed["ld-json"][1]["class"] = json!("jdgm-server-jld");
        assert!(matches!(
            normalize(classed),
            Err(NormalizeError::IslandClassMismatch { island, .. }) if island == "product"
        ));
    }

    #[test]
    fn exactly_one_product_island_is_read() {
        let mut missing = document();
        missing["ld-json"]
            .as_array_mut()
            .expect("islands")
            .remove(1);
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
    fn the_islands_must_name_the_same_product() {
        for island in ["price", "product_form"] {
            let mut input = document();
            input[island]["product_id"] = json!("1");

            assert!(
                matches!(
                    normalize(input),
                    Err(NormalizeError::ProductIdMismatch { .. })
                ),
                "{island}"
            );
        }

        let mut group = document();
        group["ld-json"][1]["data"]["productGroupID"] = json!("1");
        assert!(matches!(
            normalize(group),
            Err(NormalizeError::ProductIdMismatch { island, .. }) if island == "ld+json island"
        ));

        let mut form_url = document();
        form_url["product_form"]["product_url"] = json!("/products/other");
        assert!(matches!(
            normalize(form_url),
            Err(NormalizeError::ProductUrlMismatch { island, .. }) if island == "product form"
        ));

        let mut island_url = document();
        island_url["ld-json"][1]["data"]["url"] = json!("https://fr.ugreen.com/products/other");
        assert!(matches!(
            normalize(island_url),
            Err(NormalizeError::ProductUrlMismatch { island, .. }) if island == "ld+json island"
        ));
    }

    #[test]
    fn the_island_and_the_analytics_must_offer_the_same_variants() {
        let mut input = document();
        group_variant_mut(&mut input, 1)["offers"]["url"] =
            json!("https://fr.ugreen.com/products/45699?variant=1");

        assert!(matches!(
            normalize(input),
            Err(NormalizeError::VariantIdsMismatch { island, .. })
                if island == [50354801180992, 1]
        ));

        let mut no_variant = document();
        group_variant_mut(&mut no_variant, 1)["offers"]["url"] =
            json!("https://fr.ugreen.com/products/45699");
        assert!(matches!(
            normalize(no_variant),
            Err(NormalizeError::OfferUrlWithoutVariant { .. })
        ));
    }

    #[test]
    fn the_island_and_the_analytics_must_agree_on_a_variant() {
        let mut price = document();
        group_variant_mut(&mut price, 1)["offers"]["price"] = json!("187.00");
        assert!(matches!(
            normalize(price),
            Err(NormalizeError::VariantFieldMismatch { field, .. }) if field == "price"
        ));

        let mut sku = document();
        group_variant_mut(&mut sku, 1)["sku"] = json!("45699");
        assert!(matches!(
            normalize(sku),
            Err(NormalizeError::VariantFieldMismatch { field, .. }) if field == "sku"
        ));
    }

    #[test]
    fn an_island_price_that_is_not_decimal_fails() {
        let mut input = document();
        group_variant_mut(&mut input, 1)["offers"]["price"] = json!("187,99");

        assert!(matches!(
            normalize(input),
            Err(NormalizeError::OfferPriceNotDecimal { price, .. }) if price == "187,99"
        ));
    }

    #[test]
    fn an_island_in_another_currency_fails() {
        let mut input = document();
        group_variant_mut(&mut input, 1)["offers"]["priceCurrency"] = json!("GBP");

        assert!(matches!(
            normalize(input),
            Err(NormalizeError::OfferCurrencyMismatch { island, shop, .. })
                if island == "GBP" && shop == "EUR"
        ));
    }

    #[test]
    fn the_selected_variant_must_be_listed_and_agree() {
        let mut unlisted = document();
        unlisted["selected_variant_id"]["value"] = json!("1");
        assert!(matches!(
            normalize(unlisted),
            Err(NormalizeError::SelectedVariantNotListed { id: 1 })
        ));

        for (field, value) in [
            ("id", json!(50354801213760_u64)),
            ("price", json!(999)),
            ("available", json!(false)),
            ("sku", json!("other")),
            ("barcode", json!("6941876246992")),
            ("compare_at_price", json!(19999)),
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

        let mut missing = document();
        missing
            .as_object_mut()
            .expect("an object")
            .remove("selected_variant");
        assert!(matches!(
            normalize(missing),
            Err(NormalizeError::PickerWithoutSelectedVariant)
        ));
    }

    /// As on `products-cable-usb-c-c-pd-240w.html`, which opens on a later variant.
    #[test]
    fn the_price_block_describes_the_selected_variant_wherever_it_is() {
        let mut input = document();
        input["selected_variant_id"]["value"] = json!("50354801213760");
        input["price"]["base_price_cents"] = json!("18799");
        input["price"]["base_compare_cents"] = json!("19999");
        let record = &mut input["selected_variant"]["data"];
        record["id"] = json!(50354801213760_u64);
        record["price"] = json!(18799);
        record["compare_at_price"] = json!(19999);
        record["sku"] = json!("45699+70427P");
        record["barcode"] = Value::Null;
        record["options"] = json!(["+ 2 Câble USB C 100W 1M"]);

        let product = &normalized(input)["product"];
        assert_eq!(product["variants"][0]["compare_at_price"], Value::Null);
        assert_eq!(product["variants"][1]["compare_at_price"], eur(19999));
        assert_eq!(
            product["selected_variant"]["options"],
            json!([{"name": "Pack", "value": "+ 2 Câble USB C 100W 1M"}])
        );
    }

    #[test]
    fn option_values_pair_with_the_legends_by_position() {
        let mut input = document();
        input["option_names"] = json!([{"text": "Pack:"}, {"text": "Couleur:"}]);

        assert!(matches!(
            normalize(input),
            Err(NormalizeError::OptionCountMismatch {
                offered: 2,
                selected: 1,
                ..
            })
        ));
    }

    #[test]
    fn an_unknown_or_jpy_currency_fails() {
        let mut unknown = document();
        unknown["price"]["currency"] = json!("XXX");
        assert!(matches!(
            normalize(unknown),
            Err(NormalizeError::UnsupportedCurrency { currency, .. }) if currency == "XXX"
        ));

        let mut jpy = document();
        jpy["price"]["currency"] = json!("JPY");
        assert!(matches!(
            normalize(jpy),
            Err(NormalizeError::CurrencyWithoutHundredths { currency }) if currency == "JPY"
        ));
    }

    #[test]
    fn a_price_attribute_or_variant_id_that_is_not_a_number_fails() {
        let mut price = document();
        price["price"]["base_price_cents"] = json!("134,99");
        assert!(matches!(
            normalize(price),
            Err(NormalizeError::NotAWholeNumber { field, .. }) if field == "base_price_cents"
        ));

        let mut id = document();
        id["selected_variant_id"]["value"] = json!("");
        assert!(matches!(
            normalize(id),
            Err(NormalizeError::NotAWholeNumber { field, .. }) if field == "selected_variant_id"
        ));
    }

    #[test]
    fn an_empty_product_type_or_category_is_absent() {
        let mut input = document();
        input["analytics"]["meta"]["product"]["type"] = json!("");
        input["ld-json"][1]["data"]["category"] = json!("");

        let product = &normalized(input)["product"];
        assert_eq!(product["product_type"], Value::Null);
        assert_eq!(product["category"], Value::Null);
    }
}
