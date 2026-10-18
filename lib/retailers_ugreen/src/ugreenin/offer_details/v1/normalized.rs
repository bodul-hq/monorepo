//! Expresses the storefront's product facts in the system's vocabulary. Liquid's product record
//! crosses as the product. Shopify's analytics `meta` restates its ids, prices and SKUs, and the
//! theme's schema.org island its prices, availability, SKUs and barcodes, so normalization
//! checks both agree with it; from the island it keeps the page's URL and `category`. What
//! nothing needs stays behind as the commented-out exclusion records in `product`.
//!
//! The page's other islands (`Organization`, `BreadcrumbList`, `FAQPage` and Judge.me's review
//! `Product`) are neither checked nor kept. They are typed in `raw`, so that a change in their
//! shape fails the page, but none says anything about the offer.
//!
//! The storefront serves every product in two locales, `/products/<handle>` in English and
//! `/hi/products/<handle>` in Hindi, from one record. Each page normalizes to its own document,
//! tagged with its locale and carrying its own URL; the two documents of a product are not
//! merged. The island's URL is the only statement of the page's address, so the locale is read
//! from it; see [`Locale`].

mod analytics;
mod island;
pub mod product;

use super::raw;
use money::{Currency, Money};
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// The storefront the pages are saved from; the record states only the product's handle.
const ORIGIN: &str = "https://www.ugreenindia.com";

/// The page's language, which decides its URL prefix. 704 saved pages each; the saved file
/// names carry the same split (`products-*.html`, `hi-products-*.html`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Locale {
    /// English, the unprefixed default.
    En,
    /// Hindi, under `/hi`.
    Hi,
}

impl Locale {
    const ALL: [Locale; 2] = [Locale::En, Locale::Hi];

    fn prefix(self) -> &'static str {
        match self {
            Locale::En => "",
            Locale::Hi => "/hi",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Error)]
pub enum NormalizeError {
    #[error("shop currency `{currency}` is not supported: {reason}")]
    UnsupportedCurrency { currency: String, reason: String },

    #[error("shop currency `{currency}` has no hundredths, the unit Liquid renders amounts in")]
    CurrencyWithoutHundredths { currency: String },

    #[error("the document carries no product island")]
    MissingProductIsland,

    #[error("the document carries {count} product islands; one is read")]
    SeveralProductIslands { count: usize },

    #[error("page URL `{url}` is not a product page in any of the storefront's locales")]
    PageUrlOutsideLocales { url: String },

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

    #[error("product `{field}` is `{analytics}` in the analytics, `{record}` on the record")]
    AnalyticsProductMismatch {
        field: String,
        analytics: String,
        record: String,
    },

    #[error("the analytics list variants {analytics:?}, the record {record:?}")]
    AnalyticsVariantIdsMismatch {
        analytics: Vec<u64>,
        record: Vec<u64>,
    },

    #[error(
        "variant `{variant_id}` `{field}` is `{analytics}` in the analytics, `{record}` on the \
         record"
    )]
    AnalyticsVariantMismatch {
        variant_id: u64,
        field: String,
        analytics: String,
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
// unit is a hundredth takes them as they are. INR's is (paise).
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

// The locale whose product pages `url` is among.
fn page_locale(url: &str) -> Result<Locale, NormalizeError> {
    let path = url.strip_prefix(ORIGIN).unwrap_or_default();
    Locale::ALL
        .into_iter()
        .find(|locale| {
            path.strip_prefix(locale.prefix())
                .and_then(|rest| rest.strip_prefix("/products/"))
                .is_some_and(|handle| !handle.is_empty())
        })
        .ok_or_else(|| NormalizeError::PageUrlOutsideLocales {
            url: url.to_string(),
        })
}

// Handles can be any text (`...-thunderbolt™-5-...` is one on the saved pages), and the
// storefront spells the URL's path percent-encoded, in upper-case hex.
fn product_url(locale: Locale, handle: &str) -> String {
    let mut segment = String::with_capacity(handle.len());
    for byte in handle.bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'.' | b'_' | b'~') {
            segment.push(char::from(byte));
        } else {
            segment.push_str(&format!("%{byte:02X}"));
        }
    }

    format!("{ORIGIN}{}/products/{segment}", locale.prefix())
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

fn non_empty(value: String) -> Option<String> {
    (!value.is_empty()).then_some(value)
}

