use super::super::raw;
use super::super::raw::ldjsonitem::{LdJsonItem, Offer};
use super::{
    NormalizeError, ORIGIN, currency, money_from_offer, money_from_shopify_cents, variant_gid,
};
use money::{Currency, Money};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Product {
    /// The analytics `gid`.
    pub id: String,
    pub handle: String,
    /// The product island's `url`, percent-encoded where the handle is not ASCII. `None` on a
    /// landing page, which renders no product island.
    pub url: Option<String>,
    /// The product island's `name`. `None` on a landing page: its `h1`s are empty, and the page
    /// names the product only inside the variant `name`s.
    pub title: Option<String>,
    /// The product island's `description`; `None` where it is empty or there is no island.
    pub description: Option<String>,
    /// The product island's Shopify taxonomy `category`; `None` where it is empty or there is no
    /// island.
    pub category: Option<String>,
    // Commented out: `Minisforum KR` on every page, as is the island's `brand`.
    // pub vendor_brand: String,
    pub vendor: String,
    /// `None` where the product is untyped.
    pub product_type: Option<String>,
    // Commented out: false on every page.
    // pub remote: bool,
    // Commented out: the analytics `page` describes this request, not the product.
    // pub page: Page,
    // Commented out: the shop itself, the same on every page.
    // pub organization: Organization,
    /// The variant picker's legends without their trailing colon. Empty where there is no picker.
    pub options: Vec<String>,
    /// The main gallery, in slide order, without the display `width` the theme asks the CDN for.
    pub images: Vec<String>,
    pub variants: Vec<Variant>,
}

impl Product {
    pub(crate) fn from_raw(raw: raw::MinisforumKrRawDocument) -> Result<Product, NormalizeError> {
        let meta = raw.analytics.meta.product;
        let island = ProductIsland::from_raw(raw.ld_json)?;

        if let Some(picker) = &raw.variant_picker {
            if picker.product_id != meta.id.to_string() {
                return Err(NormalizeError::ProductIdMismatch {
                    island: "variant picker".to_string(),
                    other: picker.product_id.clone(),
                    analytics: meta.id,
                });
            }
            if let Some(island) = &island
                && format!("{ORIGIN}{}", picker.product_url) != island.url
            {
                return Err(NormalizeError::ProductUrlMismatch {
                    picker: picker.product_url.clone(),
                    island: island.url.clone(),
                });
            }
        }
        if let Some(group_id) = island.as_ref().and_then(|island| island.group_id.as_ref())
            && *group_id != meta.id.to_string()
        {
            return Err(NormalizeError::ProductIdMismatch {
                island: "product island".to_string(),
                other: group_id.clone(),
                analytics: meta.id,
            });
        }

        let options: Vec<String> = raw
            .option_names
            .into_iter()
            .map(|legend| option_name(&legend.text))
            .collect();

        let selected = raw.selected_variant.map(|script| script.data);
        if let Some(selected) = &selected
            && !meta
                .variants
                .iter()
                .any(|variant| variant.id == selected.id)
        {
            return Err(NormalizeError::UnknownSelectedVariant {
                variant_id: selected.id,
            });
        }

        let offers = match &island {
            Some(island) => {
                let analytics: Vec<u64> = meta.variants.iter().map(|variant| variant.id).collect();
                let listed: Vec<u64> = island.variants.iter().map(|offer| offer.id).collect();
                if analytics != listed {
                    return Err(NormalizeError::VariantIdsMismatch {
                        analytics,
                        island: listed,
                    });
                }
                island.variants.iter().map(Some).collect()
            }
            None => vec![None; meta.variants.len()],
        };
        let currency = island.as_ref().map(|island| island.currency);

        let variants = meta
            .variants
            .into_iter()
            .zip(offers)
            .map(|(variant, offer)| {
                let selected = selected.as_ref().filter(|record| record.id == variant.id);
                Variant::from_raw(variant, offer, selected, &options, currency)
            })
            .collect::<Result<_, _>>()?;

        Ok(Product {
            id: meta.gid,
            handle: meta.handle,
            url: island.as_ref().map(|island| island.url.clone()),
            title: island.as_ref().map(|island| island.name.clone()),
            description: island
                .as_ref()
                .and_then(|island| non_empty(island.description.clone())),
            category: island
                .as_ref()
                .and_then(|island| non_empty(island.category.clone())),
            vendor: meta.vendor,
            product_type: non_empty(meta.product_type),
            options,
            images: raw
                .images
                .into_iter()
                .map(|image| without_display_width(&image.src))
                .collect(),
            variants,
        })
    }
}

