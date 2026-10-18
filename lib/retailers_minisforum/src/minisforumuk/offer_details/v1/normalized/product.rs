use super::super::raw::productjson;
use super::island::Island;
use super::{
    NormalizeError, ORIGIN, https_from_protocol_relative, money_from_shopify_cents, product_gid,
    variant_gid,
};
use money::{Currency, Money};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Product {
    pub id: String,
    pub handle: String,
    /// The handle's page on the storefront's origin.
    pub url: String,
    pub title: String,
    /// HTML. `None` where the storefront leaves it empty.
    pub description: Option<String>,
    // Commented out: equal to `description` on every page.
    // pub content: String,
    /// Verbatim RFC 3339 instants in the shop's offset.
    pub published_at: String,
    pub created_at: String,
    /// `Minisforum UK`, or `xcotton` on the app's own products.
    pub vendor: String,
    /// `None` where the product is untyped.
    pub product_type: Option<String>,
    /// The island's Google product category. `None` where the page renders no product island.
    pub category: Option<String>,
    /// Merchandising labels, among them sale campaigns and the dispatch warehouse.
    pub tags: Vec<String>,
    // Commented out: `price`, `price_min`, `price_max` and `price_varies` restate the variants'
    // prices; the `compare_at_price` family restates their `compare_at_price`s.
    // pub price: i64,
    // Commented out: true exactly where some variant's `available` is.
    // pub available: bool,
    pub options: Vec<String>,
    pub images: Vec<String>,
    // Commented out: the first of `images` on every page.
    // pub featured_image: Option<String>,
    // Commented out: the images again with dimensions, plus one product video nothing reads.
    // pub media: Vec<Media>,
    // Commented out: no selling plans on any page; `requires_selling_plan` is false throughout.
    // pub selling_plan_groups: Vec<Value>,
    pub variants: Vec<Variant>,
}

impl Product {
    pub(super) fn from_raw(
        raw: productjson::ProductJson,
        island: Option<Island>,
        currency: Currency,
    ) -> Result<Product, NormalizeError> {
        let url = format!("{ORIGIN}/products/{}", raw.handle);

        let category = match island {
            Some(island) => {
                check_island(&island, &raw, &url, currency)?;
                Some(island.category)
            }
            None => None,
        };

        let options = raw.options;
        let variants = raw
            .variants
            .into_iter()
            .map(|variant| Variant::from_raw(variant, &options, currency))
            .collect::<Result<_, _>>()?;

        Ok(Product {
            id: product_gid(raw.id),
            handle: raw.handle,
            url,
            title: raw.title,
            description: non_empty(raw.description),
            // content: raw.content,
            published_at: raw.published_at,
            created_at: raw.created_at,
            vendor: raw.vendor,
            product_type: non_empty(raw.product_type),
            category,
            tags: raw.tags,
            // price: raw.price,
            // available: raw.available,
            options,
            images: raw
                .images
                .into_iter()
                .map(https_from_protocol_relative)
                .collect(),
            // featured_image: raw.featured_image,
            // media: raw.media,
            // selling_plan_groups: raw.selling_plan_groups,
            variants,
        })
    }
}

