use super::super::raw::{OptionName, sealproduct, variants};
use super::island::Island;
use super::{
    NormalizeError, ORIGIN, https_from_protocol_relative, money_from_shopify_cents, product_gid,
    variant_gid,
};
use money::{Currency, Money};
use serde::{Deserialize, Serialize};

/// What the page states the product in, each read once.
pub(super) struct Sources {
    pub(super) variants: Vec<variants::Variant>,
    pub(super) selected: variants::Variant,
    pub(super) seal: sealproduct::SealProduct,
    pub(super) option_names: Vec<OptionName>,
    pub(super) island: Island,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Product {
    pub id: String,
    /// The last segment of the product path, percent-encoded exactly as the path spells it where
    /// the handle is not ASCII (2 of the 29 saved pages). The pages state no handle of their own.
    pub handle: String,
    /// The handle's page on the storefront's origin.
    pub url: String,
    pub title: String,
    /// The island's description, which is plain text with its padding trimmed. `None` where it
    /// is blank (7 of the 29 saved pages).
    pub description: Option<String>,
    /// The island's `brand`: free text entered per shop, `Minisforum HK`, `Minisforum CN` and
    /// `Minisforum` all appear on the saved pages. The pages state no Shopify vendor.
    pub brand: String,
    /// The island's Google product category. `None` where it is empty (6 of the 29 saved pages).
    pub category: Option<String>,
    // Commented out: the pages state no product type, tags or publish dates.
    // pub product_type: Option<String>,
    // pub tags: Vec<String>,
    // pub published_at: String,
    /// The variant selector's labels without their trailing colon. Empty where the product has
    /// no options to choose (3 of the 29 saved pages).
    pub options: Vec<String>,
    /// The product's primary image without the display width the theme asks the CDN for. The
    /// pages render no gallery in an island, so this is the only image of the product.
    pub images: Vec<String>,
    // Commented out: the island's review summary, present on 11 of the 29 saved pages.
    // pub aggregate_rating: Option<AggregateRating>,
    // Commented out: the variant the page opens on, which is the first available one; it is
    // checked against the list and adds nothing to it.
    // pub selected_variant_id: String,
    pub variants: Vec<Variant>,
}

impl Product {
    pub(super) fn from_raw(
        sources: Sources,
        currency: Currency,
    ) -> Result<Product, NormalizeError> {
        let Sources {
            variants: records,
            selected,
            seal,
            option_names,
            island,
        } = sources;

        let handle = island
            .path
            .strip_prefix("/products/")
            .ok_or_else(|| NormalizeError::ProductPathNotUnderProducts {
                path: island.path.clone(),
            })?
            .to_string();
        let url = format!("{ORIGIN}{}", island.path);

        check_product(&island, &seal)?;
        check_selected(&selected, &records, &island)?;
        check_offers(&island, &records, currency)?;

        let options: Vec<String> = option_names
            .into_iter()
            .map(|label| option_name(&label.text))
            .collect();
        let variants = records
            .into_iter()
            .map(|record| Variant::from_raw(record, &options, currency))
            .collect::<Result<_, _>>()?;

        Ok(Product {
            id: product_gid(seal.id),
            handle,
            url,
            title: seal.title,
            description: non_empty(island.description.trim().to_string()),
            brand: island.brand,
            category: non_empty(island.category),
            options,
            images: vec![without_display_width(&island.image)],
            variants,
        })
    }
}

// The island and `seal_product` are rendered from the same Liquid product, so any disagreement
// means one of the two was misread.
fn check_product(island: &Island, seal: &sealproduct::SealProduct) -> Result<(), NormalizeError> {
    if island.product_id != seal.id {
        return Err(NormalizeError::ProductIdMismatch {
            island: island.product_id,
            seal: seal.id,
        });
    }
    if island.name != seal.title {
        return Err(NormalizeError::ProductTitleMismatch {
            island: island.name.clone(),
            seal: seal.title.clone(),
        });
    }

    // `seal_product` asks the CDN for a 300 pixel rendition, the island for a 1024 pixel one.
    let seal_image = without_display_width(&seal.image_url);
    let island_image = without_display_width(&island.image);
    if seal_image != island_image {
        return Err(NormalizeError::ProductImageMismatch {
            island: island_image,
            seal: seal_image,
        });
    }

    Ok(())
}

// The selected variant is one of the list's and is the same record; the island's product-level SKU
// is its SKU.
fn check_selected(
    selected: &variants::Variant,
    records: &[variants::Variant],
    island: &Island,
) -> Result<(), NormalizeError> {
    let Some(listed) = records.iter().find(|record| record.id == selected.id) else {
        return Err(NormalizeError::UnknownSelectedVariant {
            variant_id: selected.id,
        });
    };
    if listed != selected {
        return Err(NormalizeError::SelectedVariantDiffers {
            variant_id: selected.id,
        });
    }

    let island_sku = non_empty(island.sku.clone());
    let selected_sku = selected.sku.clone().and_then(non_empty);
    if island_sku != selected_sku {
        return Err(NormalizeError::SelectedSkuMismatch {
            island: island_sku,
            selected: selected_sku,
        });
    }

    Ok(())
}

// The offers are rendered from the same Liquid variants, so any disagreement means one of the
// two was misread.
fn check_offers(
    island: &Island,
    records: &[variants::Variant],
    currency: Currency,
) -> Result<(), NormalizeError> {
    let island_ids: Vec<u64> = island.offers.iter().map(|offer| offer.variant_id).collect();
    let record_ids: Vec<u64> = records.iter().map(|record| record.id).collect();
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
                offer: format!("variant {}", offer.variant_id),
                island: offer.currency.clone(),
                shop: currency.as_str().to_string(),
            });
        }

        // Always found: the id sets were compared above.
        let Some(listed) = records.iter().find(|record| record.id == offer.variant_id) else {
            return Err(NormalizeError::VariantIdsMismatch {
                island: island_ids.clone(),
                record: record_ids.clone(),
            });
        };
        let mismatch = |field: &str, island: String, record: String| {
            Err(NormalizeError::VariantFieldMismatch {
                variant_id: listed.id,
                field: field.to_string(),
                island,
                record,
            })
        };

        // The island states major units; the list hundredths. Both become money in the shop
        // currency, which the island's own currency was just checked against.
        let offered = Money::from_raw(offer.price, currency.as_str());
        let listed_price = money_from_shopify_cents(listed.price, currency);
        if offered.as_ref().ok() != Some(&listed_price) {
            return mismatch(
                "price",
                offer.price.to_string(),
                format!("{} hundredths", listed.price),
            );
        }
        if offer.in_stock != listed.available {
            return mismatch(
                "available",
                offer.in_stock.to_string(),
                listed.available.to_string(),
            );
        }
        let record_sku = listed.sku.clone().and_then(non_empty);
        if offer.sku != record_sku {
            return mismatch("sku", format!("{:?}", offer.sku), format!("{record_sku:?}"));
        }
    }

    Ok(())
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Variant {
    pub id: String,
    /// The selected values joined with ` / `. `None` where the product has no options to choose:
    /// Liquid names the only variant `Default Title` (3 of the 29 saved pages).
    pub title: Option<String>,
    // Commented out: `option1` to `option3` and `title` again, one slot per product option.
    // pub option1: Option<String>,
    /// The product title and `title`, joined with ` - `; the product title alone where there is
    /// no choice.
    pub name: String,
    pub sku: Option<String>,
    /// Null on all 78 variants of the saved pages.
    pub barcode: Option<String>,
    /// `None` where the storefront renders HK$0: the product would be announced but not sold. No
    /// saved variant is.
    pub price: Option<Money>,
    /// `None` where the variant is not marked down: no compare-at price, or one that is not above
    /// the price (3 of the 78 saved variants, on 3 pages, where it equals the price). 49 of the
    /// 78 are marked down.
    pub compare_at_price: Option<Money>,
    pub available: bool,
    pub requires_shipping: bool,
    pub taxable: bool,
    /// Unit unknown: Liquid documents grams. `None` where the variant is unweighed (68 of the 78
    /// saved variants).
    pub weight: Option<i64>,
    /// `featured_image.src`. `None` where the variant has no image of its own (76 of the 78 saved
    /// variants).
    pub image: Option<String>,
    // Commented out: the same image as a media entry.
    // pub featured_media: Option<FeaturedMedia>,
    // Commented out: `shopify` on every variant but one, which is not stock-tracked; no quantity
    // is rendered.
    // pub inventory_management: Option<String>,
    // Commented out: one unit, no maximum, in steps of one, on every variant.
    // pub quantity_rule: QuantityRule,
    // Commented out: no selling plans on any page; `requires_selling_plan` is false throughout.
    // pub selling_plan_allocations: Vec<Value>,
    /// One entry per product option, empty where the product has none.
    pub options: Vec<VariantOption>,
}

