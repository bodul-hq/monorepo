use super::super::raw;
use super::{
    NormalizeError, ORIGIN, money_from_shopify_cents, shop_currency, variant_gid, whole_number,
};
use money::{Currency, Money};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Product {
    /// The analytics `gid`.
    pub id: String,
    pub handle: String,
    /// The product form's path on the storefront's origin.
    pub url: String,
    /// The main section's `h1`.
    pub title: String,
    /// `UGREEN DE` on every page so far.
    pub vendor: String,
    /// `None` where the product is untyped.
    pub product_type: Option<String>,
    // Commented out: false on every page.
    // pub remote: bool,
    // Commented out: the analytics `page` describes this request, not the product: a product
    // page of `resourceId` the product id, under a fresh `requestId`.
    // pub page: Page,
    /// The variant picker's legend titles without their trailing colon. Empty without a picker.
    pub options: Vec<String>,
    /// The main gallery's image slides, in slide order, without the display `width` the theme
    /// asks the CDN for. Video slides are left out: they render only poster thumbnails.
    pub images: Vec<String>,
    // Commented out: the product title or a German caption, nothing a variation needs.
    // pub image_alts: Vec<Option<String>>,
    /// Every variant, in the analytics' order.
    pub variants: Vec<Variant>,
    pub selected_variant: SelectedVariant,
}

impl Product {
    pub(crate) fn from_raw(raw: raw::UgreenDeRawDocument) -> Result<Product, NormalizeError> {
        let currency = shop_currency(&raw.price.currency)?;
        let meta = raw.analytics.meta.product;

        same_product("price block", &raw.price.product_id, meta.id)?;
        same_product("product form", &raw.product_form.product_id, meta.id)?;
        same_url("product form", &raw.product_form.product_url, &meta.handle)?;
        if let Some(picker) = &raw.variant_picker {
            same_product("variant picker", &picker.product_id, meta.id)?;
            same_url("variant picker", &picker.product_url, &meta.handle)?;
        }

        let options: Vec<String> = raw
            .option_names
            .iter()
            .map(|legend| option_name(&legend.text))
            .collect();

        let selected_id: u64 = whole_number(&raw.selected_variant_id.value, "selected_variant_id")?;
        let listed_price = meta
            .variants
            .iter()
            .find(|variant| variant.id == selected_id)
            .map(|variant| variant.price)
            .ok_or(NormalizeError::SelectedVariantNotListed { id: selected_id })?;

        let available = raw.add_to_cart.disabled.is_none();
        let record = match (raw.variant_picker, raw.selected_variant) {
            (Some(_), Some(script)) => Some(script.data),
            (Some(_), None) => return Err(NormalizeError::PickerWithoutSelectedVariant),
            // Without a picker nothing renders the record, so there is nothing to check.
            (None, _) => None,
        };
        if let Some(record) = &record {
            agree("id", record.id == selected_id)?;
            agree("price", record.price == listed_price)?;
            agree("available", record.available == available)?;
        }

        let prices = DisplayedPrices::from_raw(&raw.price, listed_price, currency)?;

        let selected_variant = SelectedVariant {
            id: variant_gid(selected_id),
            available,
            compare_at_price: prices.compare_at_price,
            automatic_discount_price: prices.automatic_discount_price,
            barcode: record
                .as_ref()
                .and_then(|record| record.barcode.clone())
                .and_then(non_empty),
            options: match record {
                Some(record) => variant_options(record.id, &options, record.options)?,
                None => Vec::new(),
            },
        };

        let variants = meta
            .variants
            .into_iter()
            .map(|variant| Variant::from_raw(variant, currency))
            .collect();

        Ok(Product {
            id: meta.gid,
            url: format!("{ORIGIN}{}", raw.product_form.product_url),
            handle: meta.handle,
            title: raw.title.text,
            vendor: meta.vendor,
            product_type: non_empty(meta.product_type),
            options,
            images: raw
                .images
                .into_iter()
                .map(|image| gallery_image(&image.src))
                .collect(),
            variants,
            selected_variant,
        })
    }
}

fn same_product(island: &str, id: &str, analytics: u64) -> Result<(), NormalizeError> {
    if id == analytics.to_string() {
        return Ok(());
    }

    Err(NormalizeError::ProductIdMismatch {
        island: island.to_string(),
        id: id.to_string(),
        analytics,
    })
}

fn same_url(island: &str, url: &str, handle: &str) -> Result<(), NormalizeError> {
    if url == format!("/products/{handle}") {
        return Ok(());
    }

    Err(NormalizeError::ProductUrlMismatch {
        island: island.to_string(),
        url: url.to_string(),
        handle: handle.to_string(),
    })
}