// The island renders from the same Liquid product, so any disagreement means one of the two was
// misread.
fn check_island(
    island: &Island,
    record: &productjson::ProductJson,
    url: &str,
    currency: Currency,
) -> Result<(), NormalizeError> {
    let record_id = record.id.to_string();
    if island.url != url
        || island
            .product_id
            .as_ref()
            .is_some_and(|id| *id != record_id)
    {
        return Err(NormalizeError::IslandProductMismatch {
            island: island
                .product_id
                .clone()
                .unwrap_or_else(|| island.url.clone()),
            record: record_id,
        });
    }

    let island_ids: Vec<u64> = island.offers.iter().map(|offer| offer.variant_id).collect();
    let record_ids: Vec<u64> = record.variants.iter().map(|variant| variant.id).collect();
    let (mut island_sorted, mut record_sorted) = (island_ids.clone(), record_ids.clone());
    island_sorted.sort_unstable();
    record_sorted.sort_unstable();
    if island_sorted != record_sorted {
        return Err(NormalizeError::VariantIdsMismatch {
            island: island_ids,
            record: record_ids,
        });
    }

    for offer in &island.offers {
        if offer.currency != currency.as_str() {
            return Err(NormalizeError::IslandCurrencyMismatch {
                offer: offer.id.clone(),
                island: offer.currency.clone(),
                shop: currency.as_str().to_string(),
            });
        }

        let variant = record
            .variants
            .iter()
            .find(|variant| variant.id == offer.variant_id)
            .expect("the id sets are equal");
        let mismatch = |field: &str, island: String, record: String| {
            Err(NormalizeError::VariantFieldMismatch {
                variant_id: variant.id,
                field: field.to_string(),
                island,
                record,
            })
        };

        if offer.cents != variant.price {
            return mismatch("price", offer.cents.to_string(), variant.price.to_string());
        }
        if offer.in_stock != variant.available {
            return mismatch(
                "available",
                offer.in_stock.to_string(),
                variant.available.to_string(),
            );
        }
        let record_sku = variant.sku.clone().and_then(non_empty);
        if offer.sku != record_sku {
            return mismatch("sku", format!("{:?}", offer.sku), format!("{record_sku:?}"));
        }
    }

    Ok(())
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Variant {
    pub id: String,
    /// The selected values joined with ` / `, or `Default Title` where the product has no
    /// options to choose.
    pub title: String,
    // Commented out: `title` again, or null where it is `Default Title`.
    // pub public_title: Option<String>,
    // Commented out: `options` again, one slot per product option.
    // pub option1: Option<String>,
    /// The product title and `title`, joined with ` - `; the product title alone where there is
    /// no choice.
    pub name: String,
    pub sku: Option<String>,
    /// Null on every page so far.
    pub barcode: Option<String>,
    /// `None` where the storefront renders £0: the product is announced but not sold.
    pub price: Option<Money>,
    /// `None` where the variant is not marked down, including where the storefront renders £0.
    pub compare_at_price: Option<Money>,
    pub available: bool,
    pub requires_shipping: bool,
    pub taxable: bool,
    /// Unit unknown: Liquid documents grams. `None` where the variant is unweighed, as every
    /// variant is so far.
    pub weight: Option<i64>,
    /// `featured_image.src`. `None` where the variant has no image of its own.
    pub image: Option<String>,
    // Commented out: the same image as a media entry.
    // pub featured_media: Option<FeaturedMedia>,
    // Commented out: `shopify` or null; stock may be tracked, but no quantity is rendered.
    // pub inventory_management: Option<String>,
    // Commented out: one unit, no maximum, in steps of one, on every variant.
    // pub quantity_rule: QuantityRule,
    // Commented out: no selling plans on any page.
    // pub selling_plan_allocations: Vec<Value>,
    pub options: Vec<VariantOption>,
}

impl Variant {
    fn from_raw(
        raw: productjson::Variant,
        offered: &[String],
        currency: Currency,
    ) -> Result<Variant, NormalizeError> {
        Ok(Variant {
            id: variant_gid(raw.id),
            title: raw.title,
            // public_title: raw.public_title,
            // option1: raw.option1,
            name: raw.name,
            sku: raw.sku.and_then(non_empty),
            barcode: raw.barcode.and_then(non_empty),
            price: unsold_placeholder_price(raw.price)
                .map(|cents| money_from_shopify_cents(cents, currency)),
            compare_at_price: raw
                .compare_at_price
                .and_then(unsold_placeholder_price)
                .map(|cents| money_from_shopify_cents(cents, currency)),
            available: raw.available,
            requires_shipping: raw.requires_shipping,
            taxable: raw.taxable,
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

// £0 is not a price the storefront sells at; it renders it on a product it only announces, and
// leaves that product's schema.org island out.
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
