//! Expresses the storefront's product facts in the system's vocabulary. The schema.org island and
//! the `__NEXT_DATA__` record both cross, because reconciling them is structuralization's job;
//! what nothing needs stays behind as the commented-out exclusion records below.

pub mod ldjson;
pub mod metafields;

use super::raw;
use chrono::{DateTime, Utc};
use html_extractor::serde_json::Value;
use metafields::{Discount, KeyFeature, KeyParameter, MemberPrice, Specification};
use money::{Money, MoneyError};
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

    #[error("`{field}` is not money: {source:?}")]
    NotMoney { field: String, source: MoneyError },

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

    #[error("a `memberPrice` entry has no `{field}`: {entry}")]
    MissingMemberPriceField { field: String, entry: Value },

    #[error("unknown `memberPrice` `price_type` `{price_type}`: {entry}")]
    UnknownMemberPriceType { price_type: String, entry: Value },

    #[error("a `memberPrice` entry carries only one of `price_type` and `price_value`: {entry}")]
    UnpairedMemberPrice { entry: Value },

    #[error("a `key_parameter` row carries neither or both of `value` and `valueList`: {row}")]
    KeyParameterWithoutValue { row: Value },
}

fn money(amount: f64, currency: &str, field: &str) -> Result<Money, NormalizeError> {
    Money::from_raw(amount, currency).map_err(|source| NormalizeError::NotMoney {
        field: field.to_string(),
        source,
    })
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AnkerComNormalizedDocument {
    // Commented out: `us` on every page; the retailer code already names the storefront.
    // pub locale: String,
    pub ld_json: ldjson::LdJson,
    pub canonical_url: String,
    // Commented out: page metadata beyond `canonical`, lifted above. `language_alternates` names
    // the same product on the other Anker storefronts, should a cross-storefront join need it.
    // pub seo: PageSeo,
    pub product: ProductRecord,
}

impl AnkerComNormalizedDocument {
    pub fn from_raw(
        raw: raw::AnkerComRawDocument,
    ) -> Result<AnkerComNormalizedDocument, NormalizeError> {
        let next_data = raw.next_data.data;
        let page_props = next_data.props.page_props;

        Ok(AnkerComNormalizedDocument {
            // locale: next_data.locale,
            ld_json: ldjson::LdJson::from_raw(raw.ld_json)?,
            canonical_url: page_props.seo.canonical,
            // seo: PageSeo::from_raw(page_props.seo),
            product: ProductRecord::from_raw(page_props.product, &page_props.metafields)?,
        })
    }
}

/* Page SEO normalization is disabled beyond `canonical`: nothing downstream reads the rest.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct PageSeo {
    pub title: String,
    pub description: Option<String>,

    pub canonical: String,
    pub language_alternates: Vec<LanguageAlternate>,
    pub open_graph: OpenGraph,
}

impl PageSeo {
    fn from_raw(raw: raw::nextdata::Seo) -> PageSeo {
        PageSeo {
            title: raw.title,
            description: raw.description,
            canonical: raw.canonical,
            language_alternates: raw
                .language_alternates
                .into_iter()
                .map(LanguageAlternate::from_raw)
                .collect(),
            open_graph: OpenGraph::from_raw(raw.open_graph),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct LanguageAlternate {
    pub href: String,
    pub href_lang: String,
}

impl LanguageAlternate {
    fn from_raw(raw: raw::nextdata::LanguageAlternate) -> LanguageAlternate {
        LanguageAlternate {
            href: raw.href,
            href_lang: raw.href_lang,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct OpenGraph {
    pub title: String,
    pub description: Option<String>,
    #[serde(rename = "type")]
    pub kind: String,
    pub site_name: String,
    pub images: Vec<OpenGraphImage>,
}

impl OpenGraph {
    fn from_raw(raw: raw::nextdata::OpenGraph) -> OpenGraph {
        OpenGraph {
            title: raw.title,
            description: raw.description,
            kind: raw.type_,
            site_name: raw.site_name,
            images: raw
                .images
                .into_iter()
                .map(OpenGraphImage::from_raw)
                .collect(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct OpenGraphImage {
    pub url: String,
    pub alt: String,
    pub width: String,
    pub height: String,
}

impl OpenGraphImage {
    fn from_raw(raw: raw::nextdata::OpenGraphImage) -> OpenGraphImage {
        OpenGraphImage {
            url: raw.url,
            alt: raw.alt,
            width: raw.width,
            height: raw.height,
        }
    }
}
*/

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProductRecord {
    pub id: String,
    pub handle: String,
    // Commented out: equal to `handle` on every page.
    // pub slug: String,
    // Commented out: `/` followed by `handle` on every page.
    // pub path: String,
    // Commented out: equal to `title` on every page.
    // pub name: String,
    pub title: String,
    pub description: Option<String>,
    // Commented out: the same copy with markup; adds no value for now.
    // pub description_html: Option<String>,
    // Commented out: storefront bookkeeping (`beta-anker-us` or `Anker`), not a sub-brand.
    // pub vendor: String,
    pub key_features: Vec<KeyFeature>,
    pub specifications: Vec<Specification>,
    pub product_type: Option<String>,
    pub published_at: DateTime<Utc>,
    pub price: Money,
    /// `None` where the product is not marked down.
    pub compare_at_price: Option<Money>,
    pub available_for_sale: bool,
    // Commented out: the sum of the variants' `quantity_available` on every page.
    // pub total_inventory: i64,
    // Commented out: the island's `images`, the same URLs in the same order on every page.
    // pub images: Vec<ProductImage>,
    // Commented out: merchandising labels. `Refurbished` among them is the page's only statement
    // of refurbishment - the island's condition says new on those products too.
    // pub tags: Vec<String>,
    pub options: Vec<ProductOption>,
    // Commented out: merchandising groupings, not facts about the product.
    // pub collections: Vec<Collection>,
    // Commented out: storefront metadata nothing downstream reads.
    // pub seo: ProductSeo,
    // Commented out: set on a minority of pages, and on another host (`us.anker.com`).
    // pub online_store_url: Option<String>,
    // Commented out: false on every page.
    // pub requires_selling_plan: bool,
    pub variants: Vec<Variant>,
}

impl ProductRecord {
    fn from_raw(
        raw: raw::nextdata::product::Product,
        page_metafields: &Value,
    ) -> Result<ProductRecord, NormalizeError> {
        let currency = raw.price.currency_code.as_str();

        Ok(ProductRecord {
            id: raw.id,
            handle: raw.handle,
            // slug: raw.slug,
            // path: raw.path,
            // name: raw.name,
            title: raw.title,
            description: raw.description,
            // description_html: raw.description_html,
            // vendor: raw.vendor,
            key_features: metafields::key_features(&raw.metafields)?,
            specifications: metafields::specifications(page_metafields)?,
            product_type: raw.product_type,
            published_at: published_at(&raw.published_at)?,
            price: money(raw.price.value, currency, "price.value")?,
            compare_at_price: compare_at_price(raw.list_price, currency)?,
            available_for_sale: raw.available_for_sale,
            // total_inventory: raw.total_inventory,
            // images: raw.images.into_iter().map(ProductImage::from_raw).collect(),
            // tags: raw.tags,
            options: raw
                .options
                .into_iter()
                .map(ProductOption::from_raw)
                .collect(),
            // collections: raw.collections.nodes.into_iter().map(Collection::from_raw).collect(),
            // seo: ProductSeo::from_raw(raw.seo),
            // online_store_url: raw.online_store_url,
            // requires_selling_plan: raw.requires_selling_plan,
            variants: variants(raw.variants, currency)?,
        })
    }
}

fn published_at(raw: &str) -> Result<DateTime<Utc>, NormalizeError> {
    let parsed = DateTime::parse_from_rfc3339(raw).map_err(|source| {
        NormalizeError::PublishedAtNotAnInstant {
            value: raw.to_string(),
            source,
        }
    })?;

    Ok(parsed.with_timezone(&Utc))
}

// `listPrice` renders `0` rather than being absent when the product is not marked down.
fn compare_at_price(list_price: f64, currency: &str) -> Result<Option<Money>, NormalizeError> {
    (list_price != 0.0)
        .then(|| money(list_price, currency, "listPrice"))
        .transpose()
}

// Variant prices carry no currency of their own; the record's code, from the same payload, is the
// nearest statement of one.
fn variants(
    raw: Vec<raw::nextdata::product::Variant>,
    currency: &str,
) -> Result<Vec<Variant>, NormalizeError> {
    raw.into_iter()
        .map(|variant| Variant::from_raw(variant, currency))
        .collect()
}

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

/* Product SEO normalization is disabled; nothing downstream reads it.
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
    /// `None` where the storefront renders its not-for-sale placeholder instead of a price.
    pub price: Option<Money>,
    pub compare_at_price: Option<Money>,
    pub quantity_available: i64,
    pub available_for_sale: bool,
    pub currently_not_in_stock: bool,
    pub requires_shipping: bool,
    /// Unit unknown: the payload states none, and SKUs shared with `ankerau` fit neither kilograms
    /// nor pounds consistently. `None` where the variant is unweighed.
    pub weight: Option<f64>,
    pub image: ProductImage,
    pub options: Vec<VariantOption>,
    pub discounts: Vec<Discount>,
    pub member_price: Option<MemberPrice>,
    pub key_parameters: Vec<KeyParameter>,
}

impl Variant {
    fn from_raw(
        raw: raw::nextdata::product::Variant,
        currency: &str,
    ) -> Result<Variant, NormalizeError> {
        Ok(Variant {
            id: raw.id,
            sku: raw.sku,
            barcode: raw.barcode,
            name: raw.name,
            price: variant_price(raw.price, currency)?,
            compare_at_price: raw
                .list_price
                .map(|amount| money(amount, currency, "variants[].listPrice"))
                .transpose()?,
            quantity_available: raw.quantity_available,
            available_for_sale: raw.available_for_sale,
            currently_not_in_stock: raw.currently_not_in_stock,
            requires_shipping: raw.requires_shipping,
            weight: weight(raw.weight),
            image: ProductImage::from_raw(raw.image),
            options: raw
                .options
                .into_iter()
                .map(VariantOption::from_raw)
                .collect(),
            discounts: metafields::discounts(&raw.metafields)?,
            member_price: metafields::member_price(&raw.metafields, currency)?,
            key_parameters: metafields::key_parameters(&raw.metafields)?,
        })
    }
}

const NOT_FOR_SALE_PLACEHOLDER_MINOR_UNITS: i64 = 999_999_999;

// `9999999.99` is what the storefront renders on a variant it does not sell, not a price anyone
// is asked to pay. A real price on an unavailable variant still crosses.
fn variant_price(amount: f64, currency: &str) -> Result<Option<Money>, NormalizeError> {
    let price = money(amount, currency, "variants[].price")?;

    Ok((price.amount() != NOT_FOR_SALE_PLACEHOLDER_MINOR_UNITS).then_some(price))
}

// `0.0` is what an unweighed variant renders rather than a weightless one.
fn weight(raw: f64) -> Option<f64> {
    (raw != 0.0).then_some(raw)
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VariantOption {
    // Commented out: holds the variant's own gid rather than an option id, on every option.
    // pub id: String,
    pub name: String,
    pub display_name: String,
    pub values: Vec<OptionValue>,
}

impl VariantOption {
    fn from_raw(raw: raw::nextdata::product::ProductOption) -> VariantOption {
        VariantOption {
            // id: raw.id,
            name: raw.name,
            display_name: raw.display_name,
            values: raw.values.into_iter().map(OptionValue::from_raw).collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use html_extractor::serde_json::{self, json};

    fn usd(amount: i64) -> Money {
        Money::from_raw(amount as f64 / 100.0, "USD").unwrap()
    }

    fn raw_variant(price: f64) -> Value {
        json!({
            "id": "gid://shopify/ProductVariant/1", "sku": "A1637112", "barcode": null,
            "name": "Black", "price": price, "listPrice": null, "quantityAvailable": 0,
            "availableForSale": false, "currentlyNotInStock": false, "requiresShipping": true,
            "weight": 0.0,
            "image": {"url": "u", "altText": null, "width": 1, "height": 1},
            "options": [{
                "__typename": "MultipleChoiceOption", "id": "gid://shopify/ProductVariant/1",
                "name": "Color", "displayName": "color",
                "values": [{"label": "Black", "hexColors": ["#000000"]}]
            }],
            "sellingPlanAllocations": [], "metafields": {}, "metafieldsMap": {}
        })
    }

    fn raw_product(list_price: f64, variants: Vec<Value>) -> Value {
        json!({
            "id": "gid://shopify/Product/1", "handle": "a1637", "slug": "a1637",
            "path": "/a1637", "name": "n", "title": "Anker <b>633</b> Magnetic Battery",
            "description": "d", "descriptionHtml": "<p>d</p>", "vendor": "beta-anker-us",
            "productType": null, "publishedAt": "2025-09-04T06:34:16Z",
            "price": {"value": 139.98, "currencyCode": "USD"}, "listPrice": list_price,
            "availableForSale": true, "totalInventory": 0, "images": [], "tags": ["Refurbished"],
            "options": [], "collections": {"nodes": []}, "seo": {}, "onlineStoreUrl": null,
            "requiresSellingPlan": false, "sellingPlanGroups": [], "variants": variants,
            "metafields": {}, "metafieldsMap": {}
        })
    }

    fn record(list_price: f64, variants: Vec<Value>) -> Result<ProductRecord, NormalizeError> {
        let raw = serde_json::from_value(raw_product(list_price, variants)).unwrap();

        ProductRecord::from_raw(raw, &json!({}))
    }

    #[test]
    fn a_zero_list_price_is_no_markdown_and_a_real_one_crosses() {
        assert_eq!(record(0.0, vec![]).unwrap().compare_at_price, None);
        assert_eq!(
            record(159.99, vec![]).unwrap().compare_at_price,
            Some(usd(15999))
        );
    }

    #[test]
    fn the_record_price_lands_on_its_cent() {
        assert_eq!(record(0.0, vec![]).unwrap().price, usd(13998));
    }

    #[test]
    fn the_not_for_sale_placeholder_is_no_price_but_a_real_price_on_an_unsold_variant_stays() {
        let record = record(0.0, vec![raw_variant(9999999.99), raw_variant(44.99)]).unwrap();

        assert_eq!(record.variants[0].price, None);
        assert_eq!(record.variants[1].price, Some(usd(4499)));
    }

    #[test]
    fn variant_prices_take_the_records_currency() {
        let mut raw = raw_product(0.0, vec![raw_variant(44.99)]);
        raw["price"]["currencyCode"] = json!("CAD");
        let raw = serde_json::from_value(raw).unwrap();

        let record = ProductRecord::from_raw(raw, &json!({})).unwrap();

        assert_eq!(
            record.variants[0].price,
            Some(Money::from_raw(44.99, "CAD").unwrap())
        );
    }

    #[test]
    fn an_unweighed_variant_has_no_weight() {
        let record = record(0.0, vec![raw_variant(44.99)]).unwrap();

        assert_eq!(record.variants[0].weight, None);
    }

    #[test]
    fn a_published_at_that_is_not_an_instant_fails_with_its_value() {
        let mut raw = raw_product(0.0, vec![]);
        raw["publishedAt"] = json!("2025-09-04");
        let raw = serde_json::from_value(raw).unwrap();

        let error = ProductRecord::from_raw(raw, &json!({})).unwrap_err();

        assert!(matches!(
            error,
            NormalizeError::PublishedAtNotAnInstant { value, .. } if value == "2025-09-04"
        ));
    }

    #[test]
    fn the_title_crosses_with_its_markup() {
        assert_eq!(
            record(0.0, vec![]).unwrap().title,
            "Anker <b>633</b> Magnetic Battery"
        );
    }

    #[test]
    fn a_variant_option_does_not_carry_the_variant_gid_as_its_id() {
        let record = record(0.0, vec![raw_variant(44.99)]).unwrap();

        let option = serde_json::to_value(&record.variants[0].options[0]).unwrap();

        assert_eq!(
            option,
            json!({
                "name": "Color", "display_name": "color",
                "values": [{"label": "Black", "hex_colors": ["#000000"]}]
            })
        );
    }

    #[test]
    fn the_record_survives_the_round_trip_the_next_stage_makes() {
        let record = record(159.99, vec![raw_variant(9999999.99), raw_variant(44.99)]).unwrap();

        let json = serde_json::to_string(&record).unwrap();

        assert_eq!(
            serde_json::from_str::<ProductRecord>(&json).unwrap(),
            record
        );
    }
}
