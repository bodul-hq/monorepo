//! Expresses extracted product facts in the system's vocabulary. Storefront-specific names and
//! types are normalized, and retailer-only metadata not currently needed is dropped (see the
//! commented-out types below, kept as exclusion records rather than deleted).

use super::raw;
use chrono::{DateTime, Utc};
use html_extractor::serde_json::Value;
use money::{Money, MoneyError};
use retailer_sauce::item_condition::ItemCondition;
use serde::{Deserialize, Serialize};
use std::num::ParseFloatError;
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Error)]
pub enum NormalizeError {
    #[error("no `Product` ld+json island on the document")]
    MissingProductIsland,

    #[error("publishedAt `{value}` is not an RFC 3339 instant: {source}")]
    PublishedAtNotAnInstant {
        value: String,
        source: chrono::ParseError,
    },

    #[error("`{path}` is not a list")]
    NotAList { path: String },

    #[error("a `{block}` entry has no string `{field}`: {entry}")]
    EntryFieldNotAString {
        block: String,
        field: String,
        entry: Value,
    },

    #[error("a `discountCodeApp` entry has no `{field}`: {entry}")]
    MissingDiscountField { field: String, entry: Value },

    #[error("`{field}` `{value}` is not a number: {source}")]
    NotANumber {
        field: String,
        value: String,
        source: ParseFloatError,
    },

    #[error("`{field}` `{value}` is not an RFC 3339 instant: {source}")]
    NotAnInstant {
        field: String,
        value: String,
        source: chrono::ParseError,
    },

    #[error("unknown `value_type` `{value_type}`: {entry}")]
    UnknownDiscountValueType { value_type: String, entry: Value },

    #[error("`{field}` is not money: {source:?}")]
    NotMoney { field: String, source: MoneyError },
}

// Variant prices omit currency; AUD is supplied by this storefront's scope.
const VARIANT_CURRENCY: &str = "AUD";

