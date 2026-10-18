use super::super::raw;
use super::{NormalizeError, ORIGIN, money_from_shopify_cents, shop_currency, variant_gid};
use money::{Currency, Money};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Product {
    /// The analytics `gid`.
    pub id: String,
    pub handle: String,
    /// The variant picker's product path on the storefront's origin.
    pub url: String,
    /// The page's `h1`. Nothing else on the page states the title on its own: the variant `name`s
    /// prefix it to the selected values.
    pub title: String,
    /// Verbatim; its casing varies between products (`Minisforum RU`, `MINISFORUM RU`).
    pub vendor: String,
    /// `None` where the product is untyped.
    pub product_type: Option<String>,
    // Commented out: false on every page.
    // pub remote: bool,
    // Commented out: the analytics `page` describes this request, not the product: a product
    // page of `resourceId` the product id, under a fresh `requestId`.
    // pub page: Page,
    // Commented out: the shop's Apple Pay capabilities; only its `currencyCode` is read, into the
    // prices.
    // pub shop: ShopCapabilities,
    /// The variant picker's legends without their trailing colon, verbatim otherwise: the casing
    /// varies between products (`Type`, `TYPE`).
    pub options: Vec<String>,
    /// The main gallery, in slide order, without the display `width` the theme asks the CDN for.
    pub images: Vec<String>,
    // Commented out: the product title on every slide.
    // pub image_alts: Vec<Option<String>>,
    pub variants: Vec<Variant>,
}

impl Product {
    pub(crate) fn from_raw(raw: raw::MinisforumRuRawDocument) -> Result<Product, NormalizeError> {
        let currency = shop_currency(&raw.shop.data.currency_code)?;
        let meta = raw.analytics.meta.product;

        if raw.variant_picker.product_id != meta.id.to_string() {
            return Err(NormalizeError::ProductIdMismatch {
                picker: raw.variant_picker.product_id,
                analytics: meta.id,
            });
        }
        if raw.variant_picker.product_url != format!("/products/{}", meta.handle) {
            return Err(NormalizeError::ProductUrlMismatch {
                url: raw.variant_picker.product_url,
                handle: meta.handle,
            });
        }

        // The picker renders every variant's record only when there is a choice to make; a
        // single-variant product leaves just the selected one.
        let records = match raw.variants {
            Some(variants) => variants.data,
            None => vec![raw.selected_variant.data],
        };

        let analytics: Vec<u64> = meta.variants.iter().map(|variant| variant.id).collect();
        let picker: Vec<u64> = records.iter().map(|variant| variant.id).collect();
        if analytics != picker {
            return Err(NormalizeError::VariantIdsMismatch { analytics, picker });
        }

        let options: Vec<String> = raw
            .option_names
            .into_iter()
            .map(|legend| option_name(&legend.text))
            .collect();

        let variants = records
            .into_iter()
            .map(|record| Variant::from_raw(record, &options, currency))
            .collect::<Result<_, _>>()?;

        Ok(Product {
            id: meta.gid,
            url: format!("{ORIGIN}{}", raw.variant_picker.product_url),
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
        })
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Variant {
    pub id: String,
    /// The selected values joined with ` / `.
    pub title: String,
    // Commented out: `title` again.
    // pub public_title: Option<String>,
    // Commented out: `options` again, one slot per product option.
    // pub option1: Option<String>,
    /// The product title and `title`, joined with ` - `.
    pub name: String,
    /// Empty or null on every page so far.
    pub sku: Option<String>,
    /// Null on every page so far.
    pub barcode: Option<String>,
    pub price: Money,
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
    // Commented out: no selling plans on any page; `requires_selling_plan` is false throughout.
    // pub selling_plan_allocations: Vec<Value>,
    pub options: Vec<VariantOption>,
}

impl Variant {
    fn from_raw(
        raw: raw::variant::Variant,
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
            price: money_from_shopify_cents(raw.price, currency),
            compare_at_price: raw
                .compare_at_price
                .map(|cents| money_from_shopify_cents(cents, currency)),
            available: raw.available,
            requires_shipping: raw.requires_shipping,
            taxable: raw.taxable,
            weight: unweighed(raw.weight),
            // featured_image: raw.featured_image,
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

// A legend reads `CPU:`.
fn option_name(legend: &str) -> String {
    legend.trim().trim_end_matches(':').trim_end().to_string()
}

// The gallery asks the CDN for `//minisforum.ru/cdn/shop/files/a.png?v=1&width=1946`; the image is
// the file at its version, and the width only how large the theme displays it.
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

// Shopify defaults an unentered weight to 0: unmeasured, not weightless.
fn unweighed(weight: i64) -> Option<i64> {
    (weight != 0).then_some(weight)
}

fn non_empty(value: String) -> Option<String> {
    (!value.is_empty()).then_some(value)
}

#[cfg(test)]
mod tests {
    use super::{gallery_image, option_name};

    #[test]
    fn a_legend_loses_its_colon() {
        assert_eq!(option_name("CPU:"), "CPU");
        assert_eq!(option_name(" Size : "), "Size");
    }

    #[test]
    fn only_the_width_parameter_is_dropped() {
        assert_eq!(
            gallery_image("//minisforum.ru/cdn/shop/files/a.png?v=1&width=1946"),
            "https://minisforum.ru/cdn/shop/files/a.png?v=1"
        );
        assert_eq!(
            gallery_image("https://minisforum.ru/a.png?width=1946"),
            "https://minisforum.ru/a.png"
        );
        assert_eq!(
            gallery_image("https://minisforum.ru/a.png"),
            "https://minisforum.ru/a.png"
        );
    }
}
