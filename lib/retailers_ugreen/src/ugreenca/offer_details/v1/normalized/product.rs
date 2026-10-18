use super::super::raw;
use super::{
    NormalizeError, https_from_protocol_relative, money_from_shopify_cents, non_empty,
    shop_currency, variant_gid,
};
use money::{Currency, Money};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Product {
    /// The analytics `gid`.
    pub id: String,
    pub handle: String,
    /// The `Product` island's URL, in the page's language: French pages sit under `/fr/`.
    pub url: String,
    /// The page's `h1`, in the page's language.
    pub title: String,
    // Commented out: the title again, on some pages with a trailing space.
    // pub name: String,
    /// The `Product` island's plain-text summary, not the HTML product description.
    pub description: String,
    /// Verbatim; it varies between products (`UGREEN`, `UGREEN STORE`, `UGREEN CA`, `Seel`).
    pub vendor: String,
    // Commented out: equal to `vendor` on every page.
    // pub brand: String,
    /// `None` where the product is untyped.
    pub product_type: Option<String>,
    // Commented out: the island's one image, the first of `images` at `&width=600`.
    // pub image: Vec<String>,
    // Commented out: the island's offers restate the variants' SKUs, prices and stock, but only
    // for the selected variant on some multi-variant pages. Only their currency is read, to check
    // it against the shop's.
    // pub offers: Vec<Offer>,
    // Commented out: false on every page.
    // pub remote: bool,
    // Commented out: the analytics `page` describes this request, not the product.
    // pub page: Page,
    // Commented out: the featured attribute's theme settings - sticky bars, swatches, badges.
    // pub featured: Featured,
    /// The picker's option names, in position order and in the page's language; empty on a
    /// `Default Title` product.
    pub options: Vec<String>,
    /// The main gallery's images, in slide order.
    pub images: Vec<String>,
    pub variants: Vec<Variant>,
}

