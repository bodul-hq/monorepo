//! Expresses the storefront's product facts in the system's vocabulary. This storefront runs the
//! smaller `[handle]` application, whose record already states what its schema.org island
//! repeats, so the island crosses only as the proof that the page is a product; what nothing
//! needs stays behind as the commented-out exclusion records below.

pub mod ldjson;
pub mod metafields;

use super::raw;
use chrono::{DateTime, Utc};
use metafields::{KeyFeature, Specification};
use money::{Money, MoneyError};
use serde::{Deserialize, Serialize};
use std::num::ParseFloatError;
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Error)]
pub enum NormalizeError {
    #[error("no `Product` ld+json island on the document")]
    MissingProductIsland,

    #[error("updatedAt `{value}` is not an RFC 3339 instant: {source}")]
    UpdatedAtNotAnInstant {
        value: String,
        source: chrono::ParseError,
    },

    #[error("`{field}` is not money: {source:?}")]
    NotMoney { field: String, source: MoneyError },

    #[error("`{field}` `{value}` is not a number: {source}")]
    NotANumber {
        field: String,
        value: String,
        source: ParseFloatError,
    },

    #[error(
        "variant `{variant_id}` renders {selected} selected options against {rendered} option renderings"
    )]
    OptionRenderingCountMismatch {
        variant_id: String,
        selected: usize,
        rendered: usize,
    },

    #[error(
        "variant `{variant_id}` selects option `{selected}` where its rendering names `{rendered}`"
    )]
    OptionRenderingsDisagree {
        variant_id: String,
        selected: String,
        rendered: String,
    },

    #[error("discount `{code}` `{field}` `{value}` is not an RFC 3339 instant: {source}")]
    DiscountInstantNotAnInstant {
        code: String,
        field: String,
        value: String,
        source: chrono::ParseError,
    },

    #[error("discount `{code}` has unknown `value_type` `{value_type}`")]
    UnknownDiscountValueType { code: String, value_type: String },

    #[error("`{path}` is not a list")]
    NotAList { path: String },

    #[error("a `{path}` entry has no string `{field}`")]
    EntryFieldNotAString { path: String, field: String },
}

fn money(amount: f64, currency: &str, field: &str) -> Result<Money, NormalizeError> {
    Money::from_raw(amount, currency).map_err(|source| NormalizeError::NotMoney {
        field: field.to_string(),
        source,
    })
}

fn decimal_money(amount: &str, currency: &str, field: &str) -> Result<Money, NormalizeError> {
    money(decimal(amount, field)?, currency, field)
}

