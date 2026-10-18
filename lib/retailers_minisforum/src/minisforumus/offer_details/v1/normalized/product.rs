use super::super::raw::ldjsonitem::Product as Island;
use super::super::raw::productjson::{self, ProductJson, SellingPlanGroup};
use super::{
    NormalizeError, https_from_protocol_relative, money_from_shopify_cents, offer_currency,
    product_gid, variant_gid,
};
use money::{Currency, Money};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Product {
    pub id: String,
    pub handle: String,
    /// The island's URL: the product's address in the page's market (`/en-ca/products/...`).
    pub url: String,
    pub title: String,
    /// HTML.
    pub description: String,
    // Commented out: equal to `description` on every page.
    // pub content: String,
    /// `MINISFORUM US` on every page, whichever market renders it.
    pub vendor: String,
    /// The island's `brand`.
    pub brand: String,
    /// `None` where the product is untyped.
    pub product_type: Option<String>,
    /// Merchandising labels, `active` and collection names among them.
    pub tags: Vec<String>,
    /// RFC 3339, verbatim.
    pub published_at: String,
    /// RFC 3339, verbatim.
    pub created_at: String,
    // Commented out: the lowest variant `price`; `price_min`, `price_max` and `price_varies`
    // restate the variants' prices the same way, as do the island's `lowPrice` and `highPrice`,
    // over the available variants only.
    // pub price: i64,
    // Commented out: each variant carries its own `compare_at_price`; the product-level
    // `compare_at_price` family only summarises those.
    // pub compare_at_price: Option<i64>,
    // Commented out: true exactly where some variant's `available` is, as is the island's
    // `availability`.
    // pub available: bool,
    // Commented out: the record's `type` on every page.
    // pub category: String,
    // Commented out: the record's `tags` joined with `, ` on every page.
    // pub keywords: String,
    // Commented out: the first variant's `sku`, twice.
    // pub sku: Option<String>,
    // pub mpn: Option<String>,
    // Commented out: plain-text renderings of `description`, cut short.
    // pub island_description: String,
    // Commented out: the record's images again, requested at a display width.
    // pub image: String,
    // pub additional_image: Option<Vec<String>>,
    /// The record's names, or none where Shopify's `Title` placeholder stands for a product
    /// without options.
    pub options: Vec<String>,
    pub images: Vec<String>,
    // Commented out: the first of `images` on every page.
    // pub featured_image: Option<String>,
    // Commented out: the images again with dimensions, plus product videos nothing reads.
    // pub media: Vec<Media>,
    // Commented out: false on every page.
    // pub requires_selling_plan: bool,
    pub variants: Vec<Variant>,
}

impl Product {
    pub(crate) fn from_raw(record: ProductJson, island: Island) -> Result<Product, NormalizeError> {
        let currency = offer_currency(&island.offers.price_currency)?;

        if island.name != record.title {
            return Err(NormalizeError::ProductNameMismatch {
                island: island.name,
                record: record.title,
            });
        }

        let path = format!("/products/{}", record.handle);
        for url in [&island.url, &island.offers.url] {
            if !url.ends_with(&path) {
                return Err(NormalizeError::ProductUrlMismatch {
                    url: url.clone(),
                    handle: record.handle,
                });
            }
        }

        if island.offers.offer_count != record.variants.len() as u64 {
            return Err(NormalizeError::OfferCountMismatch {
                island: island.offers.offer_count,
                record: record.variants.len(),
            });
        }

        let options = if is_default_title(&record) {
            Vec::new()
        } else {
            record.options
        };

        let groups = record.selling_plan_groups;
        let variants = record
            .variants
            .into_iter()
            .map(|variant| Variant::from_raw(variant, &options, &groups, currency))
            .collect::<Result<_, _>>()?;

        Ok(Product {
            id: product_gid(record.id),
            handle: record.handle,
            url: island.url,
            title: record.title,
            description: record.description,
            vendor: record.vendor,
            brand: island.brand.name,
            product_type: non_empty(record.product_type),
            tags: record.tags,
            published_at: record.published_at,
            created_at: record.created_at,
            options,
            images: record
                .images
                .into_iter()
                .map(https_from_protocol_relative)
                .collect(),
            variants,
        })
    }
}

