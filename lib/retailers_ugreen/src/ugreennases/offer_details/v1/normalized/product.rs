use super::super::raw;
use super::{
    NormalizeError, ORIGIN, https_from_protocol_relative, money_from_shopify_cents, product_gid,
    shop_currency, variant_gid, whole_number,
};
use money::{Currency, Money};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Product {
    /// The analytics `gid`.
    pub id: String,
    pub handle: String,
    /// The handle's page on the storefront's origin.
    pub url: String,
    /// The main section's `h1`, trimmed, with runs of whitespace collapsed. `og:title` is a
    /// marketing title on most pages and is not read.
    pub title: String,
    /// Free text per product: `UGREEN NAS US` and `UGREEN NAS EU` appear.
    pub vendor: String,
    /// `None` where the product is untyped, as every saved product is.
    pub product_type: Option<String>,
    // Commented out: false on every page.
    // pub remote: bool,
    // Commented out: the analytics `page` describes this request, not the product.
    // pub page: Page,
    /// The main gallery's slides, in slide order, without the display `width` the theme asks the
    /// CDN for.
    pub images: Vec<String>,
    // Commented out: the product title or a caption, nothing a variation needs.
    // pub image_alts: Vec<String>,
    /// Every variant, in the analytics' order.
    pub variants: Vec<Variant>,
}

impl Product {
    pub(crate) fn from_raw(raw: raw::UgreenNasEsRawDocument) -> Result<Product, NormalizeError> {
        let currency = shop_currency(&raw.paypal.currency)?;
        let meta = raw.analytics.meta;
        let product = meta.product;

        if product.gid != product_gid(product.id) {
            return Err(NormalizeError::ProductGidMismatch {
                id: product.id,
                gid: product.gid,
            });
        }
        if meta.page.resource_id != product.id {
            return Err(NormalizeError::PageResourceMismatch {
                resource_id: meta.page.resource_id,
                product_id: product.id,
            });
        }

        let title = collapse_whitespace(&raw.title.text);
        if title.is_empty() {
            return Err(NormalizeError::EmptyTitle);
        }

        // The price block and the cart form describe the variant the page opens on.
        let block = PriceBlock::from_raw(&raw.price)?;
        let selected_id: u64 = whole_number(&raw.selected_variant.value, "selected_variant")?;
        let selected = product
            .variants
            .iter()
            .position(|variant| variant.id == selected_id)
            .ok_or(NormalizeError::SelectedVariantNotListed { id: selected_id })?;
        let listed = product.variants[selected].price;
        for (field, displayed) in [("regular", block.regular), ("sale", block.sale)] {
            if displayed != listed {
                return Err(NormalizeError::DisplayedPriceMismatch {
                    field: field.to_string(),
                    displayed,
                    listed,
                });
            }
        }
        let disabled = raw.selected_variant.disabled.is_some();
        block_agrees("sold_out", block.sold_out, disabled)?;

        let variants = match raw.variants {
            Some(script) => with_picker(product.variants, script.data, currency)?,
            None => without_picker(product.variants, &block, disabled, currency)?,
        };

        let selected = &variants[selected];
        block_agrees("available", !block.sold_out, selected.available)?;
        let markdown = selected.compare_at_price.as_ref().map(Money::amount);
        block_agrees("on_sale", block.on_sale, markdown.is_some())?;
        if block.on_sale && block.compare_at != markdown {
            return Err(NormalizeError::PriceBlockMismatch {
                field: "compare_at".to_string(),
                block: format!("{:?}", block.compare_at),
                variant: format!("{markdown:?}"),
            });
        }

        Ok(Product {
            id: product.gid,
            url: format!("{ORIGIN}/products/{}", product.handle),
            handle: product.handle,
            title,
            vendor: product.vendor,
            product_type: non_empty(product.product_type),
            images: raw
                .images
                .iter()
                .map(|image| gallery_image(&image.src))
                .collect(),
            variants,
        })
    }
}