// A barcode is a GTIN only where it is all digits once surrounding whitespace (`\u{a0}` among
// it) is trimmed. The shop stores 30 variants' barcodes per locale as spreadsheet-mangled
// numbers (`6.96E+12`, `6.94E+12`); those are no GTIN, and the page still normalizes.
fn gtin(barcode: Option<String>) -> Option<String> {
    let barcode = barcode?;
    let trimmed = barcode.trim();
    (!trimmed.is_empty() && trimmed.bytes().all(|b| b.is_ascii_digit()))
        .then(|| trimmed.to_string())
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UgreenInNormalizedDocument {
    /// The page's locale, from its URL.
    pub locale: Locale,
    pub product: product::Product,
}

impl UgreenInNormalizedDocument {
    pub fn from_raw(
        raw: raw::UgreenInRawDocument,
    ) -> Result<UgreenInNormalizedDocument, NormalizeError> {
        let currency = shop_currency(&raw.og_currency.content)?;
        let island = island::Island::from_raw(raw.ld_json)?;
        let locale = page_locale(&island.url)?;
        analytics::check(&raw.analytics.meta.product, &raw.product_json.data)?;

        Ok(UgreenInNormalizedDocument {
            locale,
            product: product::Product::from_raw(raw.product_json.data, island, currency, locale)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{Locale, NormalizeError, UgreenInNormalizedDocument, gtin, page_locale};
    use super::{product_url, shop_currency};
    use crate::ugreenin::offer_details::v1::raw::UgreenInRawDocument;
    use crate::ugreenin::offer_details::v1::raw::tests::document;
    use html_extractor::serde_json::{self, Value, json};

    fn normalize(document: Value) -> Result<UgreenInNormalizedDocument, NormalizeError> {
        let raw: UgreenInRawDocument =
            serde_json::from_value(document).expect("the raw fixture deserializes");
        UgreenInNormalizedDocument::from_raw(raw)
    }

    fn normalized(document: Value) -> Value {
        serde_json::to_value(normalize(document).expect("the fixture normalizes"))
            .expect("serializes")
    }

    fn inr(amount: i64) -> Value {
        json!({"amount": amount, "currency": "INR"})
    }

    fn record_mut(document: &mut Value) -> &mut Value {
        &mut document["product_json"]["data"]
    }

    fn record_variant_mut(document: &mut Value, index: usize) -> &mut Value {
        &mut document["product_json"]["data"]["variants"][index]
    }

    fn island_mut(document: &mut Value) -> &mut Value {
        &mut document["ld-json"][2]["data"]
    }

    fn island_offer_mut(document: &mut Value, index: usize) -> &mut Value {
        &mut document["ld-json"][2]["data"]["hasVariant"][index]["offers"]
    }

    fn analytics_mut(document: &mut Value) -> &mut Value {
        &mut document["analytics"]["meta"]["product"]
    }

    /// The fixture as its Hindi page renders it: every URL the island carries under `/hi`.
    fn hindi(mut document: Value) -> Value {
        let island = island_mut(&mut document);
        let text = island.to_string().replace(
            "https://www.ugreenindia.com/products/",
            "https://www.ugreenindia.com/hi/products/",
        );
        *island = serde_json::from_str(&text).expect("json");
        document
    }

    #[test]
    fn the_product_is_the_record_with_the_islands_url_and_category() {
        let output = normalized(document());
        let product = &output["product"];

        assert_eq!(output["locale"], json!("en"));
        assert_eq!(product["id"], json!("gid://shopify/Product/7702424813667"));
        assert_eq!(
            product["handle"],
            json!("ugreen-ergonomic-mouse-pad-black-25245")
        );
        assert_eq!(
            product["url"],
            json!("https://www.ugreenindia.com/products/ugreen-ergonomic-mouse-pad-black-25245")
        );
        assert_eq!(
            product["title"],
            json!("UGREEN LP668 Ergonomic Mouse Pad with Wrist Rest")
        );
        assert_eq!(product["description"], json!("<p>d</p>"));
        assert_eq!(product["vendor"], json!("UGREEN"));
        assert_eq!(product["product_type"], json!("Mouse Pad"));
        assert_eq!(product["category"], json!("Mouse Pads"));
        assert_eq!(product["options"], json!(["Color"]));
        assert_eq!(
            product["images"],
            json!(["https://www.ugreenindia.com/cdn/shop/files/LP668-Black.png?v=1763832672"])
        );
        assert_eq!(product["variants"].as_array().map(Vec::len), Some(3));
    }

    #[test]
    fn a_variant_carries_its_paise_its_markdown_and_its_options() {
        let variant = &normalized(document())["product"]["variants"][0];

        assert_eq!(
            variant,
            &json!({
                "id": "gid://shopify/ProductVariant/43454938677347",
                "title": "Black",
                "name": "UGREEN LP668 Ergonomic Mouse Pad with Wrist Rest - Black",
                "sku": "25245",
                "gtin": "6941876222452",
                "price": inr(99900),
                "compare_at_price": inr(222000),
                "available": true,
                "requires_shipping": true,
                "taxable": true,
                "weight": null,
                "image": null,
                "options": [{"name": "Color", "value": "Black"}]
            })
        );
    }

    #[test]
    fn a_hindi_page_is_tagged_hi_and_keeps_its_own_url() {
        let output = normalized(hindi(document()));

        assert_eq!(output["locale"], json!("hi"));
        assert_eq!(
            output["product"]["url"],
            json!("https://www.ugreenindia.com/hi/products/ugreen-ergonomic-mouse-pad-black-25245")
        );
    }

    #[test]
    fn the_locale_is_read_from_the_page_url() {
        let origin = "https://www.ugreenindia.com";
        assert_eq!(page_locale(&format!("{origin}/products/a")), Ok(Locale::En));
        assert_eq!(
            page_locale(&format!("{origin}/hi/products/a")),
            Ok(Locale::Hi)
        );

        for url in [
            format!("{origin}/fr/products/a"),
            format!("{origin}/products/"),
            format!("{origin}/collections/a"),
            "https://ugreen.com/products/a".to_string(),
        ] {
            assert_eq!(
                page_locale(&url),
                Err(NormalizeError::PageUrlOutsideLocales { url: url.clone() })
            );
        }
    }

    #[test]
    fn a_page_outside_the_locales_fails() {
        let mut input = document();
        island_mut(&mut input)["url"] = json!("https://www.ugreenindia.com/fr/products/x");

        assert!(matches!(
            normalize(input),
            Err(NormalizeError::PageUrlOutsideLocales { .. })
        ));
    }

    #[test]
    fn a_url_keeps_plain_handles_and_encodes_the_rest() {
        assert_eq!(
            product_url(Locale::En, "a-b.c_d~e1"),
            "https://www.ugreenindia.com/products/a-b.c_d~e1"
        );
        assert_eq!(
            product_url(Locale::Hi, "thunderbolt™-5"),
            "https://www.ugreenindia.com/hi/products/thunderbolt%E2%84%A2-5"
        );
    }

    #[test]
    fn a_barcode_is_a_gtin_only_where_it_is_all_digits() {
        let barcode = |value: &str| gtin(Some(value.to_string()));

        assert_eq!(barcode("6941876222452"), Some("6941876222452".to_string()));
        assert_eq!(barcode("694187622245"), Some("694187622245".to_string()));
        assert_eq!(
            barcode(" 6941876222452\u{a0}"),
            Some("6941876222452".to_string())
        );
        assert_eq!(barcode("6.96E+12"), None);
        assert_eq!(barcode("6.94E+12"), None);
        assert_eq!(barcode(""), None);
        assert_eq!(barcode("\u{a0}"), None);
        assert_eq!(gtin(None), None);
    }

    #[test]
    fn a_mangled_barcode_crosses_as_no_gtin_without_failing_the_page() {
        let mut input = document();
        record_variant_mut(&mut input, 0)["barcode"] = json!("6.96E+12");
        island_mut(&mut input)["hasVariant"][0]["gtin"] = json!("6.96E+12");

        let variant = &normalized(input)["product"]["variants"][0];
        assert_eq!(variant["gtin"], Value::Null);
    }

    #[test]
    fn a_null_compare_at_price_is_no_markdown() {
        let mut input = document();
        record_variant_mut(&mut input, 1)["compare_at_price"] = Value::Null;

        let variants = &normalized(input)["product"]["variants"];
        assert_eq!(variants[1]["compare_at_price"], Value::Null);
        assert_eq!(variants[0]["compare_at_price"], inr(222000));
    }

    #[test]
    fn a_zero_price_is_no_price() {
        let mut input = document();
        record_variant_mut(&mut input, 0)["price"] = json!(0);
        record_variant_mut(&mut input, 0)["compare_at_price"] = json!(0);
        island_offer_mut(&mut input, 0)["price"] = json!("0.00");
        analytics_mut(&mut input)["variants"][0]["price"] = json!(0);

        let variant = &normalized(input)["product"]["variants"][0];
        assert_eq!(variant["price"], Value::Null);
        assert_eq!(variant["compare_at_price"], Value::Null);
    }

    #[test]
    fn an_out_of_stock_variant_is_unavailable() {
        let mut input = document();
        record_variant_mut(&mut input, 2)["available"] = json!(false);
        island_offer_mut(&mut input, 2)["availability"] = json!("http://schema.org/OutOfStock");

        let variants = &normalized(input)["product"]["variants"];
        assert_eq!(variants[2]["available"], json!(false));
        assert_eq!(variants[0]["available"], json!(true));
    }

    #[test]
    fn a_weight_and_a_variant_image_cross() {
        let mut input = document();
        record_variant_mut(&mut input, 0)["weight"] = json!(150);
        record_variant_mut(&mut input, 0)["featured_image"] = json!({
            "id": 1, "product_id": 7702424813667_u64, "position": 1,
            "created_at": "2024-12-12T16:19:10+05:30", "updated_at": "2024-12-12T16:19:10+05:30",
            "alt": null, "width": 800, "height": 800,
            "src": "//www.ugreenindia.com/cdn/shop/files/v.png?v=1",
            "variant_ids": [43454938677347_u64]
        });

        let variant = &normalized(input)["product"]["variants"][0];
        assert_eq!(variant["weight"], json!(150));
        assert_eq!(
            variant["image"],
            json!("https://www.ugreenindia.com/cdn/shop/files/v.png?v=1")
        );
    }

    #[test]
    fn a_single_variant_product_island_is_read() {
        let mut input = document();
        record_mut(&mut input)["variants"]
            .as_array_mut()
            .expect("variants")
            .truncate(1);
        analytics_mut(&mut input)["variants"]
            .as_array_mut()
            .expect("variants")
            .truncate(1);
        input["ld-json"][2]["data"] =
            crate::ugreenin::offer_details::v1::raw::ldjson::tests::product();

        let product = &normalized(input)["product"];
        assert_eq!(product["category"], json!("Mouse Pads"));
        assert_eq!(product["variants"].as_array().map(Vec::len), Some(1));
    }

    #[test]
    fn the_shop_currency_must_have_hundredths() {
        assert!(shop_currency("INR").is_ok());
        assert_eq!(
            shop_currency("JPY"),
            Err(NormalizeError::CurrencyWithoutHundredths {
                currency: "JPY".to_string()
            })
        );
        assert!(matches!(
            shop_currency("XXX"),
            Err(NormalizeError::UnsupportedCurrency { currency, .. }) if currency == "XXX"
        ));
    }

    #[test]
    fn the_currency_is_the_open_graph_tags() {
        let mut input = document();
        input["og_currency"]["content"] = json!("JPY");

        assert!(matches!(
            normalize(input),
            Err(NormalizeError::CurrencyWithoutHundredths { currency }) if currency == "JPY"
        ));
    }

    /// Judge.me's review `Product` stays on the page, and is not taken for the product island.
    #[test]
    fn a_page_without_a_product_island_fails() {
        let mut input = document();
        input["ld-json"].as_array_mut().expect("islands").remove(2);
        assert_eq!(input["ld-json"][3]["data"]["@type"], json!("Product"));

        assert_eq!(normalize(input), Err(NormalizeError::MissingProductIsland));
    }

    #[test]
    fn several_product_islands_fail() {
        let mut input = document();
        let island = input["ld-json"][2].clone();
        input["ld-json"]
            .as_array_mut()
            .expect("islands")
            .push(island);

        assert_eq!(
            normalize(input),
            Err(NormalizeError::SeveralProductIslands { count: 2 })
        );
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
        let mut id = document();
        island_mut(&mut id)["productGroupID"] = json!("1");
        assert!(matches!(
            normalize(id),
            Err(NormalizeError::IslandProductMismatch { .. })
        ));

        let mut url = document();
        island_mut(&mut url)["url"] = json!("https://www.ugreenindia.com/products/other");
        assert!(matches!(
            normalize(url),
            Err(NormalizeError::IslandProductMismatch { .. })
        ));
    }

    #[test]
    fn an_offer_url_without_a_variant_fails() {
        let mut input = document();
        island_offer_mut(&mut input, 0)["url"] =
            json!("https://www.ugreenindia.com/products/ugreen-ergonomic-mouse-pad-black-25245");

        assert!(matches!(
            normalize(input),
            Err(NormalizeError::OfferUrlWithoutVariant { .. })
        ));
    }

    #[test]
    fn an_offer_price_that_is_not_decimal_fails() {
        let mut input = document();
        island_offer_mut(&mut input, 0)["price"] = json!("999,00");

        assert!(matches!(
            normalize(input),
            Err(NormalizeError::OfferPriceNotDecimal { price, .. }) if price == "999,00"
        ));
    }

    #[test]
    fn the_island_and_the_record_must_offer_the_same_variants() {
        let mut input = document();
        island_offer_mut(&mut input, 1)["url"] = json!(
            "https://www.ugreenindia.com/products/ugreen-ergonomic-mouse-pad-black-25245?variant=1"
        );

        assert!(matches!(
            normalize(input),
            Err(NormalizeError::VariantIdsMismatch { island, .. })
                if island == [43454938677347, 1, 43742608719971]
        ));
    }

    #[test]
    fn the_island_and_the_record_must_agree_on_a_variant() {
        type InvalidCase = (&'static str, fn(&mut Value));
        let cases: [InvalidCase; 4] = [
            ("price", |input| {
                island_offer_mut(input, 0)["price"] = json!("1000.00")
            }),
            ("available", |input| {
                island_offer_mut(input, 0)["availability"] = json!("http://schema.org/OutOfStock")
            }),
            ("sku", |input| {
                island_mut(input)["hasVariant"][0]["sku"] = json!("25246")
            }),
            ("gtin", |input| {
                island_mut(input)["hasVariant"][1]["gtin"] = json!("6941876222453")
            }),
        ];

        for (expected, change) in cases {
            let mut input = document();
            change(&mut input);
            assert!(
                matches!(
                    normalize(input),
                    Err(NormalizeError::VariantFieldMismatch { ref field, .. }) if field == expected
                ),
                "{expected}"
            );
        }
    }

    #[test]
    fn the_analytics_and_the_record_must_be_for_the_same_product() {
        let mut id = document();
        analytics_mut(&mut id)["id"] = json!(1);
        assert!(matches!(
            normalize(id),
            Err(NormalizeError::AnalyticsProductMismatch { field, .. }) if field == "id"
        ));

        let mut handle = document();
        analytics_mut(&mut handle)["handle"] = json!("other");
        assert!(matches!(
            normalize(handle),
            Err(NormalizeError::AnalyticsProductMismatch { field, .. }) if field == "handle"
        ));
    }

    #[test]
    fn the_analytics_and_the_record_must_list_the_same_variants_in_order() {
        let mut input = document();
        analytics_mut(&mut input)["variants"]
            .as_array_mut()
            .expect("variants")
            .swap(0, 1);

        assert!(matches!(
            normalize(input),
            Err(NormalizeError::AnalyticsVariantIdsMismatch { analytics, .. })
                if analytics == [43742608687203, 43454938677347, 43742608719971]
        ));
    }

    #[test]
    fn the_analytics_and_the_record_must_agree_on_price_and_sku() {
        let mut price = document();
        analytics_mut(&mut price)["variants"][1]["price"] = json!(100000);
        assert!(matches!(
            normalize(price),
            Err(NormalizeError::AnalyticsVariantMismatch { field, variant_id, .. })
                if field == "price" && variant_id == 43742608687203
        ));

        let mut sku = document();
        analytics_mut(&mut sku)["variants"][2]["sku"] = json!("25246");
        assert!(matches!(
            normalize(sku),
            Err(NormalizeError::AnalyticsVariantMismatch { field, .. }) if field == "sku"
        ));
    }

    #[test]
    fn option_values_pair_with_the_products_names_by_position() {
        let mut input = document();
        record_mut(&mut input)["options"] = json!(["Color", "Size"]);

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
    fn the_normalized_document_reads_back() {
        let output = normalize(hindi(document())).expect("the fixture normalizes");
        let json = serde_json::to_string(&output).expect("serializes");

        let read: UgreenInNormalizedDocument = serde_json::from_str(&json).expect("reads back");
        assert_eq!(read, output);
    }
}