impl Product {
    pub(crate) fn from_raw(raw: raw::UgreenCaRawDocument) -> Result<Product, NormalizeError> {
        let shop_currency_code = raw.shop.data.currency_code;
        let currency = shop_currency(&shop_currency_code)?;
        let meta = raw.analytics.meta.product;
        let featured = raw.featured.data;

        let island = raw
            .ld_json
            .into_iter()
            .find_map(|item| match item.data {
                raw::ldjsonitem::LdJson::Product(product) => Some(*product),
                _ => None,
            })
            .ok_or(NormalizeError::MissingProductIsland)?;

        let product_id = meta.id.to_string();
        if featured.id != product_id || island.product_id != product_id {
            return Err(NormalizeError::ProductIdMismatch {
                analytics: meta.id,
                featured: featured.id,
                island: island.product_id,
            });
        }
        if !island.url.ends_with(&format!("/products/{}", meta.handle)) {
            return Err(NormalizeError::ProductUrlMismatch {
                url: island.url,
                handle: meta.handle,
            });
        }

        let offers = island.offers.into_vec();
        if let Some(offer) = offers
            .iter()
            .find(|offer| offer.price_currency != shop_currency_code)
        {
            return Err(NormalizeError::OfferCurrencyMismatch {
                offer: offer.price_currency.clone(),
                shop: shop_currency_code,
            });
        }

        let (options, variants) = match (raw.variants, raw.options) {
            (Some(variants), Some(options)) => {
                let options = option_names(options.data)?;

                let analytics: Vec<u64> = meta.variants.iter().map(|variant| variant.id).collect();
                let picker: Vec<u64> = variants.data.iter().map(|variant| variant.id).collect();
                if analytics != picker {
                    return Err(NormalizeError::VariantIdsMismatch { analytics, picker });
                }

                let variants = variants
                    .data
                    .into_iter()
                    .map(|record| Variant::from_record(record, &options, currency))
                    .collect::<Result<_, _>>()?;
                (options, variants)
            }
            (Some(_), None) => {
                return Err(NormalizeError::IncompletePicker {
                    present: "variants",
                    missing: "options",
                });
            }
            (None, Some(_)) => {
                return Err(NormalizeError::IncompletePicker {
                    present: "options",
                    missing: "variants",
                });
            }
            (None, None) => {
                let variant =
                    Variant::from_default_title(meta.variants, &featured, island.mpn, currency)?;
                (Vec::new(), vec![variant])
            }
        };

        Ok(Product {
            id: meta.gid,
            handle: meta.handle,
            url: island.url,
            title: raw.title.text,
            description: island.description,
            vendor: meta.vendor,
            product_type: non_empty(meta.product_type),
            options,
            images: raw
                .media
                .into_iter()
                .filter(|media| media.media_type == "image")
                .map(|media| https_from_protocol_relative(&media.src))
                .collect(),
            variants,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Variant {
    pub id: String,
    /// The selected values joined with ` / `; `None` on a `Default Title` variant.
    pub title: Option<String>,
    // Commented out: `title` again, or `Default Title`.
    // pub public_title: Option<String>,
    // Commented out: `options` again, one slot per product option.
    // pub option1: Option<String>,
    /// The product title, joined with ` - ` to `title` where there is one.
    pub name: String,
    /// Sometimes a bundle of SKUs joined with `+`, such as `40737+10232`.
    pub sku: Option<String>,
    pub barcode: Option<String>,
    pub price: Money,
    /// `None` where the variant is not marked down.
    pub compare_at_price: Option<Money>,
    pub available: bool,
    /// `None` on a `Default Title` variant, which renders no record to state it.
    pub requires_shipping: Option<bool>,
    // Commented out: true on every variant.
    // pub taxable: bool,
    /// Unit unknown: Liquid documents grams. `None` where the variant is unweighed or renders no
    /// record.
    pub weight: Option<i64>,
    /// `featured_image.src`. `None` where the variant has no image of its own.
    pub image: Option<String>,
    // Commented out: the same image as a media entry.
    // pub featured_media: Option<FeaturedMedia>,
    // Commented out: `shopify` or null; stock may be tracked, but no quantity is rendered.
    // pub inventory_management: Option<String>,
    // Commented out: one unit, no maximum, in steps of one, on every variant.
    // pub quantity_rule: QuantityRule,
    // Commented out: no selling plans on any page; `requires_selling_plan` is false throughout.
    // pub selling_plan_allocations: Vec<Value>,
    pub options: Vec<VariantOption>,
}

impl Variant {
    fn from_record(
        raw: raw::variant::Variant,
        offered: &[String],
        currency: Currency,
    ) -> Result<Variant, NormalizeError> {
        Ok(Variant {
            id: variant_gid(raw.id),
            title: Some(raw.title),
            // public_title: raw.public_title,
            // option1: raw.option1,
            name: raw.name,
            sku: raw.sku.and_then(non_empty),
            barcode: raw.barcode.and_then(non_empty),
            price: money_from_shopify_cents(raw.price, currency),
            compare_at_price: raw
                .compare_at_price
                .map(|cents| money_from_shopify_cents(cents, currency)),
            available: raw.available,
            requires_shipping: Some(raw.requires_shipping),
            // taxable: raw.taxable,
            weight: unweighed(raw.weight),
            image: raw
                .featured_image
                .map(|image| https_from_protocol_relative(&image.src)),
            // featured_media: raw.featured_media,
            // inventory_management: raw.inventory_management,
            // quantity_rule: raw.quantity_rule,
            // selling_plan_allocations: raw.selling_plan_allocations,
            options: variant_options(raw.id, offered, raw.options)?,
        })
    }

    // A `Default Title` product renders no variant record: the analytics name its one variant,
    // the featured attribute states its price and stock, and the `Product` island its barcode.
    fn from_default_title(
        analytics: Vec<raw::analytics::Variant>,
        featured: &raw::featured::Featured,
        barcode: String,
        currency: Currency,
    ) -> Result<Variant, NormalizeError> {
        let count = analytics.len();
        let [variant]: [raw::analytics::Variant; 1] = analytics
            .try_into()
            .map_err(|_| NormalizeError::NoVariantRecords { count })?;

        if variant.price != featured.price {
            return Err(NormalizeError::FeaturedPriceMismatch {
                featured: featured.price,
                analytics: variant.price,
            });
        }

        Ok(Variant {
            id: variant_gid(variant.id),
            title: variant.public_title,
            name: variant.name,
            sku: variant.sku.and_then(non_empty),
            barcode: non_empty(barcode),
            price: money_from_shopify_cents(variant.price, currency),
            compare_at_price: featured
                .compare_at_price
                .map(|cents| money_from_shopify_cents(cents, currency)),
            available: featured.available,
            requires_shipping: None,
            weight: None,
            image: None,
            options: Vec::new(),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VariantOption {
    pub name: String,
    pub value: String,
}

// The option island numbers its entries from one; a variant states its values only by position.
fn option_names(options: Vec<raw::variant::ProductOption>) -> Result<Vec<String>, NormalizeError> {
    let mut options = options;
    options.sort_by_key(|option| option.position);

    let positions: Vec<usize> = options.iter().map(|option| option.position).collect();
    if !positions.iter().copied().eq(1..=positions.len()) {
        return Err(NormalizeError::OptionPositionsOutOfOrder { positions });
    }

    Ok(options.into_iter().map(|option| option.name).collect())
}

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

// Shopify defaults an unentered weight to 0: unmeasured, not weightless.
fn unweighed(weight: i64) -> Option<i64> {
    (weight != 0).then_some(weight)
}