// The picker renders `product.variants | json` from the same Liquid product the analytics
// describe, so any disagreement means one of the two was misread.
fn with_picker(
    analytics: Vec<raw::analytics::Variant>,
    records: Vec<raw::variants::Variant>,
    currency: Currency,
) -> Result<Vec<Variant>, NormalizeError> {
    if records.len() != analytics.len() {
        return Err(NormalizeError::PickerVariantCountMismatch {
            picker: records.len(),
            analytics: analytics.len(),
        });
    }

    analytics
        .into_iter()
        .zip(records)
        .map(|(variant, record)| {
            let mismatch = |field: &str, picker: String, analytics: String| {
                Err(NormalizeError::PickerVariantMismatch {
                    variant_id: variant.id,
                    field: field.to_string(),
                    picker,
                    analytics,
                })
            };
            if record.id != variant.id {
                return mismatch("id", record.id.to_string(), variant.id.to_string());
            }
            if record.sku != variant.sku {
                return mismatch("sku", record.sku.clone(), variant.sku.clone());
            }
            if record.price != variant.price {
                return mismatch("price", record.price.to_string(), variant.price.to_string());
            }

            let price = unsold_placeholder_price(variant.price);
            Ok(Variant {
                id: variant_gid(variant.id),
                title: variant.public_title,
                name: variant.name,
                sku: non_empty(variant.sku),
                barcode: record.barcode.as_deref().and_then(gtin_digits),
                compare_at_price: markdown(price, record.compare_at_price)
                    .map(|cents| money_from_shopify_cents(cents, currency)),
                price: price.map(|cents| money_from_shopify_cents(cents, currency)),
                available: record.available,
                weight: unweighed(record.weight),
                image: record
                    .featured_image
                    .map(|image| https_from_protocol_relative(&image.src)),
                // option1..3 restate `options`.
                options: record.options,
            })
        })
        .collect()
}

// Without a picker the product has a single variant, and only the price block and the cart
// form state its markdown and availability.
fn without_picker(
    analytics: Vec<raw::analytics::Variant>,
    block: &PriceBlock,
    disabled: bool,
    currency: Currency,
) -> Result<Vec<Variant>, NormalizeError> {
    if analytics.len() != 1 {
        return Err(NormalizeError::NoPickerForSeveralVariants {
            count: analytics.len(),
        });
    }

    Ok(analytics
        .into_iter()
        .map(|variant| {
            let price = unsold_placeholder_price(variant.price);
            Variant {
                id: variant_gid(variant.id),
                title: variant.public_title,
                name: variant.name,
                sku: non_empty(variant.sku),
                barcode: None,
                compare_at_price: markdown(price, block.compare_at)
                    .map(|cents| money_from_shopify_cents(cents, currency)),
                price: price.map(|cents| money_from_shopify_cents(cents, currency)),
                available: !disabled,
                weight: None,
                image: None,
                options: Vec::new(),
            }
        })
        .collect())
}

