use super::super::raw;
use super::{
    NormalizeError, https_from_protocol_relative, instant, product_gid, variant_gid,
    yen_from_shopify_cents,
};
use chrono::{DateTime, Utc};
use money::{Currency, Money};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProductRecord {
    pub id: String,
    pub handle: String,
    pub title: String,
    pub description: String,
    // Commented out: equal to `description` on every page.
    // pub content: String,
    // Commented out: equal to the island's `brand` on every page.
    // pub vendor: String,
    pub published_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub product_type: Option<String>,
    /// Merchandising labels. `eol` among them may mean end of life; nothing reads it as such yet.
    pub tags: Vec<String>,
    // Commented out: the lowest variant `price`; `price_min`, `price_max` and `price_varies`
    // restate the variants' prices the same way.
    // pub price: i64,
    // Commented out: no variant is marked down, and each carries its own `compare_at_price`; the
    // product-level `compare_at_price` family only summarises those.
    // pub compare_at_price: Option<i64>,
    // Commented out: true exactly where some variant's `available` is.
    // pub available: bool,
    pub options: Vec<String>,
    pub images: Vec<String>,
    // Commented out: the first of `images` on every page.
    // pub featured_image: Option<String>,
    // Commented out: the images again with dimensions, plus product videos nothing reads.
    // pub media: Vec<Media>,
    // Commented out: no selling plans on any page; `requires_selling_plan` is false throughout.
    // pub selling_plan_groups: Vec<Value>,
    pub variants: Vec<Variant>,
}

impl ProductRecord {
    pub(crate) fn from_raw(
        raw: raw::productjson::ProductJson,
        currency: Currency,
    ) -> Result<ProductRecord, NormalizeError> {
        let options = raw.options;
        let variants = raw
            .variants
            .into_iter()
            .map(|variant| Variant::from_raw(variant, &options, currency))
            .collect::<Result<_, _>>()?;

        Ok(ProductRecord {
            id: product_gid(raw.id),
            handle: raw.handle,
            title: raw.title,
            description: raw.description,
            // content: raw.content,
            // vendor: raw.vendor,
            published_at: instant(&raw.published_at, "published_at")?,
            created_at: instant(&raw.created_at, "created_at")?,
            product_type: non_empty(raw.product_type),
            tags: raw.tags,
            // price: raw.price,
            // compare_at_price: raw.compare_at_price,
            // available: raw.available,
            images: raw
                .images
                .into_iter()
                .map(https_from_protocol_relative)
                .collect(),
            options,
            // featured_image: raw.featured_image,
            // media: raw.media,
            // selling_plan_groups: raw.selling_plan_groups,
            variants,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Variant {
    pub id: String,
    // Commented out: the selected values joined with ` / `, restated by `options`.
    // pub title: String,
    // Commented out: `options` again, one slot per product option.
    // pub option1: Option<String>,
    pub sku: Option<String>,
    /// The storefront's barcode field verbatim: mostly a JAN, but sometimes the handle or the SKU
    /// typed into it.
    pub barcode: Option<String>,
    /// The product title and `public_title`, joined with ` - `.
    pub name: String,
    // Commented out: `title` again, or null.
    // pub public_title: Option<String>,
    /// `None` where the storefront renders ¥0: the variant is not sold.
    pub price: Option<Money>,
    /// `None` where the variant is not marked down.
    pub compare_at_price: Option<Money>,
    pub available: bool,
    pub requires_shipping: bool,
    // Commented out: true on every variant.
    // pub taxable: bool,
    /// Unit unknown: Liquid documents grams, but the values do not fit the products. `None` where
    /// the variant is unweighed.
    pub weight: Option<i64>,
    /// `featured_image.src`. `None` where the variant has no image of its own.
    pub image: Option<String>,
    // Commented out: the same image as a media entry.
    // pub featured_media: Option<FeaturedMedia>,
    // Commented out: `shopify` on every variant; stock is tracked, but no quantity is rendered.
    // pub inventory_management: Option<String>,
    // Commented out: one unit, no maximum, in steps of one, on every variant.
    // pub quantity_rule: QuantityRule,
    // Commented out: no selling plans on any page.
    // pub selling_plan_allocations: Vec<Value>,
    pub options: Vec<VariantOption>,
}

impl Variant {
    fn from_raw(
        raw: raw::productjson::Variant,
        offered: &[String],
        currency: Currency,
    ) -> Result<Variant, NormalizeError> {
        Ok(Variant {
            id: variant_gid(raw.id),
            // title: raw.title,
            // option1: raw.option1,
            sku: raw.sku.and_then(non_empty),
            barcode: raw.barcode.and_then(non_empty),
            name: raw.name,
            // public_title: raw.public_title,
            price: unsold_placeholder_price(raw.price)
                .map(|cents| yen_from_shopify_cents(cents, currency, "variants.price"))
                .transpose()?,
            compare_at_price: raw
                .compare_at_price
                .map(|cents| yen_from_shopify_cents(cents, currency, "variants.compare_at_price"))
                .transpose()?,
            available: raw.available,
            requires_shipping: raw.requires_shipping,
            // taxable: raw.taxable,
            weight: unweighed(raw.weight),
            image: raw
                .featured_image
                .map(|image| https_from_protocol_relative(image.src)),
            // featured_media: raw.featured_media,
            // inventory_management: raw.inventory_management,
            // quantity_rule: raw.quantity_rule,
            // selling_plan_allocations: raw.selling_plan_allocations,
            options: variant_options(raw.id, offered, raw.options)?,
        })
    }
}

// ¥0 is not a price the storefront sells at; it renders it on variants it no longer offers.
fn unsold_placeholder_price(cents: i64) -> Option<i64> {
    (cents != 0).then_some(cents)
}

// Shopify defaults an unentered weight to 0: unmeasured, not weightless.
fn unweighed(weight: i64) -> Option<i64> {
    (weight != 0).then_some(weight)
}

fn non_empty(value: String) -> Option<String> {
    (!value.is_empty()).then_some(value)
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VariantOption {
    pub name: String,
    pub value: String,
}

// Liquid names the options only on the product, and a variant's values only by position.
fn variant_options(
    variant_id: u64,
    offered: &[String],
    selected: Vec<String>,
) -> Result<Vec<VariantOption>, NormalizeError> {
    if selected.len() != offered.len() {
        return Err(NormalizeError::OptionCountMismatch {
            variant_id,
            offered: offered.len(),
            selected: selected.len(),
        });
    }

    Ok(offered
        .iter()
        .zip(selected)
        .map(|(name, value)| VariantOption {
            name: name.clone(),
            value,
        })
        .collect())
}
