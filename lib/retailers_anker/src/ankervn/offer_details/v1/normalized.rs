//! Expresses the storefront's product facts in the system's vocabulary. The schema.org island and
//! the `__NEXT_DATA__` record both cross, because reconciling them is structuralization's job;
//! what nothing needs stays behind as the commented-out exclusion records below.

pub mod ldjson;
pub mod metafields;

use super::raw;
use chrono::{DateTime, Utc};
use html_extractor::serde_json::Value;
use metafields::Specification;
use money::{Money, MoneyError};
use serde::{Deserialize, Serialize};
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
}

fn money(amount: f64, currency: &str, field: &str) -> Result<Money, NormalizeError> {
    Money::from_raw(amount, currency).map_err(|source| NormalizeError::NotMoney {
        field: field.to_string(),
        source,
    })
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AnkerVnNormalizedDocument {
    // Commented out: `vn` on every page; the retailer code already names the storefront.
    // pub locale: String,
    pub ld_json: ldjson::LdJson,
    pub canonical_url: String,
    // Commented out: page metadata beyond `canonical`, lifted above. `language_alternates` names
    // the same product on the other Anker storefronts, should a cross-storefront join need it.
    // pub seo: PageSeo,
    pub product: ProductRecord,
    // Commented out: the EU's guarantee notices per variant SKU, an empty list on every variant.
    // pub legal_guarantee: BTreeMap<String, Vec<LegalGuarantee>>,
}

impl AnkerVnNormalizedDocument {
    pub fn from_raw(
        raw: raw::AnkerVnRawDocument,
    ) -> Result<AnkerVnNormalizedDocument, NormalizeError> {
        let next_data = raw.next_data.data;
        let page_props = next_data.props.page_props;

        Ok(AnkerVnNormalizedDocument {
            // locale: next_data.locale,
            ld_json: ldjson::LdJson::from_raw(raw.ld_json)?,
            canonical_url: page_props.seo.canonical,
            // seo: PageSeo::from_raw(page_props.seo),
            product: ProductRecord::from_raw(page_props.product, &page_props.metafields)?,
            // legal_guarantee: page_props.legal_guarantee,
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
    pub description: String,
    // Commented out: the same copy with markup; adds no value for now.
    // pub description_html: String,
    // Commented out: storefront bookkeeping (`beta-anker-us`, `beta-ankerwork-us` or `Anker`),
    // not a sub-brand.
    // pub vendor: String,
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
    // Commented out: merchandising labels; none states a fact the other fields do not.
    // pub tags: Vec<String>,
    pub options: Vec<ProductOption>,
    // Commented out: merchandising groupings, not facts about the product.
    // pub collections: Vec<Collection>,
    // Commented out: storefront metadata nothing downstream reads.
    // pub seo: ProductSeo,
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
}

impl OptionValue {
    fn from_raw(raw: raw::nextdata::product::OptionValue) -> OptionValue {
        OptionValue { label: raw.label }
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
    /// `None` where the storefront renders no barcode for the variant.
    pub barcode: Option<String>,
    pub name: String,
    pub price: Money,
    pub compare_at_price: Option<Money>,
    pub quantity_available: i64,
    pub available_for_sale: bool,
    pub currently_not_in_stock: bool,
    pub requires_shipping: bool,
    /// Unit unknown: the payload states none, and SKUs shared with the other Anker storefronts fit
    /// neither kilograms nor pounds consistently. `None` where the variant is unweighed.
    pub weight: Option<f64>,
    pub image: ProductImage,
    pub options: Vec<VariantOption>,
    // Commented out: empty on every variant.
    // pub selling_plan_allocations: Vec<Value>,
    // Commented out: an Amazon listing link, a pre-sale flag and a trade-in offer on a few
    // variants; none states this variant's price.
    // pub metafields: Value,
    // Commented out: the same metafields flattened to strings.
    // pub metafields_map: MetafieldsMap,
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
            price: money(raw.price, currency, "variants[].price")?,
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
            // selling_plan_allocations: raw.selling_plan_allocations,
            // metafields: raw.metafields,
            // metafields_map: raw.metafields_map,
        })
    }
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

    fn vnd(amount: i64) -> Money {
        Money::from_raw(amount as f64, "VND").unwrap()
    }

    fn raw_variant(price: f64, weight: f64) -> Value {
        json!({
            "id": "gid://shopify/ProductVariant/1", "sku": "A2331111", "barcode": "194644098117",
            "name": "Đá đen", "price": price, "listPrice": null, "quantityAvailable": 10,
            "availableForSale": true, "currentlyNotInStock": false, "requiresShipping": true,
            "weight": weight,
            "image": {"url": "u", "altText": null, "width": 1, "height": 1},
            "options": [{
                "__typename": "MultipleChoiceOption", "id": "gid://shopify/ProductVariant/1",
                "name": "Màu sắc", "displayName": "màu sắc", "values": [{"label": "Đá đen"}]
            }],
            "sellingPlanAllocations": [], "metafields": {}, "metafieldsMap": {}
        })
    }

    fn raw_product(list_price: f64, variants: Vec<Value>) -> Value {
        json!({
            "id": "gid://shopify/Product/1", "handle": "a2331", "slug": "a2331",
            "path": "/a2331", "name": "n", "title": "Sạc Anker <b>323</b> (33W)",
            "description": "d", "descriptionHtml": "<p>d</p>", "vendor": "beta-anker-us",
            "productType": null, "publishedAt": "2025-09-04T06:34:16Z",
            "price": {"value": 380000.0, "currencyCode": "VND"}, "listPrice": list_price,
            "availableForSale": true, "totalInventory": 10, "images": [], "tags": [],
            "options": [], "collections": {"nodes": []}, "seo": {},
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
            record(450000.0, vec![]).unwrap().compare_at_price,
            Some(vnd(450000))
        );
    }

    #[test]
    fn a_dong_price_is_stored_whole() {
        let record = record(0.0, vec![raw_variant(1014000.0, 0.2)]).unwrap();

        assert_eq!(record.price.amount(), 380_000);
        assert_eq!(record.variants[0].price.amount(), 1_014_000);
    }

    #[test]
    fn an_implausibly_low_price_crosses_as_the_page_states_it() {
        let record = record(
            0.0,
            vec![raw_variant(20.0, 0.2), raw_variant(380000.0, 0.2)],
        )
        .unwrap();

        assert_eq!(record.variants[0].price, vnd(20));
        assert_eq!(record.variants[1].price, vnd(380000));
    }

    #[test]
    fn variant_prices_take_the_records_currency() {
        let record = record(0.0, vec![raw_variant(380000.0, 0.2)]).unwrap();

        assert_eq!(record.variants[0].price.currency(), money::Currency::VND);
    }

    #[test]
    fn an_unweighed_variant_has_no_weight() {
        let record = record(0.0, vec![raw_variant(380000.0, 0.0)]).unwrap();

        assert_eq!(record.variants[0].weight, None);
    }

    #[test]
    fn a_variant_without_a_barcode_has_none() {
        let mut without = raw_variant(380000.0, 0.2);
        without
            .as_object_mut()
            .unwrap()
            .remove("barcode")
            .expect("the fixture has a barcode");

        let record = record(0.0, vec![raw_variant(380000.0, 0.2), without]).unwrap();

        assert_eq!(record.variants[0].barcode.as_deref(), Some("194644098117"));
        assert_eq!(record.variants[1].barcode, None);
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
            "Sạc Anker <b>323</b> (33W)"
        );
    }

    #[test]
    fn a_variant_option_does_not_carry_the_variant_gid_as_its_id() {
        let record = record(0.0, vec![raw_variant(380000.0, 0.2)]).unwrap();

        assert_eq!(
            serde_json::to_value(&record.variants[0].options[0]).unwrap(),
            json!({"name": "Màu sắc", "display_name": "màu sắc", "values": [{"label": "Đá đen"}]})
        );
    }

    #[test]
    fn the_record_survives_the_round_trip_the_next_stage_makes() {
        let record = record(450000.0, vec![raw_variant(380000.0, 0.2)]).unwrap();

        let json = serde_json::to_string(&record).unwrap();

        assert_eq!(
            serde_json::from_str::<ProductRecord>(&json).unwrap(),
            record
        );
    }
}
