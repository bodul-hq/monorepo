use super::super::raw::productjson;
use super::island::Island;
use super::{
    NormalizeError, ORIGIN, https_from_protocol_relative, minor_units_from_hundredths, non_empty,
    percent_encoded, product_gid, variant_gid,
};
use money::{Currency, Money};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Product {
    pub id: String,
    /// As the record states it: the 2 saved handles with Japanese characters keep them.
    pub handle: String,
    /// The handle's page on the storefront's origin, the handle percent-encoded as in the
    /// island's URL.
    pub url: String,
    pub title: String,
    /// HTML. `None` where the storefront leaves it empty (23 of the 73 saved pages).
    pub description: Option<String>,
    // Commented out: equal to `description` on every page.
    // pub content: String,
    /// Verbatim RFC 3339 instants in the shop's offset.
    pub published_at: String,
    pub created_at: String,
    /// Free text entered per product: `Minisforum JP` on 72 of the 73 saved pages, `xcotton`
    /// (the shipping-protection app's product) on the other.
    pub vendor: String,
    /// `None` where the product is untyped (1 of the 73 saved pages). Free text, Japanese or
    /// English: `Mini PC`, `gift`, `adapter`, ...
    pub product_type: Option<String>,
    /// The island's product category, free text such as `デスクトップPC` or `gift`. `None` where
    /// the page renders no product island (2 of the 73 saved pages) or the island's is empty
    /// (the 1 single-`Product` island).
    pub category: Option<String>,
    /// Merchandising labels, among them product series and sale campaigns. Empty on 3 of the 73
    /// saved pages.
    pub tags: Vec<String>,
    // Commented out: `price`, `price_min`, `price_max` and `price_varies` restate the variants'
    // prices; the `compare_at_price` family restates their `compare_at_price`s.
    // pub price: i64,
    // Commented out: true exactly where some variant's `available` is.
    // pub available: bool,
    pub options: Vec<String>,
    /// The product's images only, from the record's `images`: the same as the `image` entries of
    /// `media` on all 73 saved pages.
    pub images: Vec<String>,
    // Commented out: the first of `images` on every page.
    // pub featured_image: Option<String>,
    // Commented out: the images again with dimensions, and the product's videos (21 YouTube
    // embeds on 14 pages, 2 hosted videos on 2) that nothing needs.
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
        let url = format!("{ORIGIN}/products/{}", percent_encoded(&raw.handle));

        let category = match island {
            Some(island) => {
                check_island(&island, &raw, &url, currency)?;
                island.category
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

fn price_field(variant_id: u64) -> String {
    format!("variant {variant_id} price")
}

fn compare_at_price_field(variant_id: u64) -> String {
    format!("variant {variant_id} compare_at_price")
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

        // Both sides in minor units of the shop currency: whole yen for the island's `"130399"`,
        // and the record's `13039900` hundredths divided down.
        let island_price = offer.minor_units(currency)?;
        let record_price =
            minor_units_from_hundredths(variant.price, currency, &price_field(variant.id))?;
        if island_price != record_price {
            return mismatch("price", offer.price.clone(), record_price.to_string());
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
    /// `None` on 13 of the 277 saved variants. Not unique: the 98 variants of
    /// `products-shipping-protection.html` share one.
    pub sku: Option<String>,
    /// Null on all 277 variants of the saved pages.
    pub barcode: Option<String>,
    /// `None` where the storefront renders ¥0: the product is announced but not sold (1 of the
    /// 277 saved variants). In whole yen.
    pub price: Option<Money>,
    /// `None` where the storefront states none or renders ¥0 (155 of the 277 saved variants), in
    /// whole yen. Where it is stated it is not always above `price`: 16 variants carry one equal
    /// to it, 1 (`products-knife-and-fork-set.html`) one below, and all are kept as stated.
    pub compare_at_price: Option<Money>,
    pub available: bool,
    pub requires_shipping: bool,
    pub taxable: bool,
    /// Unit unknown: Liquid documents grams. `None` where the variant is unweighed (266 of the
    /// 277 saved variants).
    pub weight: Option<i64>,
    /// `featured_image.src`. `None` where the variant has no image of its own.
    pub image: Option<String>,
    // Commented out: the same image as a media entry.
    // pub featured_media: Option<FeaturedMedia>,
    // Commented out: `shopify` or null; stock is tracked on some variants, but no quantity is
    // rendered.
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
        let price = minor_units_from_hundredths(raw.price, currency, &price_field(raw.id))?;
        let compare_at_price = raw
            .compare_at_price
            .map(|hundredths| {
                minor_units_from_hundredths(hundredths, currency, &compare_at_price_field(raw.id))
            })
            .transpose()?;

        Ok(Variant {
            id: variant_gid(raw.id),
            title: raw.title,
            // public_title: raw.public_title,
            // option1: raw.option1,
            name: raw.name,
            sku: raw.sku.and_then(non_empty),
            barcode: raw.barcode.and_then(non_empty),
            price: unsold_placeholder_price(price).map(|amount| Money::new(amount, currency)),
            compare_at_price: compare_at_price
                .and_then(unsold_placeholder_price)
                .map(|amount| Money::new(amount, currency)),
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

// ¥0 is not a price the storefront sells at; it renders it on a product it only announces, and
// leaves that product's schema.org island out.
fn unsold_placeholder_price(amount: i64) -> Option<i64> {
    (amount != 0).then_some(amount)
}

// Shopify defaults an unentered weight to 0: unmeasured, not weightless.
fn unweighed(weight: i64) -> Option<i64> {
    (weight != 0).then_some(weight)
}
