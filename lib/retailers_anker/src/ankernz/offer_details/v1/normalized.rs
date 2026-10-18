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
pub struct AnkerNzNormalizedDocument {
    // Commented out: `nz` on every page; the retailer code already names the storefront.
    // pub locale: String,
    pub ld_json: ldjson::LdJson,
    pub canonical_url: String,
    // Commented out: page metadata beyond `canonical`, lifted above. `language_alternates` names
    // the same product on the other Anker storefronts, should a cross-storefront join need it.
    // pub seo: PageSeo,
    pub product: ProductRecord,
}

impl AnkerNzNormalizedDocument {
    pub fn from_raw(
        raw: raw::AnkerNzRawDocument,
    ) -> Result<AnkerNzNormalizedDocument, NormalizeError> {
        let next_data = raw.next_data.data;
        let page_props = next_data.props.page_props;

        Ok(AnkerNzNormalizedDocument {
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
    pub description: String,
    // Commented out: the same copy with markup; adds no value for now.
    // pub description_html: String,
    // Commented out: storefront bookkeeping - mostly `Anker AU`, the store this one is served
    // from - rather than a sub-brand.
    // pub vendor: String,
    pub specifications: Vec<Specification>,
    pub product_type: Option<String>,
    pub published_at: DateTime<Utc>,
    /// Labelled `USD` as the page states it, though the amounts are the AU store's AUD prices.
    pub price: Money,
    /// `None` where the product is not marked down.
    pub compare_at_price: Option<Money>,
    pub available_for_sale: bool,
    // Commented out: the sum of the variants' `quantity_available` on every page.
    // pub total_inventory: i64,
    // Commented out: the island's `images`, the same URLs in the same order on every page.
    // pub images: Vec<ProductImage>,
    // Commented out: merchandising labels, not needed for now.
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
    // Commented out: on one image in the corpus, holding an upload date and the SKU rather than
    // alt text.
    // pub alt_text: Option<String>,
    pub width: i64,
    pub height: i64,
}

impl ProductImage {
    fn from_raw(raw: raw::nextdata::product::Image) -> ProductImage {
        ProductImage {
            url: raw.url,
            // alt_text: raw.alt_text,
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
    /// Labelled `USD` as the page states it, though the amounts are the AU store's AUD prices.
    pub price: Money,
    pub compare_at_price: Option<Money>,
    pub quantity_available: i64,
    pub available_for_sale: bool,
    pub currently_not_in_stock: bool,
    pub requires_shipping: bool,
    /// Unit unknown: the payload states none, and the figures are the AU store's, whose unit is
    /// itself open. `None` where the variant is unweighed.
    pub weight: Option<f64>,
    pub image: ProductImage,
    pub options: Vec<VariantOption>,
    // Commented out: an empty object on every variant.
    // pub metafields: Value,
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
            // metafields: raw.metafields,
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

    fn usd(amount: i64) -> Money {
        Money::from_raw(amount as f64 / 100.0, "USD").unwrap()
    }

    fn raw_variant(list_price: Value, weight: f64) -> Value {
        json!({
            "id": "gid://shopify/ProductVariant/1", "sku": "A1259011", "barcode": null,
            "name": "Black", "price": 99.95, "listPrice": list_price, "quantityAvailable": 0,
            "availableForSale": false, "currentlyNotInStock": true, "requiresShipping": true,
            "weight": weight, "image": {"url": "u", "width": 1, "height": 1},
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
            "id": "gid://shopify/Product/1", "handle": "a1259", "slug": "a1259",
            "path": "/a1259", "name": "n", "title": "Anker <b>334</b> MagGo Battery",
            "description": "d", "descriptionHtml": "<p>d</p>", "vendor": "Anker AU",
            "publishedAt": "2025-09-04T06:34:16Z",
            "price": {"value": 99.95, "currencyCode": "USD"}, "listPrice": list_price,
            "availableForSale": false, "totalInventory": 0, "images": [], "tags": [],
            "options": [], "collections": {"nodes": []}, "seo": {},
            "requiresSellingPlan": false, "sellingPlanGroups": [], "variants": variants,
            "metafields": {}, "metafieldsMap": {}
        })
    }

    fn record(raw: Value) -> Result<ProductRecord, NormalizeError> {
        ProductRecord::from_raw(serde_json::from_value(raw).unwrap(), &json!({}))
    }

    #[test]
    fn a_zero_list_price_is_no_markdown_and_one_equal_to_the_price_still_crosses() {
        assert_eq!(
            record(raw_product(0.0, vec![])).unwrap().compare_at_price,
            None
        );
        assert_eq!(
            record(raw_product(99.95, vec![])).unwrap().compare_at_price,
            Some(usd(9995))
        );
    }

    #[test]
    fn a_variant_image_carrying_alt_text_still_normalizes() {
        let mut variant = raw_variant(json!(null), 0.2);
        variant["image"]["altText"] = json!("2024-11-05A8895011");

        let record = record(raw_product(0.0, vec![variant])).unwrap();

        assert_eq!(record.variants[0].image.url, "u");
    }

    #[test]
    fn the_pages_usd_label_crosses_as_stated() {
        let record = record(raw_product(0.0, vec![raw_variant(json!(null), 0.2)])).unwrap();

        assert_eq!(record.price, usd(9995));
        assert_eq!(record.variants[0].price, usd(9995));
    }

    #[test]
    fn a_variant_list_price_crosses_verbatim() {
        let record = record(raw_product(
            0.0,
            vec![
                raw_variant(json!(99.95), 0.2),
                raw_variant(json!(null), 0.2),
            ],
        ))
        .unwrap();

        assert_eq!(record.variants[0].compare_at_price, Some(usd(9995)));
        assert_eq!(record.variants[1].compare_at_price, None);
    }

    #[test]
    fn an_unweighed_variant_has_no_weight() {
        let record = record(raw_product(
            0.0,
            vec![
                raw_variant(json!(null), 0.0),
                raw_variant(json!(null), 0.2127),
            ],
        ))
        .unwrap();

        assert_eq!(record.variants[0].weight, None);
        assert_eq!(record.variants[1].weight, Some(0.2127));
    }

    #[test]
    fn a_published_at_that_is_not_an_instant_fails_with_its_value() {
        let mut raw = raw_product(0.0, vec![]);
        raw["publishedAt"] = json!("2025-09-04");

        assert!(matches!(
            record(raw).unwrap_err(),
            NormalizeError::PublishedAtNotAnInstant { value, .. } if value == "2025-09-04"
        ));
    }

    #[test]
    fn the_title_crosses_with_its_markup() {
        assert_eq!(
            record(raw_product(0.0, vec![])).unwrap().title,
            "Anker <b>334</b> MagGo Battery"
        );
    }

    #[test]
    fn a_variant_option_does_not_carry_the_variant_gid_as_its_id() {
        let record = record(raw_product(0.0, vec![raw_variant(json!(null), 0.2)])).unwrap();

        assert_eq!(
            serde_json::to_value(&record.variants[0].options[0]).unwrap(),
            json!({
                "name": "Color", "display_name": "color",
                "values": [{"label": "Black", "hex_colors": ["#000000"]}]
            })
        );
    }

    #[test]
    fn the_record_survives_the_round_trip_the_next_stage_makes() {
        let record = record(raw_product(
            99.95,
            vec![
                raw_variant(json!(99.95), 0.2),
                raw_variant(json!(null), 0.0),
            ],
        ))
        .unwrap();

        let json = serde_json::to_string(&record).unwrap();

        assert_eq!(
            serde_json::from_str::<ProductRecord>(&json).unwrap(),
            record
        );
    }
}