fn money(amount: f64, currency: &str, field: &str) -> Result<Money, NormalizeError> {
    Money::from_raw(amount, currency).map_err(|source| NormalizeError::NotMoney {
        field: field.to_string(),
        source,
    })
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AnkerAuNormalizedDocument {
    pub ld_json: ldjson::LdJson,
    pub product: ProductRecord,
    pub variants: Vec<Variant>,

    pub canonical_url: String,
}

impl AnkerAuNormalizedDocument {
    pub fn from_raw(raw: raw::AnkerAuRawDocument) -> Result<Self, NormalizeError> {
        let ld_json = ldjson::LdJson::from_raw(raw.ld_json)?;

        let page_props = raw.next_data.data.props.page_props;
        let canonical_url = page_props.seo.canonical;

        let mut raw_product = page_props.product;
        let variants = variants(std::mem::take(&mut raw_product.variants))?;
        let product = ProductRecord::from_raw(raw_product, &page_props.metafields)?;

        Ok(AnkerAuNormalizedDocument {
            ld_json,
            product,
            variants,
            canonical_url,
        })
    }
}

fn variants(raw: Vec<raw::nextdata::product::Variant>) -> Result<Vec<Variant>, NormalizeError> {
    raw.into_iter().map(Variant::from_raw).collect()
}

pub mod ldjson {

    use super::NormalizeError;
    use super::raw;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    pub struct LdJson {
        pub product: Product,
    }

    impl LdJson {
        pub(crate) fn from_raw(
            items: Vec<raw::ldjsonitem::LdJsonItem>,
        ) -> Result<LdJson, NormalizeError> {
            let mut product: Option<Product> = None;

            for item in items {
                match item.data {
                    raw::ldjsonitem::LdJson::Product(island) => {
                        // First island wins; no document in the subset carries two.
                        if product.is_none() {
                            product = Some(Product::from_raw(island)?);
                        }
                    }
                    raw::ldjsonitem::LdJson::Corporation(_) => {}
                    raw::ldjsonitem::LdJson::BreadcrumbList(_) => {}
                    raw::ldjsonitem::LdJson::FaqPage(_) => {}
                }
            }

            let product = product.ok_or(NormalizeError::MissingProductIsland)?;

            Ok(LdJson { product })
        }
    }

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    pub struct Product {
        pub brand: String,
        pub description: String,
        pub images: Vec<String>,
        pub gtin: Option<String>,
        pub sku: String,
        pub offer: offer::Offer,
    }

    impl Product {
        pub(crate) fn from_raw(raw: raw::ldjsonitem::Product) -> Result<Product, NormalizeError> {
            let price = super::money(raw.offers.price, &raw.offers.price_currency, "offers.price")?;

            let offer = offer::Offer {
                item_condition: super::ItemCondition::from_schema_org(raw.offers.item_condition),
                price,
                url: raw.offers.url,
            };

            Ok(Product {
                brand: raw.brand.name,

                description: raw.description,
                images: raw.image,
                gtin: raw.gtin,
                sku: raw.sku,
                offer,
            })
        }
    }

    pub mod offer {
        use money::Money;
        pub use retailer_sauce::item_condition::ItemCondition;
        use serde::{Deserialize, Serialize};

        #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
        pub struct Offer {
            pub item_condition: ItemCondition,
            pub price: Money,
            pub url: String,
        }
    }

    /* Corporation normalization is disabled; the organisation record is not needed.
    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    pub struct Corporation {
        pub id: String,
        pub name: String,
        pub legal_name: String,
        pub url: String,
        pub logo: String,
        pub email: String,
    }

    impl Corporation {
        fn from_raw(raw: raw::ldjsonitem::Corporation) -> Corporation {
            Corporation {
                id: raw.id,
                name: raw.name,
                legal_name: raw.legal_name,
                url: raw.url,
                logo: raw.logo,
                email: raw.email,
            }
        }
    }
    */

    /* FAQ normalization is disabled while questions and answers are not needed.
    #[derive(Debug, Clone, PartialEq, Serialize)]
    pub struct FaqPage {
        pub questions: Vec<FaqQuestion>,
    }

    impl FaqPage {
        fn from_raw(raw: raw::ldjsonitem::FaqPage) -> FaqPage {
            let questions = raw.main_entity.into_iter().map(FaqQuestion::from_raw).collect();

            FaqPage { questions }
        }
    }

    #[derive(Debug, Clone, PartialEq, Serialize)]
    pub struct FaqQuestion {
        pub question: String,
        pub answer: String,
    }

    impl FaqQuestion {
        fn from_raw(raw: raw::ldjsonitem::FaqQuestion) -> FaqQuestion {
            FaqQuestion {
                question: raw.name,
                answer: raw.accepted_answer.text,
            }
        }
    }
    */

    /* Breadcrumb normalization is disabled: home and product details are duplicated,
    and the intermediate collection is not relevant.
    #[derive(Debug, Clone, PartialEq, Serialize)]
    pub struct BreadcrumbList {
        pub crumbs: Vec<breadcrumb_list::Breadcrumb>,
    }

    impl BreadcrumbList {
        fn from_raw(raw: raw::ldjsonitem::BreadcrumbList) -> BreadcrumbList {
            let crumbs = raw
                .item_list_element
                .into_iter()
                .map(breadcrumb_list::Breadcrumb::from_raw)
                .collect();

            BreadcrumbList { crumbs }
        }
    }

    pub mod breadcrumb_list {
        #[derive(Debug, Clone, PartialEq, serde::Serialize)]
        pub struct Breadcrumb {
            pub position: i64,
            pub name: String,
            pub url: String,
        }

        impl Breadcrumb {
            fn from_raw(raw: super::raw::ldjsonitem::BreadcrumbItem) -> Breadcrumb {
                Breadcrumb {
                    position: raw.position,
                    name: raw.name,
                    url: raw.item,
                }
            }
        }
    }
    */
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProductRecord {
    pub id: String,
    pub handle: String,

    pub title: String,
    /// `None` where the record states no description.
    pub description: Option<String>,
    pub key_features: Vec<KeyFeature>,

    pub specifications: Vec<Specification>,

    pub product_type: Option<String>,
    pub published_at: DateTime<Utc>,

    pub price: Money,
    /// `None` when the storefront's zero sentinel indicates no markdown.
    pub compare_at_price: Option<Money>,
    pub available_for_sale: bool,

    pub options: Vec<ProductOption>,
}

impl ProductRecord {
    pub fn from_raw(
        raw: raw::nextdata::product::Product,
        page_metafields: &Value,
    ) -> Result<Self, NormalizeError> {
        let price = money(raw.price.value, &raw.price.currency_code, "price.value")?;

        let compare_at_price = (raw.list_price > 0.0)
            .then(|| money(raw.list_price, &raw.price.currency_code, "listPrice"))
            .transpose()?;

        let published_at = DateTime::parse_from_rfc3339(&raw.published_at)
            .map_err(|source| NormalizeError::PublishedAtNotAnInstant {
                value: raw.published_at.clone(),
                source,
            })?
            .with_timezone(&Utc);

        let key_features = key_features(&raw.metafields)?;

        // The displayed specifications belong to the page's metafields, not the product's.
        let components = page_metafields.get("components");
        let specifications = specifications(components)?;

        Ok(ProductRecord {
            id: raw.id,
            handle: raw.handle,

            title: raw.title,
            description: raw.description,
            key_features,
            specifications,

            product_type: raw.product_type,
            published_at,
            price,
            compare_at_price,
            available_for_sale: raw.available_for_sale,

            options: raw
                .options
                .into_iter()
                .map(ProductOption::from_raw)
                .collect(),
        })
    }
}

fn specifications(components: Option<&Value>) -> Result<Vec<Specification>, NormalizeError> {
    let Some(items) = components
        .and_then(|components| components.get("productSpecs"))
        .and_then(|specs| specs.get("items"))
    else {
        return Ok(vec![]);
    };

    let items = items.as_array().ok_or_else(|| NormalizeError::NotAList {
        path: "productSpecs.items".to_string(),
    })?;

    items
        .iter()
        .map(|item| {
            Ok(Specification {
                name: string_field(item, "name", "productSpecs")?,
                value: string_field(item, "presentation", "productSpecs")?,
            })
        })
        .collect()
}

fn string_field(entry: &Value, name: &str, block: &str) -> Result<String, NormalizeError> {
    entry
        .get(name)
        .and_then(Value::as_str)
        .map(str::to_string)
        .ok_or_else(|| NormalizeError::EntryFieldNotAString {
            block: block.to_string(),
            field: name.to_string(),
            entry: entry.clone(),
        })
}

fn key_features(metafields: &Value) -> Result<Vec<KeyFeature>, NormalizeError> {
    let Some(list) = metafields
        .get("productInfos")
        .and_then(|infos| infos.get("key_features"))
        .and_then(|features| features.get("list"))
    else {
        return Ok(vec![]);
    };

    let list = list.as_array().ok_or_else(|| NormalizeError::NotAList {
        path: "productInfos.key_features.list".to_string(),
    })?;

    list.iter()
        .map(|entry| {
            Ok(KeyFeature {
                title: string_field(entry, "key", "key_features")?,
                body: string_field(entry, "value", "key_features")?,
            })
        })
        .collect()
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct KeyFeature {
    pub title: String,
    pub body: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Specification {
    pub name: String,
    pub value: String,
}

/* Feature-block normalization is disabled: the panels are marketing rather than information.
   The measurements are kept because they are what settled it, and because the block is still
   there on the raw side for whoever revisits the decision.

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FeatureBlock {
    pub learn_more_url: String,
    pub panels: Vec<FeaturePanel>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FeaturePanel {
    pub image: String,
    pub image_mobile: String,
    pub caption: Option<String>,
}
*/

/* The `Download` type is disabled with the field above; we are not interested in that data at the
   moment. Kept whole rather than deleted, so this is one uncomment away.

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Download {
    pub title: String,
    pub url: String,
    pub kind: Option<String>,
    pub file_name: Option<String>,
    pub uploaded_at: Option<NaiveDate>,
    pub size: Option<i64>,
    pub description: Option<String>,
}
*/

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProductImage {
    pub url: String,
    pub alt_text: Option<String>,
    pub width: i64,
    pub height: i64,
}

impl ProductImage {
    fn from_raw(raw: raw::nextdata::product::Image) -> ProductImage {
        ProductImage {
            url: raw.url,
            alt_text: raw.alt_text,
            width: raw.width,
            height: raw.height,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProductOption {
    pub id: String,
    pub name: String,
    pub display_name: String,
    pub values: Vec<OptionValue>,
}

impl ProductOption {
    fn from_raw(raw: raw::nextdata::product::ProductOption) -> ProductOption {
        ProductOption {
            id: raw.id,
            name: raw.name,
            display_name: raw.display_name,
            values: raw.values.into_iter().map(OptionValue::from_raw).collect(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OptionValue {
    pub label: String,
    pub hex_colors: Option<Vec<String>>,
}

impl OptionValue {
    fn from_raw(raw: raw::nextdata::product::OptionValue) -> OptionValue {
        OptionValue {
            label: raw.label,
            hex_colors: raw.hex_colors,
        }
    }
}

/* Collections are not needed now; retained for possible later use.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Collection {
    pub id: String,
    pub handle: String,
    pub title: String,
}

impl Collection {
    fn from_raw(raw: raw::nextdata::product::CollectionNode) -> Collection {
        Collection {
            id: raw.id,
            handle: raw.handle,
            title: raw.title,
        }
    }
}
*/

/* SEO normalization is disabled; the description has a useful call to action for later use.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ProductSeo {
    pub title: Option<String>,
    pub description: Option<String>,
}

impl ProductSeo {
    fn from_raw(raw: raw::nextdata::product::ProductSeo) -> ProductSeo {
        ProductSeo {
            title: raw.title,
            description: raw.description,
        }
    }
}
*/

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Variant {
    pub id: String,
    pub sku: String,
    pub barcode: Option<String>,
    pub name: String,
    pub price: Money,
    pub compare_at_price: Option<Money>,
    pub quantity_available: i64,
    pub available_for_sale: bool,
    pub currently_not_in_stock: bool,
    pub requires_shipping: bool,
    /// Probably kilograms: the payload states no unit, but a 2-pack of 91 cm USB-C cables reads
    /// `0.0510` and the magnitudes hold across the subset. `None` where the variant renders the
    /// `0.0` an unweighed one does.
    pub weight: Option<f64>,
    pub image: ProductImage,
    pub options: Vec<ProductOption>,

    pub discounts: Vec<Discount>,
}

impl Variant {
    pub fn from_raw(raw: raw::nextdata::product::Variant) -> Result<Self, NormalizeError> {
        let price = money(raw.price, VARIANT_CURRENCY, "variants[].price")?;
        let compare_at_price = raw
            .list_price
            .map(|amount| money(amount, VARIANT_CURRENCY, "variants[].listPrice"))
            .transpose()?;

        let discounts = discounts(&raw.metafields)?;

        Ok(Variant {
            id: raw.id,
            sku: raw.sku,
            barcode: raw.barcode,
            name: raw.name,
            price,
            compare_at_price,
            quantity_available: raw.quantity_available,
            available_for_sale: raw.available_for_sale,
            currently_not_in_stock: raw.currently_not_in_stock,
            requires_shipping: raw.requires_shipping,
            weight: weight(raw.weight),
            image: ProductImage::from_raw(raw.image),
            options: raw
                .options
                .into_iter()
                .map(ProductOption::from_raw)
                .collect(),
            discounts,
        })
    }
}

// `0.0` is what an unweighed variant renders rather than a weightless one.
fn weight(raw: f64) -> Option<f64> {
    (raw != 0.0).then_some(raw)
}

fn discounts(metafields: &Value) -> Result<Vec<Discount>, NormalizeError> {
    let Some(entries) = metafields.get("discountCodeApp") else {
        return Ok(vec![]);
    };

    let entries = entries.as_array().ok_or_else(|| NormalizeError::NotAList {
        path: "discountCodeApp".to_string(),
    })?;

    entries.iter().map(discount).collect()
}

fn discount(entry: &Value) -> Result<Discount, NormalizeError> {
    let missing = |name: &str| NormalizeError::MissingDiscountField {
        field: name.to_string(),
        entry: entry.clone(),
    };

    let string = |name: &str| {
        entry
            .get(name)
            .and_then(Value::as_str)
            .map(str::to_string)
            .ok_or_else(|| missing(name))
    };

    let quoted_number = |name: &str| {
        let raw = entry
            .get(name)
            .and_then(Value::as_str)
            .ok_or_else(|| missing(name))?;

        raw.parse::<f64>()
            .map_err(|source| NormalizeError::NotANumber {
                field: name.to_string(),
                value: raw.to_string(),
                source,
            })
    };

    let instant = |name: &str| {
        let raw = string(name)?;

        DateTime::parse_from_rfc3339(&raw)
            .map_err(|source| NormalizeError::NotAnInstant {
                field: name.to_string(),
                value: raw,
                source,
            })
            .map(|instant| instant.with_timezone(&Utc))
    };

    let currency = string("currency")?;

    let value = match string("value_type")?.as_str() {
        "percentage" => DiscountValue::Percentage(quoted_number("value")?),
        "fixed_amount" => {
            DiscountValue::FixedAmount(money(quoted_number("value")?, &currency, "value")?)
        }
        other => {
            return Err(NormalizeError::UnknownDiscountValueType {
                value_type: other.to_string(),
                entry: entry.clone(),
            });
        }
    };

    let price_amount = entry
        .get("variant_price4wscode")
        .and_then(Value::as_f64)
        .ok_or_else(|| missing("variant_price4wscode"))?;

    let variant_shopify_id = entry
        .get("variant_shopify_id")
        .and_then(Value::as_i64)
        .ok_or_else(|| missing("variant_shopify_id"))?;

    let amazon_deal = entry
        .get("amazon_deal")
        .and_then(Value::as_bool)
        .ok_or_else(|| missing("amazon_deal"))?;

    let ends_at = match entry.get("ends_at") {
        Some(Value::Null) | None => None,
        Some(_) => Some(instant("ends_at")?),
    };

    let code = string("title")?;
    let kind = string("discount_type")?;
    let price = money(price_amount, &currency, "variant_price4wscode")?;
    let saving = money(quoted_number("fixed_value")?, &currency, "fixed_value")?;
    let starts_at = instant("starts_at")?;

    Ok(Discount {
        code,
        kind,
        value,
        price,
        saving,
        starts_at,
        ends_at,
        variant_shopify_id,
        amazon_deal,
    })
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Discount {
    pub code: String,
    pub kind: String,
    pub value: DiscountValue,
    pub price: Money,
    pub saving: Money,
    pub starts_at: DateTime<Utc>,
    pub ends_at: Option<DateTime<Utc>>,
    pub variant_shopify_id: i64,
    pub amazon_deal: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DiscountValue {
    /// Signed percentage points as supplied by the page: `-25.0` means 25% off.
    Percentage(f64),
    FixedAmount(Money),
}

#[cfg(test)]
mod tests {
    use super::*;
    use html_extractor::serde_json::{self, json};
    use ldjson::offer::ItemCondition;

    #[test]
    fn product_offer_omits_availability_and_preserves_condition_and_money() {
        for (availability, condition, expected_condition, serialized) in [
            (
                "https://schema.org/InStock",
                "https://schema.org/NewCondition",
                ItemCondition::New,
                json!("new"),
            ),
            (
                "https://schema.org/SoldOut",
                "https://schema.org/NewCondition",
                ItemCondition::New,
                json!("new"),
            ),
            (
                "https://example.com/stock",
                "https://example.com/condition",
                ItemCondition::Other("https://example.com/condition".into()),
                json!({"other": "https://example.com/condition"}),
            ),
        ] {
            let raw: raw::ldjsonitem::LdJson = serde_json::from_value(json!({
                "@type": "Product", "@context": "https://schema.org",
                "brand": {"@type": "Brand", "name": "Anker"},
                "description": "d", "image": ["i"], "name": "n", "sku": "A1",
                "offers": {
                    "@type": "Offer", "availability": availability,
                    "itemCondition": condition, "price": 99.99,
                    "priceCurrency": "AUD", "url": "u"
                }
            }))
            .unwrap();
            let raw::ldjsonitem::LdJson::Product(raw) = raw else {
                panic!("expected Product");
            };
            let product = ldjson::Product::from_raw(raw).expect("the island converts");
            assert_eq!(product.offer.item_condition, expected_condition);
            let output = serde_json::to_value(product).unwrap();
            assert!(output["offer"].get("availability").is_none());
            assert_eq!(output["offer"]["item_condition"], serialized);
            assert_eq!(
                output["offer"]["price"],
                json!({"amount": 9999, "currency": "AUD"})
            );
        }
    }

    fn islands(bodies: &[serde_json::Value]) -> Vec<raw::ldjsonitem::LdJsonItem> {
        bodies
            .iter()
            .map(|data| {
                serde_json::from_value(json!({
                    "type": "application/ld+json",
                    "data": data,
                }))
                .expect("the island deserializes")
            })
            .collect()
    }

    fn product_island() -> serde_json::Value {
        json!({
            "@type": "Product", "@context": "https://schema.org",
            "brand": {"@type": "Brand", "name": "Anker"},
            "description": "d", "image": ["i"], "name": "n", "sku": "A1",
            "offers": {
                "@type": "Offer", "availability": "https://schema.org/InStock",
                "itemCondition": "https://schema.org/NewCondition", "price": 99.99,
                "priceCurrency": "AUD", "url": "u"
            }
        })
    }

    fn corporation_island() -> serde_json::Value {
        json!({
            "@type": "Corporation", "@context": "https://schema.org",
            "@id": "i", "name": "Anker", "legalName": "Anker Innovations",
            "url": "u", "logo": "l", "email": "e", "sameAs": ["s"]
        })
    }

    #[test]
    fn the_product_island_is_picked_out_of_the_list() {
        let ld_json = ldjson::LdJson::from_raw(islands(&[corporation_island(), product_island()]))
            .expect("a list carrying a Product island converts");

        assert_eq!(ld_json.product.sku, "A1");
    }

    #[test]
    fn a_list_without_a_product_island_fails_the_document() {
        for bodies in [vec![], vec![corporation_island()]] {
            let error = ldjson::LdJson::from_raw(islands(&bodies))
                .expect_err("a list with no Product island cannot normalize");

            assert_eq!(error, NormalizeError::MissingProductIsland);
            assert_eq!(
                error.to_string(),
                "no `Product` ld+json island on the document"
            );
        }
    }

    #[test]
    fn the_first_product_island_wins() {
        let mut second = product_island();
        second["sku"] = json!("A2");

        let ld_json = ldjson::LdJson::from_raw(islands(&[product_island(), second]))
            .expect("two Product islands still convert");

        assert_eq!(ld_json.product.sku, "A1");
    }

    #[test]
    fn a_zero_compare_at_price_is_no_markdown_rather_than_a_price_of_nothing() {
        let compare_at_price = |spelling| {
            let mut raw = raw_product();
            raw["listPrice"] = spelling;

            record(raw).compare_at_price
        };

        assert_eq!(
            compare_at_price(json!(0)),
            None,
            "the integer spelling, 41 of 50"
        );
        assert_eq!(
            compare_at_price(json!(0.0)),
            None,
            "the float spelling, 9 of 50"
        );
        assert_eq!(
            compare_at_price(json!(279.99)),
            Some(Money::from_raw(279.99, "AUD").unwrap()),
            "a real markdown still crosses"
        );
    }

    #[test]
    fn a_published_at_that_is_not_an_instant_fails_the_document() {
        let mut raw = raw_product();
        raw["publishedAt"] = json!("1 January 2026");
        let raw: raw::nextdata::product::Product =
            serde_json::from_value(raw).expect("the record deserializes");
        let page_metafields = json!({});

        let error = ProductRecord::from_raw(raw, &page_metafields)
            .expect_err("a date the wire has never rendered cannot cross");

        assert_eq!(
            error,
            NormalizeError::PublishedAtNotAnInstant {
                value: "1 January 2026".to_string(),
                source: DateTime::parse_from_rfc3339("1 January 2026").unwrap_err(),
            }
        );
        assert!(
            error
                .to_string()
                .starts_with("publishedAt `1 January 2026` is not an RFC 3339 instant"),
            "{error}"
        );
    }

    #[test]
    fn a_variant_price_takes_the_currency_the_variant_does_not_render() {
        let mut raw = raw_variant();
        raw["price"] = json!(229.99);
        raw["listPrice"] = json!(279.99);
        let raw: raw::nextdata::product::Variant =
            serde_json::from_value(raw).expect("the variant deserializes");

        let variant = Variant::from_raw(raw).expect("the variant converts");

        assert_eq!(variant.price, Money::from_raw(229.99, "AUD").unwrap());
        assert_eq!(
            variant.compare_at_price,
            Some(Money::from_raw(279.99, "AUD").unwrap())
        );
        assert_eq!(
            serde_json::to_value(&variant.price).unwrap(),
            json!({"amount": 22999, "currency": "AUD"})
        );
    }

    #[test]
    fn the_document_reads_the_canonical_and_each_metafield_blob_for_its_own_block() {
        let raw: raw::AnkerAuRawDocument =
            serde_json::from_value(raw_document()).expect("the document deserializes");

        let document = AnkerAuNormalizedDocument::from_raw(raw).expect("the document normalizes");

        assert_eq!(
            document.canonical_url,
            "https://www.anker.com/au/products/h"
        );
        assert_eq!(
            document.product.specifications,
            vec![Specification {
                name: "Total Capacity".to_string(),
                value: "20,100mAh".to_string(),
            }],
            "the page's blob, not the decoy under the record's"
        );
        assert_eq!(
            document.product.key_features,
            vec![KeyFeature {
                title: "220W Total Output".to_string(),
                body: "Keep your work setup powered.".to_string(),
            }],
            "the record's own blob"
        );
        assert_eq!(document.variants.len(), 1);
    }

    fn record(raw: serde_json::Value) -> ProductRecord {
        let raw: raw::nextdata::product::Product =
            serde_json::from_value(raw).expect("the record deserializes");
        let page_metafields = json!({});

        ProductRecord::from_raw(raw, &page_metafields).expect("the record converts")
    }

    fn raw_product() -> serde_json::Value {
        json!({
            "id": "gid://shopify/Product/1", "handle": "h", "slug": "s", "path": "/p",
            "name": "n", "title": "t", "description": "d", "descriptionHtml": "<p>d</p>",
            "vendor": "Anker", "publishedAt": "2026-01-01T00:00:00Z",
            "price": {"value": 229.99, "currencyCode": "AUD"}, "listPrice": 0,
            "availableForSale": true, "totalInventory": 13,
            "images": [{"url": "u", "width": 800, "height": 800}], "tags": ["t"],
            "options": [], "collections": {"nodes": []}, "seo": {},
            "requiresSellingPlan": false, "sellingPlanGroups": [],
            "metafields": {}, "metafieldsMap": {}, "variants": [raw_variant()]
        })
    }

    fn raw_variant() -> serde_json::Value {
        json!({
            "id": "gid://shopify/ProductVariant/1", "sku": "A110BH11",
            "barcode": "194644239350", "name": "v", "price": 229.99, "listPrice": null,
            "quantityAvailable": 0, "availableForSale": false, "currentlyNotInStock": false,
            "requiresShipping": true, "weight": 2.0216,
            "image": {"url": "u", "width": 800, "height": 800},
            "options": [], "sellingPlanAllocations": [],
            "metafields": {}, "metafieldsMap": {}
        })
    }

    fn raw_document() -> serde_json::Value {
        let mut product = raw_product();
        product["metafields"] = json!({
            "productInfos": {"key_features": {"list": [
                {"key": "220W Total Output", "value": "Keep your work setup powered."}
            ]}},
            "components": {"productSpecs": {"items": [
                {"name": "Decoy", "presentation": "read from the page instead"}
            ]}}
        });

        json!({
            "ld-json": [{"type": "application/ld+json", "data": product_island()}],
            "next_data": {"id": "__NEXT_DATA__", "type": "application/json", "data": {
                "buildId": "b", "page": "/products/[slug]", "locale": "au",
                "defaultLocale": "au", "locales": ["au"], "gsp": true, "isFallback": false,
                "isExperimentalCompile": false, "dynamicIds": [1], "scriptLoader": [],
                "query": {"slug": "s"},
                "props": {"__N_SSG": true, "pageProps": {
                    "slug": "s", "pageType": "charger", "type": "product",
                    "relatedProducts": [], "brands": [], "navCategories": [],
                    "seo": {
                        "title": "t", "description": "d",
                        "canonical": "https://www.anker.com/au/products/h",
                        "languageAlternates": [],
                        "openGraph": {"title": "t", "description": "d", "type": "website",
                                      "site_name": "Anker", "images": []}
                    },
                    "buildProps": {}, "shop": {},
                    "metafields": {"components": {"productSpecs": {"items": [
                        {"name": "Total Capacity", "presentation": "20,100mAh"}
                    ]}}},
                    "product": product
                }}
            }}
        })
    }

    #[test]
    fn a_product_with_no_component_blocks_gets_empty_lists() {
        for components in [None, Some(&json!({})), Some(&json!({"faq": {}}))] {
            assert_eq!(super::specifications(components).unwrap(), vec![]);
        }
    }

    #[test]
    fn specifications_cross_over_renamed() {
        let components = json!({"productSpecs": {
            "title": "<strong>Specifications</strong>",
            "items": [{"name": "Total Capacity", "presentation": "20,100mAh"}]
        }});

        let specs = super::specifications(Some(&components)).expect("the block converts");

        assert_eq!(specs[0].name, "Total Capacity");
        assert_eq!(specs[0].value, "20,100mAh");
    }

    #[test]
    fn the_specs_styling_stub_is_not_read_as_a_specification_block() {
        let components = json!({"specs": {"styles": {"paddingBottom": true}}});

        assert_eq!(super::specifications(Some(&components)).unwrap(), vec![]);
    }

    #[test]
    fn a_misshapen_component_block_fails_the_document() {
        for (components, expected, message) in [
            (
                json!({"productSpecs": {"items": "nope"}}),
                NormalizeError::NotAList {
                    path: "productSpecs.items".to_string(),
                },
                "`productSpecs.items` is not a list",
            ),
            (
                json!({"productSpecs": {"items": [{"name": "n"}]}}),
                NormalizeError::EntryFieldNotAString {
                    block: "productSpecs".to_string(),
                    field: "presentation".to_string(),
                    entry: json!({"name": "n"}),
                },
                r#"a `productSpecs` entry has no string `presentation`: {"name":"n"}"#,
            ),
        ] {
            let error = super::specifications(Some(&components))
                .expect_err("a shape never rendered is a finding");

            assert_eq!(error, expected);
            assert_eq!(error.to_string(), message);
        }
    }

    #[test]
    fn an_absent_and_an_empty_discount_list_both_read_as_none() {
        for metafields in [
            json!({}),
            json!({"discountCodeApp": []}),
            json!({"infos": {}}),
        ] {
            assert_eq!(super::discounts(&metafields).unwrap(), vec![]);
        }
    }

    #[test]
    fn a_percentage_discount_carries_its_rate_price_and_saving() {
        let discounts =
            super::discounts(&json!({"discountCodeApp": [entry()]})).expect("the entry converts");

        let [discount] = discounts.as_slice() else {
            panic!("one entry in, one out");
        };

        assert_eq!(discount.code, "WSTDAS25PCTOFF");
        assert_eq!(discount.kind, "WSTD");
        assert_eq!(
            discount.value,
            DiscountValue::Percentage(-25.0),
            "the sign is the wire's and is kept"
        );
        assert_eq!(discount.price, Money::from_raw(172.5, "AUD").unwrap());
        assert_eq!(discount.saving, Money::from_raw(57.49, "AUD").unwrap());
        assert!(discount.ends_at.is_none());
        assert_eq!(discount.variant_shopify_id, 56758478831919);
        assert!(discount.amazon_deal);

        let serialized = serde_json::to_value(discount).unwrap();
        for dropped in ["cdi_id", "cdi_updated_at", "value_style"] {
            assert!(serialized.get(dropped).is_none(), "{dropped} crossed");
        }
    }

    #[test]
    fn a_fixed_amount_discount_carries_money_rather_than_a_rate() {
        let mut raw = entry();
        raw["value_type"] = json!("fixed_amount");
        raw["value"] = json!("-80.0");

        let discounts = super::discounts(&json!({"discountCodeApp": [raw]})).unwrap();

        assert_eq!(
            discounts[0].value,
            DiscountValue::FixedAmount(Money::from_raw(-80.0, "AUD").unwrap())
        );
    }

    #[test]
    fn an_integer_discount_price_and_a_float_one_both_read() {
        for (spelling, expected) in [(json!(172.5), 17250), (json!(88), 8800)] {
            let mut raw = entry();
            raw["variant_price4wscode"] = spelling.clone();

            let discounts = super::discounts(&json!({"discountCodeApp": [raw]})).unwrap();

            assert_eq!(discounts[0].price.amount(), expected, "{spelling}");
        }
    }

    #[test]
    fn the_stale_entry_crosses_carrying_the_evidence_that_it_is_stale() {
        let mut raw = entry();
        raw["variant_shopify_id"] = json!(45066711335215_i64);
        raw["ends_at"] = json!("2025-06-30T08:00:59.000Z");

        let discounts = super::discounts(&json!({"discountCodeApp": [raw]})).unwrap();

        assert_eq!(discounts[0].variant_shopify_id, 45066711335215);
        assert_eq!(
            discounts[0].ends_at.unwrap().to_rfc3339(),
            "2025-06-30T08:00:59+00:00"
        );
    }

    #[test]
    fn a_misshapen_entry_fails_the_document() {
        type Case = (
            serde_json::Value,
            fn(&serde_json::Value) -> NormalizeError,
            &'static str,
        );

        let cases: [Case; 4] = [
            (
                json!({"value_type": "buy_one_get_one"}),
                |entry| NormalizeError::UnknownDiscountValueType {
                    value_type: "buy_one_get_one".to_string(),
                    entry: entry.clone(),
                },
                "unknown `value_type`",
            ),
            (
                json!({"value": "twenty percent"}),
                |_| NormalizeError::NotANumber {
                    field: "value".to_string(),
                    value: "twenty percent".to_string(),
                    source: "twenty percent".parse::<f64>().unwrap_err(),
                },
                "is not a number",
            ),
            (
                json!({"starts_at": "yesterday"}),
                |_| NormalizeError::NotAnInstant {
                    field: "starts_at".to_string(),
                    value: "yesterday".to_string(),
                    source: DateTime::parse_from_rfc3339("yesterday").unwrap_err(),
                },
                "not an RFC 3339 instant",
            ),
            (
                json!({"title": null}),
                |entry| NormalizeError::MissingDiscountField {
                    field: "title".to_string(),
                    entry: entry.clone(),
                },
                "no `title`",
            ),
        ];

        for (mutate, expected, message) in cases {
            let mut raw = entry();
            for (key, value) in mutate.as_object().unwrap() {
                raw[key] = value.clone();
            }

            let error = super::discounts(&json!({"discountCodeApp": [raw.clone()]}))
                .expect_err("a shape never rendered is a finding");

            assert_eq!(error, expected(&raw));
            assert!(
                error.to_string().contains(message),
                "{message} not in: {error}"
            );
        }

        assert_eq!(
            super::discounts(&json!({"discountCodeApp": {}}))
                .expect_err("an object where the list belongs is a finding"),
            NormalizeError::NotAList {
                path: "discountCodeApp".to_string(),
            }
        );
    }

    #[test]
    fn a_price_that_is_not_money_names_the_wire_key() {
        let error = NormalizeError::NotMoney {
            field: "variant_price4wscode".to_string(),
            source: money::MoneyError::InvalidArgument,
        };

        assert_eq!(
            error.to_string(),
            "`variant_price4wscode` is not money: InvalidArgument"
        );
    }

    fn entry() -> serde_json::Value {
        json!({
            "title": "WSTDAS25PCTOFF", "starts_at": "2026-08-21T03:48:39.000Z",
            "variant_shopify_id": 56758478831919_i64, "value_type": "percentage",
            "value": "-25.0", "currency": "AUD", "fixed_value": "57.49",
            "value_style": "$25", "variant_price4wscode": 172.5, "discount_type": "WSTD",
            "amazon_deal": true, "cdi_id": 174004,
            "cdi_updated_at": "2026-08-25T01:40:48.373Z"
        })
    }

    #[test]
    fn key_features_cross_over_renamed() {
        let metafields = json!({
            "productInfos": {"key_features": {"list": [
                {"key": "220W Total Output", "value": "Keep your work setup powered."}
            ]}}
        });

        let features = super::key_features(&metafields).expect("the list converts");

        assert_eq!(features.len(), 1);
        assert_eq!(features[0].title, "220W Total Output");
        assert_eq!(features[0].body, "Keep your work setup powered.");
    }

    #[test]
    fn a_product_without_bullets_gets_an_empty_list() {
        for metafields in [
            json!({}),
            json!({"productInfos": null}),
            json!({"productInfos": {"subtitle": "s"}}),
            json!({"productInfos": {"key_features": {}}}),
        ] {
            assert_eq!(super::key_features(&metafields).unwrap(), vec![]);
        }
    }

    #[test]
    fn an_entry_that_is_not_a_pair_of_strings_fails() {
        let metafields = json!({
            "productInfos": {"key_features": {"list": [{"key": "Headline"}]}}
        });

        let error = super::key_features(&metafields).expect_err("a bodyless entry fails");

        assert_eq!(
            error,
            NormalizeError::EntryFieldNotAString {
                block: "key_features".to_string(),
                field: "value".to_string(),
                entry: json!({"key": "Headline"}),
            }
        );
        assert_eq!(
            error.to_string(),
            r#"a `key_features` entry has no string `value`: {"key":"Headline"}"#
        );
    }
}