fn agree(field: &str, agrees: bool) -> Result<(), NormalizeError> {
    if agrees {
        return Ok(());
    }

    Err(NormalizeError::SelectedVariantMismatch {
        field: field.to_string(),
    })
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Variant {
    pub id: String,
    /// The selected values joined with ` / `. `None` on a product without options.
    pub title: Option<String>,
    /// The product title and `title`, joined with ` - `, or the product title alone.
    pub name: String,
    pub sku: Option<String>,
    /// Before any automatic discount; see `SelectedVariant::automatic_discount_price`.
    pub price: Money,
}

impl Variant {
    fn from_raw(raw: raw::analytics::Variant, currency: Currency) -> Variant {
        Variant {
            id: variant_gid(raw.id),
            title: raw.public_title,
            name: raw.name,
            sku: raw.sku.and_then(non_empty),
            price: money_from_shopify_cents(raw.price, currency),
        }
    }
}

/// What the page states about the variant it opens on, beyond its price.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SelectedVariant {
    pub id: String,
    /// Whether the add-to-cart button is enabled.
    pub available: bool,
    /// The struck-through price, where the variant is marked down. `None` also where an
    /// automatic discount shows: the theme then strikes through the variant price instead, and
    /// the markdown, if any, is not rendered.
    pub compare_at_price: Option<Money>,
    /// The price after an automatic discount (a storefront app's, not a code), where one applies.
    pub automatic_discount_price: Option<Money>,
    /// From the picker's record; `None` without a picker.
    pub barcode: Option<String>,
    // Commented out: `price` and `public_title` restate the analytics variant; `available`
    // restates the add-to-cart button.
    // Commented out: `requires_shipping` and `taxable` are true on every record.
    // Commented out: the variant's image is one of the gallery's.
    // pub featured_image: Option<FeaturedImage>,
    // pub featured_media: Option<FeaturedMedia>,
    // Commented out: `shopify` on every record; stock is tracked, but no quantity is rendered.
    // pub inventory_management: Option<String>,
    // Commented out: one unit, no maximum, in steps of one, on every record.
    // pub quantity_rule: QuantityRule,
    // Commented out: no selling plans on any page; `requires_selling_plan` is false throughout.
    // pub selling_plan_allocations: Vec<Value>,
    // Commented out: 0, unweighed, on every record.
    // pub weight: i64,
    /// The picker's record's values paired with the legends. Empty without a picker.
    pub options: Vec<VariantOption>,
}

/// The price block's reading of the selected variant's prices against its analytics price.
struct DisplayedPrices {
    compare_at_price: Option<Money>,
    automatic_discount_price: Option<Money>,
}

impl DisplayedPrices {
    // The block displays `base_price_cents` and strikes through `base_compare_cents`. Without an
    // automatic discount these are the variant's price and its compare-at price (equal to the
    // price where it is not marked down). With one, the theme displays the discounted price and
    // strikes through the variant's own price.
    fn from_raw(
        block: &raw::PriceBlock,
        listed: i64,
        currency: Currency,
    ) -> Result<DisplayedPrices, NormalizeError> {
        let displayed: i64 = whole_number(&block.base_price_cents, "base_price_cents")?;
        let compare: i64 = whole_number(&block.base_compare_cents, "base_compare_cents")?;

        if displayed > listed {
            return Err(NormalizeError::DisplayedPriceAboveListed { displayed, listed });
        }

        if displayed < listed {
            if compare != listed {
                return Err(NormalizeError::DiscountedFromAnotherPrice {
                    displayed,
                    listed,
                    compare,
                });
            }

            return Ok(DisplayedPrices {
                compare_at_price: None,
                automatic_discount_price: Some(money_from_shopify_cents(displayed, currency)),
            });
        }

        Ok(DisplayedPrices {
            compare_at_price: (compare > listed)
                .then(|| money_from_shopify_cents(compare, currency)),
            automatic_discount_price: None,
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

// A legend title reads `Länge:`.
fn option_name(legend: &str) -> String {
    legend.trim().trim_end_matches(':').trim_end().to_string()
}

// The gallery asks the CDN for `//de.ugreen.com/cdn/shop/files/a.png?v=1&width=3840`; the image
// is the file at its version, and the width only how large the theme displays it.
fn gallery_image(src: &str) -> String {
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

fn non_empty(value: String) -> Option<String> {
    (!value.is_empty()).then_some(value)
}

#[cfg(test)]
mod tests {
    use super::{gallery_image, option_name};

    #[test]
    fn a_legend_loses_its_colon() {
        assert_eq!(option_name("Länge:"), "Länge");
        assert_eq!(option_name(" Farbe : "), "Farbe");
    }

    #[test]
    fn only_the_width_parameter_is_dropped() {
        assert_eq!(
            gallery_image("//de.ugreen.com/cdn/shop/files/a.png?v=1&width=3840"),
            "https://de.ugreen.com/cdn/shop/files/a.png?v=1"
        );
        assert_eq!(
            gallery_image("https://de.ugreen.com/a.png?width=3840"),
            "https://de.ugreen.com/a.png"
        );
        assert_eq!(
            gallery_image("https://de.ugreen.com/a.png"),
            "https://de.ugreen.com/a.png"
        );
    }
}