// A product without options still has one variant, which Shopify files under a `Title` option
// valued `Default Title`.
fn is_default_title(record: &ProductJson) -> bool {
    record.options == ["Title"]
        && record
            .variants
            .iter()
            .all(|variant| variant.options == ["Default Title"])
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Variant {
    pub id: String,
    /// The selected values joined with ` / `, or `Default Title`.
    pub title: String,
    // Commented out: `title` again, or null where it is `Default Title`.
    // pub public_title: Option<String>,
    // Commented out: `options` again, one slot per product option.
    // pub option1: Option<String>,
    /// The product title and `title`, joined with ` - `; the bare product title for a product
    /// without options.
    pub name: String,
    /// `None` on the gift card.
    pub sku: Option<String>,
    /// Null on every page so far.
    pub barcode: Option<String>,
    /// `None` where the storefront renders 0: the variant is not sold.
    pub price: Option<Money>,
    /// `None` where the variant is not marked down.
    pub compare_at_price: Option<Money>,
    pub available: bool,
    pub requires_shipping: bool,
    pub taxable: bool,
    /// Unit unknown: Liquid documents grams. `None` where the variant is unweighed, as every
    /// variant is so far.
    pub weight: Option<i64>,
    // Commented out: null on every variant.
    // pub featured_image: Option<Value>,
    // Commented out: `shopify` on every variant; stock is tracked, but no quantity is rendered.
    // pub inventory_management: Option<String>,
    // Commented out: one unit, no maximum, in steps of one, on every variant.
    // pub quantity_rule: QuantityRule,
    // Commented out: false on every variant.
    // pub requires_selling_plan: bool,
    pub options: Vec<VariantOption>,
    /// The plans the variant can also be bought under; so far only a full-payment preorder, at
    /// the variant's own price.
    pub selling_plans: Vec<SellingPlan>,
}

impl Variant {
    fn from_raw(
        raw: productjson::Variant,
        offered: &[String],
        groups: &[SellingPlanGroup],
        currency: Currency,
    ) -> Result<Variant, NormalizeError> {
        let options = if offered.is_empty() {
            Vec::new()
        } else {
            variant_options(raw.id, offered, raw.options)?
        };

        let selling_plans = raw
            .selling_plan_allocations
            .into_iter()
            .map(|allocation| SellingPlan::from_raw(raw.id, allocation, groups, currency))
            .collect::<Result<_, _>>()?;

        Ok(Variant {
            id: variant_gid(raw.id),
            title: raw.title,
            // public_title: raw.public_title,
            // option1: raw.option1,
            name: raw.name,
            sku: raw.sku.and_then(non_empty),
            barcode: raw.barcode.and_then(non_empty),
            price: unsold_placeholder(raw.price)
                .map(|cents| money_from_shopify_cents(cents, currency)),
            compare_at_price: raw
                .compare_at_price
                .and_then(unsold_placeholder)
                .map(|cents| money_from_shopify_cents(cents, currency)),
            available: raw.available,
            requires_shipping: raw.requires_shipping,
            taxable: raw.taxable,
            weight: unweighed(raw.weight),
            // featured_image: raw.featured_image,
            // inventory_management: raw.inventory_management,
            // quantity_rule: raw.quantity_rule,
            // requires_selling_plan: raw.requires_selling_plan,
            options,
            selling_plans,
        })
    }
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SellingPlan {
    /// The plan's group, such as `Preorder`.
    pub group: String,
    pub name: String,
    pub price: Money,
    pub compare_at_price: Option<Money>,
    // Commented out: `price` again, on every allocation.
    // pub per_delivery_price: i64,
    // Commented out: none on any allocation or plan.
    // pub price_adjustments: Vec<Value>,
    // Commented out: 100 percent at checkout, no recurring deliveries, on every plan.
    // pub checkout_charge: CheckoutCharge,
}

impl SellingPlan {
    fn from_raw(
        variant_id: u64,
        allocation: productjson::SellingPlanAllocation,
        groups: &[SellingPlanGroup],
        currency: Currency,
    ) -> Result<SellingPlan, NormalizeError> {
        // An allocation names its plan only by id; the plan and its group live on the product.
        let (group, plan) = groups
            .iter()
            .filter(|group| group.id == allocation.selling_plan_group_id)
            .find_map(|group| {
                group
                    .selling_plans
                    .iter()
                    .find(|plan| plan.id == allocation.selling_plan_id)
                    .map(|plan| (group, plan))
            })
            .ok_or(NormalizeError::UnknownSellingPlan {
                variant_id,
                selling_plan_id: allocation.selling_plan_id,
            })?;

        Ok(SellingPlan {
            group: group.name.clone(),
            name: plan.name.clone(),
            price: money_from_shopify_cents(allocation.price, currency),
            compare_at_price: allocation
                .compare_at_price
                .and_then(unsold_placeholder)
                .map(|cents| money_from_shopify_cents(cents, currency)),
        })
    }
}

// 0 is not a price the storefront sells at; it renders it on variants it does not offer, and as
// the compare-at price of variants that are not marked down.
fn unsold_placeholder(cents: i64) -> Option<i64> {
    (cents != 0).then_some(cents)
}

// Shopify defaults an unentered weight to 0: unmeasured, not weightless.
fn unweighed(weight: i64) -> Option<i64> {
    (weight != 0).then_some(weight)
}

fn non_empty(value: String) -> Option<String> {
    (!value.is_empty()).then_some(value)
}