impl Variant {
    fn from_raw(
        raw: variants::Variant,
        offered: &[String],
        currency: Currency,
    ) -> Result<Variant, NormalizeError> {
        let price = unsold_placeholder_price(raw.price);
        let compare_at_price = raw
            .compare_at_price
            .filter(|compare_at| *compare_at > raw.price)
            .and_then(|compare_at| price.map(|_| compare_at));

        Ok(Variant {
            id: variant_gid(raw.id),
            title: raw.public_title,
            name: raw.name,
            sku: raw.sku.and_then(non_empty),
            barcode: raw.barcode.and_then(non_empty),
            price: price.map(|cents| money_from_shopify_cents(cents, currency)),
            compare_at_price: compare_at_price
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

// The selector names the options only in its labels, and a variant's values only by position. A
// product without options renders no selector, and Liquid gives its only variant the single
// value `Default Title`, which is no choice.
fn variant_options(
    variant_id: u64,
    offered: &[String],
    selected: Vec<String>,
) -> Result<Vec<VariantOption>, NormalizeError> {
    if offered.is_empty() && selected == [DEFAULT_TITLE] {
        return Ok(Vec::new());
    }
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

const DEFAULT_TITLE: &str = "Default Title";

// A label reads `CPU:`.
fn option_name(label: &str) -> String {
    label.trim().trim_end_matches(':').trim_end().to_string()
}

// Images are requested as `https://minisforum.hk/cdn/shop/files/a.png?v=1&width=1024`; the image
// is the file at its version, and the width only how large the page displays it.
fn without_display_width(src: &str) -> String {
    let url = https_from_protocol_relative(src.to_string());

    let Some((path, query)) = url.split_once('?') else {
        return url;
    };
    let kept: Vec<&str> = query
        .split('&')
        .filter(|pair| !pair.starts_with("width="))
        .collect();

    if kept.is_empty() {
        path.to_string()
    } else {
        format!("{path}?{}", kept.join("&"))
    }
}

// HK$0 is not a price the storefront sells at.
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

#[cfg(test)]
mod tests {
    use super::{option_name, without_display_width};

    #[test]
    fn a_label_loses_its_colon() {
        assert_eq!(option_name("處理器:"), "處理器");
        assert_eq!(option_name(" Warranty: "), "Warranty");
        assert_eq!(option_name("CPU"), "CPU");
    }

    #[test]
    fn the_display_width_is_not_part_of_the_image() {
        assert_eq!(
            without_display_width("https://minisforum.hk/cdn/shop/files/a.png?v=1&width=1024"),
            "https://minisforum.hk/cdn/shop/files/a.png?v=1"
        );
        assert_eq!(
            without_display_width("//minisforum.hk/cdn/shop/files/a.png?width=300"),
            "https://minisforum.hk/cdn/shop/files/a.png"
        );
        assert_eq!(
            without_display_width("//minisforum.hk/cdn/shop/files/a.png"),
            "https://minisforum.hk/cdn/shop/files/a.png"
        );
    }
}