/// The schema.org product island, a `ProductGroup` and a single `Product` read alike.
struct ProductIsland {
    group_id: Option<String>,
    url: String,
    name: String,
    description: String,
    category: String,
    currency: Currency,
    variants: Vec<IslandVariant>,
}

struct IslandVariant {
    id: u64,
    image: String,
    sku: Option<String>,
    offer: Offer,
}

impl ProductIsland {
    fn from_raw(
        scripts: Vec<raw::ldjsonitem::LdJsonScript>,
    ) -> Result<Option<Self>, NormalizeError> {
        let mut islands: Vec<ProductIsland> = scripts
            .into_iter()
            .filter_map(|script| match script.data {
                LdJsonItem::Organization(_) => None,
                LdJsonItem::ProductGroup(group) => Some(ProductIsland::from_group(group)),
                LdJsonItem::Product(product) => Some(ProductIsland::from_product(product)),
            })
            .collect::<Result<_, _>>()?;

        match islands.len() {
            0 => Ok(None),
            1 => Ok(islands.pop()),
            count => Err(NormalizeError::MultipleProductIslands { count }),
        }
    }

    fn from_group(group: raw::ldjsonitem::ProductGroup) -> Result<ProductIsland, NormalizeError> {
        let variants = group
            .has_variant
            .into_iter()
            .map(|variant| {
                Ok(IslandVariant {
                    id: offer_variant_id(&variant.offers.url)?,
                    image: variant.image,
                    sku: variant.sku,
                    offer: variant.offers,
                })
            })
            .collect::<Result<Vec<_>, NormalizeError>>()?;

        Ok(ProductIsland {
            group_id: Some(group.product_group_id),
            url: group.url,
            name: group.name,
            description: group.description,
            category: group.category,
            currency: offers_currency(&variants)?,
            variants,
        })
    }

    fn from_product(product: raw::ldjsonitem::Product) -> Result<ProductIsland, NormalizeError> {
        let variants = vec![IslandVariant {
            id: offer_variant_id(&product.offers.url)?,
            image: product.image,
            sku: product.sku,
            offer: product.offers,
        }];

        Ok(ProductIsland {
            group_id: None,
            url: product.url,
            name: product.name,
            description: product.description,
            category: product.category,
            currency: offers_currency(&variants)?,
            variants,
        })
    }
}

// The island names no currency of its own; every offer names one, and they must agree.
fn offers_currency(variants: &[IslandVariant]) -> Result<Currency, NormalizeError> {
    let mut currencies: Vec<String> = variants
        .iter()
        .map(|variant| variant.offer.price_currency.clone())
        .collect();
    currencies.dedup();

    match currencies.as_slice() {
        [code] => currency(code),
        _ => Err(NormalizeError::MixedOfferCurrencies { currencies }),
    }
}