fn decimal(raw: &str, field: &str) -> Result<f64, NormalizeError> {
    raw.parse::<f64>()
        .map_err(|source| NormalizeError::NotANumber {
            field: field.to_string(),
            value: raw.to_string(),
            source,
        })
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AnkerNordicsComNormalizedDocument {
    // Commented out: `se` on every page; the retailer code already names the storefront.
    // pub locale: String,
    pub ld_json: ldjson::LdJson,
    pub canonical_url: String,
    // Commented out: page metadata beyond `canonical`, lifted above. `language_alternates` names
    // the same product under the other Nordic locales, all at the same URL.
    // pub seo: PageSeo,
    pub product: ProductRecord,
    // Commented out: images of the EU's guarantee notices per variant SKU. The statutory notice
    // is the same generic image on every variant; nothing downstream reads the commercial labels.
    // pub legal_guarantee: BTreeMap<String, Vec<LegalGuarantee>>,
}

impl AnkerNordicsComNormalizedDocument {
    pub fn from_raw(
        raw: raw::AnkerNordicsComRawDocument,
    ) -> Result<AnkerNordicsComNormalizedDocument, NormalizeError> {
        let next_data = raw.next_data.data;
        let page_props = next_data.props.page_props;

        Ok(AnkerNordicsComNormalizedDocument {
            // locale: next_data.locale,
            ld_json: ldjson::LdJson::from_raw(raw.ld_json)?,
            canonical_url: page_props.seo.canonical,
            // seo: PageSeo::from_raw(page_props.seo),
            product: ProductRecord::from_raw(page_props.product)?,
            // legal_guarantee: page_props.legal_guarantee,
        })
    }
}

/* Page SEO normalization is disabled beyond `canonical`: nothing downstream reads the rest.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct PageSeo {
    pub title: String,
    pub description: String,

    pub canonical: String,

    pub noindex: bool,
    pub thumbnail: String,
    pub language_alternates: Vec<LanguageAlternate>,
    pub open_graph: OpenGraph,
}

impl PageSeo {
    fn from_raw(raw: raw::nextdata::Seo) -> PageSeo {
        PageSeo {
            title: raw.title,
            description: raw.description,
            canonical: raw.canonical,
            noindex: raw.noindex,
            thumbnail: raw.thumbnail,
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
    pub description: String,
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
    pub title: String,
    pub description: String,
    // Commented out: the same copy with markup; adds no value for now.
    // pub description_html: String,
    /// The sub-brand as the backend store spells it, suffixes included: `eufy EU`, `soundcoreus`.
    /// The page names no other brand; the island's is the storefront's own name.
    pub vendor: String,
    pub key_features: Vec<KeyFeature>,
    pub specifications: Vec<Specification>,
    pub product_type: Option<String>,
    /// The only timestamp the payload carries; it is not a publication date.
    pub updated_at: DateTime<Utc>,
    pub price: Money,
    /// `None` where the product is not marked down.
    pub compare_at_price: Option<Money>,
    // Commented out: the variants' minimum and maximum prices on every page.
    // pub price_range: PriceRange,
    pub available_for_sale: bool,
    // Commented out: `images[0]` on every page.
    // pub featured_image: ProductImage,
    // Commented out: the island's `images`, the same URLs in the same order on every page.
    // pub images: Vec<ProductImage>,
    // Commented out: `images` again, plus one video on one page.
    // pub media: Vec<Media>,
    // Commented out: merchandising labels. Refurbishment is read from the SKU instead, which
    // states it on every refurbished variant where the tag does on a minority.
    // pub tags: Vec<String>,
    // Commented out: storefront metadata nothing downstream reads.
    // pub seo: ProductSeo,
    // Commented out: review widgets (`judgeme`), breadcrumbs (`collection`), a page template
    // name (`template`), links to sibling bundles (`product.data`) and page content blocks
    // (`global`, `component`) beyond the lists lifted above.
    // pub metafields: ProductMetafields,
    pub options: Vec<ProductOption>,
    pub variants: Vec<Variant>,
}

impl ProductRecord {
    fn from_raw(raw: raw::nextdata::product::Product) -> Result<ProductRecord, NormalizeError> {
        let currency = raw.price.currency_code.as_str();

        Ok(ProductRecord {
            id: raw.id,
            handle: raw.handle,
            // slug: raw.slug,
            // path: raw.path,
            title: raw.title,
            description: raw.description,
            // description_html: raw.description_html,
            vendor: raw.vendor,
            key_features: metafields::key_features(&raw.metafields)?,
            specifications: metafields::specifications(&raw.metafields)?,
            product_type: product_type(raw.product_type),
            updated_at: updated_at(&raw.updated_at)?,
            price: money(raw.price.amount, currency, "price.amount")?,
            compare_at_price: compare_at_price(raw.compare_at_price_range.min_variant_price)?,
            // price_range: PriceRange::from_raw(raw.price_range),
            available_for_sale: raw.available_for_sale,
            // featured_image: ProductImage::from_raw(raw.featured_image),
            // images: raw.images.into_iter().map(ProductImage::from_raw).collect(),
            // media: raw.media,
            // tags: raw.tags,
            // seo: ProductSeo::from_raw(raw.seo),
            // metafields: raw.metafields,
            options: raw
                .options
                .into_iter()
                .map(ProductOption::from_raw)
                .collect(),
            variants: raw
                .variants
                .into_iter()
                .map(Variant::from_raw)
                .collect::<Result<_, _>>()?,
        })
    }
}

// The storefront renders an empty string for a product with no type.
fn product_type(raw: String) -> Option<String> {
    (!raw.is_empty()).then_some(raw)
}

fn updated_at(raw: &str) -> Result<DateTime<Utc>, NormalizeError> {
    let parsed = DateTime::parse_from_rfc3339(raw).map_err(|source| {
        NormalizeError::UpdatedAtNotAnInstant {
            value: raw.to_string(),
            source,
        }
    })?;

    Ok(parsed.with_timezone(&Utc))
}

// The range's minimum renders `0.0` rather than being absent when no variant is marked down.
fn compare_at_price(
    raw: raw::nextdata::product::PriceStr,
) -> Result<Option<Money>, NormalizeError> {
    let price = decimal_money(
        &raw.amount,
        &raw.currency_code,
        "compareAtPriceRange.minVariantPrice",
    )?;

    Ok((price.amount() != 0).then_some(price))
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

/* Price-range normalization is disabled: both ends restate the variants' own prices.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct PriceRange {
    pub min_variant_price: Money,
    pub max_variant_price: Money,
}

impl PriceRange {
    fn from_raw(raw: raw::nextdata::product::PriceRange) -> PriceRange {
        PriceRange {
            min_variant_price: money(raw.min_variant_price),
            max_variant_price: money(raw.max_variant_price),
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
pub struct ProductOption {
    pub id: String,
    /// Lowercase, e.g. `color`: the spelling a variant's `display_name` carries.
    pub name: String,
    // Commented out: `MultipleChoiceOption` on every option.
    // pub typename: String,
    pub values: Vec<OptionValue>,
}

impl ProductOption {
    fn from_raw(raw: raw::nextdata::product::ProductOption) -> ProductOption {
        ProductOption {
            id: raw.id,
            name: raw.name,
            // typename: raw.typename,
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Variant {
    pub id: String,
    pub sku: String,
    pub barcode: Option<String>,
    // Commented out: the selected option values joined, `Default Title` where none are offered.
    // pub title: String,
    pub price: Money,
    pub compare_at_price: Option<Money>,
    pub quantity_available: i64,
    pub available_for_sale: bool,
    pub currently_not_in_stock: bool,
    pub image: ProductImage,
    pub options: Vec<VariantOption>,
    pub discounts: Vec<Discount>,
    // Commented out: `discounts` filtered by the storefront, which drops expired codes.
    // pub coupons: Vec<Discount>,
    // Commented out: the parent record again, differing only in its metafields.
    // pub product: Value,
    // Commented out: page content blocks (`component`, `global`), bundle and free-gift
    // configuration (`bundle`, `free_gift`, `discounts_function`) and quantity tiers
    // (`qty_level`), none of which states a price for this variant.
    // pub metafields: VariantMetafields,
}

impl Variant {
    fn from_raw(raw: raw::nextdata::product::Variant) -> Result<Variant, NormalizeError> {
        let currency = raw.price.currency_code.clone();
        let options = variant_options(&raw.id, raw.selected_options, raw.options)?;

        Ok(Variant {
            price: money(raw.price.amount, &currency, "variants[].price.amount")?,
            compare_at_price: raw
                .compare_at_price
                .map(|price| {
                    money(
                        price.amount,
                        &price.currency_code,
                        "variants[].compareAtPrice.amount",
                    )
                })
                .transpose()?,
            quantity_available: raw.quantity_available,
            available_for_sale: raw.available_for_sale,
            currently_not_in_stock: raw.currently_not_in_stock,
            image: ProductImage::from_raw(raw.image),
            options,
            discounts: discounts(raw.metafields.discount_code_app)?,
            // coupons: raw.coupons,
            // product: raw.product,
            // metafields: raw.metafields,
            id: raw.id,
            sku: raw.sku,
            barcode: raw.barcode,
            // title: raw.title,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VariantOption {
    /// From `selectedOptions`, capitalised: `Color`.
    pub name: String,
    /// From the variant's `options`, lowercase: `color`, as the product's own list spells it.
    pub display_name: String,
    // Commented out: holds the variant's own gid rather than an option id, on every option.
    // pub id: String,
    pub values: Vec<OptionValue>,
}

// The variant renders each choice twice: `selectedOptions` with the shopper-facing name, and
// `options` with the product list's spelling and the swatch colours. Paired by position, and
// only when they name the same option.
fn variant_options(
    variant_id: &str,
    selected: Vec<raw::nextdata::product::SelectedOption>,
    rendered: Vec<raw::nextdata::product::VariantOptionRendering>,
) -> Result<Vec<VariantOption>, NormalizeError> {
    if selected.len() != rendered.len() {
        return Err(NormalizeError::OptionRenderingCountMismatch {
            variant_id: variant_id.to_string(),
            selected: selected.len(),
            rendered: rendered.len(),
        });
    }

    selected
        .into_iter()
        .zip(rendered)
        .map(|(selected, rendered)| {
            if !selected.name.eq_ignore_ascii_case(&rendered.name) {
                return Err(NormalizeError::OptionRenderingsDisagree {
                    variant_id: variant_id.to_string(),
                    selected: selected.name,
                    rendered: rendered.name,
                });
            }

            Ok(VariantOption {
                name: selected.name,
                display_name: rendered.name,
                values: rendered
                    .values
                    .into_iter()
                    .map(OptionValue::from_raw)
                    .collect(),
            })
        })
        .collect()
}

fn discounts(
    raw: Option<raw::nextdata::product::DiscountCodeApp>,
) -> Result<Vec<Discount>, NormalizeError> {
    raw.map(|app| app.data)
        .unwrap_or_default()
        .into_iter()
        .map(Discount::from_raw)
        .collect()
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
    /// The channels the code is limited to, e.g. `["app"]`. What `None` means is not established.
    pub sales_channels: Option<Vec<String>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DiscountValue {
    /// Signed percentage points as supplied by the page: `-20.0` means 20% off.
    Percentage(f64),
    FixedAmount(Money),
}

impl Discount {
    fn from_raw(raw: raw::nextdata::product::DiscountCode) -> Result<Discount, NormalizeError> {
        let instant = |field: &str, value: &str| {
            DateTime::parse_from_rfc3339(value)
                .map(|instant| instant.with_timezone(&Utc))
                .map_err(|source| NormalizeError::DiscountInstantNotAnInstant {
                    code: raw.title.clone(),
                    field: field.to_string(),
                    value: value.to_string(),
                    source,
                })
        };

        let value = match raw.value_type.as_str() {
            "percentage" => DiscountValue::Percentage(decimal(&raw.value, "value")?),
            "fixed_amount" => {
                DiscountValue::FixedAmount(decimal_money(&raw.value, &raw.currency, "value")?)
            }
            other => {
                return Err(NormalizeError::UnknownDiscountValueType {
                    code: raw.title,
                    value_type: other.to_string(),
                });
            }
        };

        Ok(Discount {
            value,
            price: money(
                raw.variant_price4wscode,
                &raw.currency,
                "variant_price4wscode",
            )?,
            saving: decimal_money(&raw.fixed_value, &raw.currency, "fixed_value")?,
            starts_at: instant("starts_at", &raw.starts_at)?,
            ends_at: raw
                .ends_at
                .as_deref()
                .map(|ends_at| instant("ends_at", ends_at))
                .transpose()?,
            code: raw.title,
            kind: raw.discount_type,
            variant_shopify_id: raw.variant_shopify_id,
            amazon_deal: raw.amazon_deal,
            sales_channels: raw.sales_channel,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use html_extractor::serde_json::{self, Value, json};

    fn sek(amount: f64) -> Money {
        Money::from_raw(amount, "SEK").unwrap()
    }

    fn code(title: &str, ends_at: &str) -> Value {
        json!({
            "title": title, "starts_at": "2026-08-12T02:59:59.000Z", "ends_at": ends_at,
            "variant_shopify_id": 1, "value_type": "percentage", "value": "-50.0",
            "currency": "SEK", "fixed_value": "6495.0", "value_style": "kr50",
            "variant_price4wscode": 6495, "discount_type": "WSAP", "amazon_deal": false,
            "sales_channel": ["app"], "cdi_id": 1, "cdi_updated_at": "2026-08-10T07:21:07.765Z"
        })
    }

    fn raw_variant(amount: Value) -> Value {
        json!({
            "id": "gid://shopify/ProductVariant/1", "sku": "A3947G11-F0", "barcode": null,
            "title": "Black", "price": {"amount": amount, "currencyCode": "SEK"},
            "compareAtPrice": null, "quantityAvailable": 3, "availableForSale": true,
            "currentlyNotInStock": false,
            "image": {"url": "u", "altText": null, "width": 1, "height": 1},
            "options": [{
                "__typename": "MultipleChoiceOption", "id": "gid://shopify/ProductVariant/1",
                "name": "color", "values": [{"label": "Black", "hexColors": ["#000000"]}]
            }],
            "selectedOptions": [{"name": "Color", "value": "Black"}],
            "coupons": [],
            "product": {},
            "metafields": {"__discountCodeApp": {"data": [code("WSAP8V6ZVXRG", "2026-08-14T02:59:59.000Z")]}}
        })
    }

    fn raw_product(compare_at_min: &str, variants: Vec<Value>, metafields: Value) -> Value {
        json!({
            "id": "gid://shopify/Product/1", "handle": "a3947g11-f0", "slug": "a3947g11-f0",
            "path": "/a3947g11-f0", "title": "Anker <b>315</b> Wireless Charger (Pad)",
            "description": "d", "descriptionHtml": "<p>d</p>", "vendor": "soundcoreeu",
            "productType": "", "updatedAt": "2026-09-01T10:00:00Z", "availableForSale": true,
            "tags": [], "price": {"amount": 2490, "currencyCode": "SEK"},
            "priceRange": {
                "minVariantPrice": {"amount": "2490.0", "currencyCode": "SEK"},
                "maxVariantPrice": {"amount": "2490.0", "currencyCode": "SEK"}
            },
            "compareAtPriceRange": {"minVariantPrice": {"amount": compare_at_min, "currencyCode": "SEK"}},
            "featuredImage": {"url": "u", "altText": null, "width": 1, "height": 1},
            "images": [], "seo": {"title": null, "description": null}, "media": [],
            "options": [], "variants": variants, "metafields": metafields
        })
    }

    fn record(raw: Value) -> Result<ProductRecord, NormalizeError> {
        ProductRecord::from_raw(serde_json::from_value(raw).unwrap())
    }

    #[test]
    fn a_zero_compare_at_minimum_is_no_markdown_and_a_real_one_crosses() {
        assert_eq!(
            record(raw_product("0.0", vec![], json!({})))
                .unwrap()
                .compare_at_price,
            None
        );
        assert_eq!(
            record(raw_product("2990.0", vec![], json!({})))
                .unwrap()
                .compare_at_price,
            Some(sek(2990.0))
        );
    }

    #[test]
    fn integer_and_float_amounts_both_become_money() {
        let record = record(raw_product(
            "0.0",
            vec![raw_variant(json!(2490)), raw_variant(json!(23443.3))],
            json!({}),
        ))
        .unwrap();

        assert_eq!(record.price, sek(2490.0));
        assert_eq!(record.variants[0].price, sek(2490.0));
        assert_eq!(record.variants[1].price, sek(23443.3));
    }

    #[test]
    fn an_empty_product_type_is_none() {
        assert_eq!(
            record(raw_product("0.0", vec![], json!({})))
                .unwrap()
                .product_type,
            None
        );
    }

    #[test]
    fn an_updated_at_that_is_not_an_instant_fails_with_its_value() {
        let mut raw = raw_product("0.0", vec![], json!({}));
        raw["updatedAt"] = json!("2026-09-01");

        assert!(matches!(
            record(raw).unwrap_err(),
            NormalizeError::UpdatedAtNotAnInstant { value, .. } if value == "2026-09-01"
        ));
    }

    #[test]
    fn the_vendor_and_the_title_cross_verbatim() {
        let record = record(raw_product("0.0", vec![], json!({}))).unwrap();

        assert_eq!(record.vendor, "soundcoreeu");
        assert_eq!(record.title, "Anker <b>315</b> Wireless Charger (Pad)");
    }

    #[test]
    fn the_two_option_renderings_merge_into_one_option() {
        let record = record(raw_product(
            "0.0",
            vec![raw_variant(json!(2490))],
            json!({}),
        ))
        .unwrap();

        assert_eq!(
            serde_json::to_value(&record.variants[0].options).unwrap(),
            json!([{
                "name": "Color", "display_name": "color",
                "values": [{"label": "Black", "hex_colors": ["#000000"]}]
            }])
        );
    }

    #[test]
    fn option_renderings_that_name_different_options_fail() {
        let mut variant = raw_variant(json!(2490));
        variant["selectedOptions"][0]["name"] = json!("Size");

        assert!(matches!(
            record(raw_product("0.0", vec![variant], json!({}))).unwrap_err(),
            NormalizeError::OptionRenderingsDisagree { selected, rendered, .. }
                if selected == "Size" && rendered == "color"
        ));
    }

    #[test]
    fn option_renderings_of_different_lengths_fail() {
        let mut variant = raw_variant(json!(2490));
        variant["selectedOptions"] = json!([]);

        assert!(matches!(
            record(raw_product("0.0", vec![variant], json!({}))).unwrap_err(),
            NormalizeError::OptionRenderingCountMismatch {
                selected: 0,
                rendered: 1,
                ..
            }
        ));
    }

    #[test]
    fn discounts_come_from_the_app_namespace_including_a_code_the_coupons_dropped() {
        let record = record(raw_product(
            "0.0",
            vec![raw_variant(json!(12990))],
            json!({}),
        ))
        .unwrap();
        let discounts = &record.variants[0].discounts;

        assert_eq!(discounts.len(), 1);
        assert_eq!(discounts[0].code, "WSAP8V6ZVXRG");
        assert_eq!(discounts[0].value, DiscountValue::Percentage(-50.0));
        assert_eq!(discounts[0].price, sek(6495.0));
        assert_eq!(discounts[0].saving, sek(6495.0));
        assert_eq!(discounts[0].sales_channels, Some(vec!["app".to_string()]));
    }

    #[test]
    fn a_variant_without_the_app_namespace_has_no_discounts() {
        let mut variant = raw_variant(json!(2490));
        variant["metafields"] = json!({"component": {}});

        let record = record(raw_product("0.0", vec![variant], json!({}))).unwrap();

        assert!(record.variants[0].discounts.is_empty());
    }

    #[test]
    fn the_record_survives_the_round_trip_the_next_stage_makes() {
        let record = record(raw_product(
            "2990.0",
            vec![raw_variant(json!(2490))],
            json!({"product": {"infos": {"key_features": {"list": [{"key": "k", "value": "v"}]}}}}),
        ))
        .unwrap();

        let json = serde_json::to_string(&record).unwrap();

        assert_eq!(
            serde_json::from_str::<ProductRecord>(&json).unwrap(),
            record
        );
    }
}
