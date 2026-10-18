use super::super::raw::productjson;
use super::island::Island;
use super::{
    Locale, NormalizeError, gtin, https_from_protocol_relative, money_from_shopify_cents,
    non_empty, product_gid, product_url, variant_gid,
};
use money::{Currency, Money};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Product {
    pub id: String,
    pub handle: String,
    /// The handle's page on the storefront's origin, in the page's locale: under `/hi` for the
    /// Hindi pages, percent-encoded where the handle is not plain ASCII. The island states the
    /// same URL on every saved page.
    pub url: String,
    /// In the page's locale, as are `description`, `product_type`, `tags`, `options` and the
    /// variants' `title`, `name` and option values, where the shop has translated them.
    pub title: String,
    /// HTML. `None` where the storefront leaves it empty, as no saved page does.
    pub description: Option<String>,
    // Commented out: equal to `description` on every page.
    // pub content: String,
    /// Verbatim RFC 3339 instants in the shop's offset (+05:30).
    pub published_at: String,
    pub created_at: String,
    /// `UGREEN`, or `Elektrik` on one product.
    pub vendor: String,
    /// `None` where the product is untyped, as no saved product is.
    pub product_type: Option<String>,
    /// The island's Google product category, in English on both locales.
    pub category: String,
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
    // Commented out: the images again with dimensions and alt texts.
    // pub media: Vec<Media>,
    // Commented out: a pre-order plan on 5 products per locale ("Full upfront payment, ships 15
    // days after checkout") with a 0% price adjustment, so the variants' prices stand; no
    // variant requires it.
    // pub selling_plan_groups: Vec<SellingPlanGroup>,
    pub variants: Vec<Variant>,
}

impl Product {
    pub(super) fn from_raw(
        raw: productjson::ProductJson,
        island: Island,
        currency: Currency,
        locale: Locale,
    ) -> Result<Product, NormalizeError> {
        let url = product_url(locale, &raw.handle);
        check_island(&island, &raw, &url, currency)?;

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
            category: island.category,
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
            record: format!("{record_id} at {url}"),
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

        // Always found: the id sets were compared above.
        let Some(variant) = record
            .variants
            .iter()
            .find(|variant| variant.id == offer.variant_id)
        else {
            return Err(NormalizeError::VariantIdsMismatch {
                island: island_ids.clone(),
                record: record_ids.clone(),
            });
        };
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
        if Some(&offer.sku) != record_sku.as_ref() {
            return mismatch("sku", offer.sku.clone(), format!("{record_sku:?}"));
        }
        // Compared before the barcode is screened: the island repeats mangled values too.
        if offer.gtin != variant.barcode {
            return mismatch(
                "gtin",
                format!("{:?}", offer.gtin),
                format!("{:?}", variant.barcode),
            );
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
    /// Set on every saved variant.
    pub sku: Option<String>,
    /// The barcode where it is all digits, after trimming whitespace. 13 digits on 450 saved
    /// variants per locale and 12 on one; null on 230. `None` too where the shop stores a
    /// spreadsheet-mangled number, which is no GTIN: `6.96E+12` on 23 variants per locale and
    /// `6.94E+12` on 7, 30 per locale (60 pages) in all.
    pub gtin: Option<String>,
    /// `None` where Shopify renders 0, its unset price; no saved variant has it.
    pub price: Option<Money>,
    /// `None` where the variant is not marked down: null on 4 saved variants per locale, above
    /// `price` on the other 707.
    pub compare_at_price: Option<Money>,
    /// False on 126 saved variants per locale.
    pub available: bool,
    pub requires_shipping: bool,
    pub taxable: bool,
    /// Unit unknown: Liquid documents grams. `None` where the variant is unweighed (286 of the
    /// 1,422 saved variants).
    pub weight: Option<i64>,
    /// `featured_image.src`. `None` where the variant has no image of its own, as on every
    /// saved variant.
    pub image: Option<String>,
    // Commented out: `shopify` on every variant; stock is tracked, but no quantity is rendered.
    // pub inventory_management: Option<String>,
    // Commented out: one unit, no maximum, in steps of one, on every variant.
    // pub quantity_rule: QuantityRule,
    // Commented out: the pre-order plan's price, equal to `price` wherever it is rendered.
    // pub selling_plan_allocations: Vec<SellingPlanAllocation>,
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
            gtin: gtin(raw.barcode),
            price: unset_price(raw.price).map(|cents| money_from_shopify_cents(cents, currency)),
            compare_at_price: raw
                .compare_at_price
                .and_then(unset_price)
                .map(|cents| money_from_shopify_cents(cents, currency)),
            available: raw.available,
            requires_shipping: raw.requires_shipping,
            taxable: raw.taxable,
            weight: unweighed(raw.weight),
            image: raw
                .featured_image
                .map(|image| https_from_protocol_relative(image.src)),
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

// Shopify renders an unset amount as 0, which is no price to sell at.
fn unset_price(cents: i64) -> Option<i64> {
    (cents != 0).then_some(cents)
}

// Shopify defaults an unentered weight to 0: unmeasured, not weightless.
fn unweighed(weight: i64) -> Option<i64> {
    (weight != 0).then_some(weight)
}