// An offer URL reads `https://www.minisforum.kr/products/<handle>?variant=<id>`.
fn offer_variant_id(url: &str) -> Result<u64, NormalizeError> {
    url.split_once('?')
        .into_iter()
        .flat_map(|(_, query)| query.split('&'))
        .find_map(|pair| pair.strip_prefix("variant="))
        .and_then(|id| id.parse().ok())
        .ok_or_else(|| NormalizeError::OfferUrlWithoutVariant {
            url: url.to_string(),
        })
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Variant {
    pub id: String,
    /// The selected values joined with ` / `. `None` where the product has no options.
    pub title: Option<String>,
    /// The product title and `title`, joined with ` - `.
    pub name: String,
    pub sku: Option<String>,
    /// `None` where the analytics render ₩0: the variant is not sold, as on landing pages.
    pub price: Option<Money>,
    /// The island offer's availability. `None` where there is no island.
    pub available: Option<bool>,
    /// The island offer's URL, which selects the variant. `None` where there is no island.
    pub url: Option<String>,
    /// The island's image for the variant, without its display `width`. `None` where there is
    /// no island.
    pub image: Option<String>,
    // Commented out: the island's variant `name`, equal to `name` on every page.
    // pub island_name: String,
    /// What the variant picker's record adds, for the one variant the page opens on. `None` on
    /// every other variant: the page renders no record of them.
    pub selected: Option<SelectedVariant>,
}

impl Variant {
    fn from_raw(
        raw: raw::analytics::Variant,
        island: Option<&IslandVariant>,
        selected: Option<&raw::variant::Variant>,
        offered: &[String],
        currency: Option<Currency>,
    ) -> Result<Variant, NormalizeError> {
        let price = match (unsold_placeholder_price(raw.price), currency) {
            (None, _) => None,
            (Some(cents), Some(currency)) => {
                Some(money_from_shopify_cents(cents, currency, "variants.price")?)
            }
            (Some(_), None) => {
                return Err(NormalizeError::MissingCurrency { variant_id: raw.id });
            }
        };

        let sku = raw.sku.and_then(non_empty);
        let (available, url, image) = match island {
            Some(island) => {
                check_island_agrees(raw.id, raw.price, price.as_ref(), &sku, island)?;
                (
                    Some(availability(&island.offer.availability)?),
                    Some(island.offer.url.clone()),
                    Some(without_display_width(&island.image)),
                )
            }
            None => (None, None, None),
        };

        let selected = selected
            .map(|record| SelectedVariant::from_raw(record, raw.price, offered, currency))
            .transpose()?;

        Ok(Variant {
            id: variant_gid(raw.id),
            title: raw.public_title,
            name: raw.name,
            sku,
            price,
            available,
            url,
            image,
            selected,
        })
    }
}

fn check_island_agrees(
    variant_id: u64,
    cents: i64,
    price: Option<&Money>,
    sku: &Option<String>,
    island: &IslandVariant,
) -> Result<(), NormalizeError> {
    let offered = money_from_offer(&island.offer.price, currency(&island.offer.price_currency)?)?;
    let agrees = match price {
        Some(price) => *price == offered,
        None => offered.amount() == 0,
    };
    if !agrees {
        return Err(NormalizeError::PriceMismatch {
            variant_id,
            analytics: cents,
            other: format!("the island offers it at `{}`", island.offer.price),
        });
    }

    let island_sku = island.sku.clone().and_then(non_empty);
    if island_sku.is_some() && island_sku != *sku {
        return Err(NormalizeError::SkuMismatch {
            variant_id,
            analytics: sku.clone(),
            island: island_sku,
        });
    }

    Ok(())
}

// Only the two values the corpus shows are read; anything else is a fact to learn first.
fn availability(value: &str) -> Result<bool, NormalizeError> {
    match value {
        "http://schema.org/InStock" => Ok(true),
        "http://schema.org/OutOfStock" => Ok(false),
        other => Err(NormalizeError::UnknownAvailability {
            value: other.to_string(),
        }),
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SelectedVariant {
    /// `None` where the variant is not marked down.
    pub compare_at_price: Option<Money>,
    /// Null on every page so far.
    pub barcode: Option<String>,
    pub requires_shipping: bool,
    pub taxable: bool,
    /// Unit unknown: Liquid documents grams. `None` where the variant is unweighed.
    pub weight: Option<i64>,
    pub options: Vec<VariantOption>,
    // Commented out: true on every page, and the island's availability covers every variant.
    // pub available: bool,
    // Commented out: `title`, `name`, `sku` and `price` again, as the analytics state them.
    // pub title: String,
    // Commented out: `options` again, one slot per product option.
    // pub option1: Option<String>,
    // Commented out: the island already gives each variant an image.
    // pub featured_image: Option<FeaturedImage>,
    // pub featured_media: Option<FeaturedMedia>,
    // Commented out: `shopify` on every variant; stock is tracked, but no quantity is rendered.
    // pub inventory_management: Option<String>,
    // Commented out: one unit, no maximum, in steps of one, on every variant.
    // pub quantity_rule: QuantityRule,
    // Commented out: no selling plans on any page; `requires_selling_plan` is false throughout.
    // pub selling_plan_allocations: Vec<Value>,
}

impl SelectedVariant {
    fn from_raw(
        raw: &raw::variant::Variant,
        analytics_cents: i64,
        offered: &[String],
        currency: Option<Currency>,
    ) -> Result<SelectedVariant, NormalizeError> {
        if raw.price != analytics_cents {
            return Err(NormalizeError::PriceMismatch {
                variant_id: raw.id,
                analytics: analytics_cents,
                other: format!("the picker's record states {} hundredths", raw.price),
            });
        }

        let compare_at_price = match (raw.compare_at_price, currency) {
            (None, _) => None,
            (Some(cents), Some(currency)) => Some(money_from_shopify_cents(
                cents,
                currency,
                "selected_variant.compare_at_price",
            )?),
            (Some(_), None) => {
                return Err(NormalizeError::MissingCurrency { variant_id: raw.id });
            }
        };

        Ok(SelectedVariant {
            compare_at_price,
            barcode: raw.barcode.clone().and_then(non_empty),
            requires_shipping: raw.requires_shipping,
            taxable: raw.taxable,
            weight: unweighed(raw.weight),
            options: variant_options(raw.id, offered, &raw.options)?,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VariantOption {
    pub name: String,
    pub value: String,
}

// The picker names the options only in its legends, and a variant's values only by position.
fn variant_options(
    variant_id: u64,
    offered: &[String],
    selected: &[String],
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
            value: value.clone(),
        })
        .collect())
}

// A legend reads `CPU:`.
fn option_name(legend: &str) -> String {
    legend.trim().trim_end_matches(':').trim_end().to_string()
}

// Images are requested as `//www.minisforum.kr/cdn/shop/files/a.png?v=1&width=1946`; the image is
// the file at its version, and the width only how large the page displays it.
fn without_display_width(src: &str) -> String {
    let url = match src.strip_prefix("//") {
        Some(rest) => format!("https://{rest}"),
        None => src.to_string(),
    };

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

// ₩0 is not a price the storefront sells at; it renders it on products it does not sell yet.
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
    use super::{offer_variant_id, option_name, without_display_width};

    #[test]
    fn a_legend_loses_its_colon() {
        assert_eq!(option_name("CPU&GPU:"), "CPU&GPU");
        assert_eq!(option_name(" 사양 : "), "사양");
    }

    #[test]
    fn only_the_width_parameter_is_dropped() {
        assert_eq!(
            without_display_width("//www.minisforum.kr/cdn/shop/files/a.png?v=1&width=1946"),
            "https://www.minisforum.kr/cdn/shop/files/a.png?v=1"
        );
        assert_eq!(
            without_display_width("https://www.minisforum.kr/a.png?width=1920"),
            "https://www.minisforum.kr/a.png"
        );
    }

    #[test]
    fn the_variant_id_comes_from_the_offer_url() {
        assert_eq!(
            offer_variant_id("https://www.minisforum.kr/products/a?variant=41622560604206").ok(),
            Some(41622560604206)
        );
        assert!(offer_variant_id("https://www.minisforum.kr/products/a").is_err());
    }
}