fn block_agrees(field: &str, block: bool, variant: bool) -> Result<(), NormalizeError> {
    if block == variant {
        return Ok(());
    }

    Err(NormalizeError::PriceBlockMismatch {
        field: field.to_string(),
        block: block.to_string(),
        variant: variant.to_string(),
    })
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Variant {
    pub id: String,
    /// The selected values joined with ` / `. `None` on a product without options.
    pub title: Option<String>,
    /// The product's admin title and `title`, joined with ` - `, or the admin title alone.
    pub name: String,
    pub sku: Option<String>,
    /// The picker's `barcode`, trimmed, where it is all digits: a 13-digit EAN on 15 of the 34
    /// variants with a picker, null on the rest. Pages without a picker render none. No saved
    /// barcode is dropped for holding something other than digits.
    pub barcode: Option<String>,
    /// The analytics price. `None` where Shopify renders 0, which no saved variant does.
    pub price: Option<Money>,
    /// The picker's `compare_at_price` where it is above the price; without a picker, the price
    /// block's struck-through price where it is above the price.
    pub compare_at_price: Option<Money>,
    /// The picker's `available`; without a picker, whether the cart form's `id` input is
    /// enabled (the price block's `price--sold-out` agrees).
    pub available: bool,
    /// Unit unknown: Liquid documents grams. `None` where the variant is unweighed or the page
    /// renders no picker; 18 of the 34 saved variants with a picker carry one (16800 or 5280).
    pub weight: Option<i64>,
    /// The picker's `featured_image.src`. `None` where the variant has no image of its own or the
    /// page renders no picker.
    pub image: Option<String>,
    // Commented out: the same image as a media entry.
    // pub featured_media: Option<FeaturedMedia>,
    // Commented out: `requires_shipping` and `taxable` are true throughout.
    // pub requires_shipping: bool,
    // pub taxable: bool,
    // Commented out: `shopify` or null; stock is tracked, but no quantity is rendered.
    // pub inventory_management: Option<String>,
    // Commented out: one unit, no maximum, in steps of one, on every record.
    // pub quantity_rule: QuantityRule,
    // Commented out: no selling plans on any page; `requires_selling_plan` is false throughout.
    // pub selling_plan_allocations: Vec<Value>,
    /// The picker's option values, in the product's option order. The page names the options
    /// nowhere the architecture reads. Empty without a picker.
    pub options: Vec<String>,
}

/// Dawn's price snippet, read as hundredths of the shop currency.
struct PriceBlock {
    on_sale: bool,
    sold_out: bool,
    regular: i64,
    sale: i64,
    /// `None` where the struck-through price is empty.
    compare_at: Option<i64>,
}

impl PriceBlock {
    fn from_raw(raw: &raw::PriceBlock) -> Result<PriceBlock, NormalizeError> {
        let classes: Vec<&str> = raw.class.split_whitespace().collect();
        let compare_at = raw.compare_at.text.trim();

        Ok(PriceBlock {
            on_sale: classes.contains(&"price--on-sale"),
            sold_out: classes.contains(&"price--sold-out"),
            regular: rendered_price(&raw.regular.text, "regular")?,
            sale: rendered_price(&raw.sale.text, "sale")?,
            compare_at: if compare_at.is_empty() {
                None
            } else {
                Some(rendered_price(compare_at, "compare_at")?)
            },
        })
    }
}

fn rendered_price(text: &str, field: &str) -> Result<i64, NormalizeError> {
    rendered_cents(text).ok_or_else(|| NormalizeError::NotARenderedPrice {
        field: field.to_string(),
        text: text.to_string(),
    })
}

// `$1,149.99 AUD`, `2.349,99€`, `€2.349,99`: the amount runs from the first digit to the last,
// and every price has two decimals after a point or a comma, with the other one grouping
// thousands.
fn rendered_cents(text: &str) -> Option<i64> {
    let start = text.find(|c: char| c.is_ascii_digit())?;
    let end = text.rfind(|c: char| c.is_ascii_digit())? + 1;
    let amount = &text.as_bytes()[start..end];

    let split = amount.len().checked_sub(3)?;
    let (units, fraction) = amount.split_at(split);
    let (separator, hundredths) = fraction.split_first()?;
    let grouping = match separator {
        b'.' => b',',
        b',' => b'.',
        _ => return None,
    };
    if units.is_empty()
        || !units[0].is_ascii_digit()
        || !units
            .iter()
            .all(|byte| byte.is_ascii_digit() || *byte == grouping)
        || !hundredths.iter().all(u8::is_ascii_digit)
    {
        return None;
    }

    units
        .iter()
        .chain(hundredths)
        .filter(|byte| byte.is_ascii_digit())
        .try_fold(0_i64, |cents, byte| {
            cents.checked_mul(10)?.checked_add(i64::from(byte - b'0'))
        })
}

// A barcode is a GTIN only where it is digits. GTIN-14 is a valid length: the bundle variants
// of `ugreennasde` carry 14-digit `178…` codes.
fn gtin_digits(barcode: &str) -> Option<String> {
    let trimmed = barcode.trim();
    (!trimmed.is_empty() && trimmed.bytes().all(|byte| byte.is_ascii_digit()))
        .then(|| trimmed.to_string())
}

// Only a compare-at price above the price marks the variant down; one equal to it is the
// storefront restating the price.
fn markdown(price: Option<i64>, compare_at: Option<i64>) -> Option<i64> {
    let price = price?;
    compare_at.filter(|compare_at| *compare_at > price)
}

// 0 is not a price the storefront sells at; the family reads it as a product announced but not
// sold. No saved variant renders one.
fn unsold_placeholder_price(cents: i64) -> Option<i64> {
    (cents != 0).then_some(cents)
}

// Shopify defaults an unentered weight to 0: unmeasured, not weightless.
fn unweighed(weight: i64) -> Option<i64> {
    (weight != 0).then_some(weight)
}

fn collapse_whitespace(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

// The gallery asks the CDN for `//ai-es.ugreen.com/cdn/shop/files/a.png?v=1&width=1445`; the
// image is the file at its version, and the width only how large the theme displays it.
fn gallery_image(src: &str) -> String {
    let url = https_from_protocol_relative(src);

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
    use super::{collapse_whitespace, gallery_image, gtin_digits, markdown, rendered_cents};

    #[test]
    fn a_rendered_price_is_read_in_each_locale() {
        assert_eq!(rendered_cents("$1,149.99 AUD"), Some(114999));
        assert_eq!(rendered_cents("$679.99 AUD"), Some(67999));
        assert_eq!(rendered_cents("2.349,99€"), Some(234999));
        assert_eq!(rendered_cents("€2.349,99"), Some(234999));
        assert_eq!(rendered_cents("0,98€"), Some(98));
    }

    #[test]
    fn a_rendered_price_needs_two_decimals_and_one_grouping_mark() {
        assert_eq!(rendered_cents("$679.9 AUD"), None);
        assert_eq!(rendered_cents("$679 AUD"), None);
        assert_eq!(rendered_cents("1.149.99"), None);
        assert_eq!(rendered_cents(",99"), None);
        assert_eq!(rendered_cents("1 149,99"), None);
        assert_eq!(rendered_cents(""), None);
        assert_eq!(rendered_cents("AUD"), None);
    }

    #[test]
    fn only_digits_make_a_gtin() {
        assert_eq!(
            gtin_digits(" 6942752010545\u{a0}"),
            Some("6942752010545".to_string())
        );
        assert_eq!(
            gtin_digits("17841947458048"),
            Some("17841947458048".to_string())
        );
        assert_eq!(gtin_digits("6942752-010545"), None);
        assert_eq!(gtin_digits("\u{a0}"), None);
    }

    #[test]
    fn only_a_higher_compare_at_price_is_a_markdown() {
        assert_eq!(markdown(Some(100), Some(120)), Some(120));
        assert_eq!(markdown(Some(100), Some(100)), None);
        assert_eq!(markdown(Some(100), Some(90)), None);
        assert_eq!(markdown(Some(100), None), None);
        assert_eq!(markdown(None, Some(120)), None);
    }

    #[test]
    fn whitespace_runs_collapse() {
        assert_eq!(collapse_whitespace("\n a  b\u{a0} c \n"), "a b c");
    }

    #[test]
    fn only_the_width_parameter_is_dropped() {
        assert_eq!(
            gallery_image("//ai-es.ugreen.com/cdn/shop/files/a.png?v=1&width=1445"),
            "https://ai-es.ugreen.com/cdn/shop/files/a.png?v=1"
        );
        assert_eq!(
            gallery_image("https://ai-es.ugreen.com/a.png?width=1445"),
            "https://ai-es.ugreen.com/a.png"
        );
        assert_eq!(
            gallery_image("https://ai-es.ugreen.com/a.png"),
            "https://ai-es.ugreen.com/a.png"
        );
    }
}
