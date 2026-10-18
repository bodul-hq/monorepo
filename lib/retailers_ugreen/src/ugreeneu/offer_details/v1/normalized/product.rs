use super::super::raw;
use super::island::{Island, IslandVariant};
use super::page_url::PageUrl;
use super::{NormalizeError, money_from_shopify_cents, shop_currency, variant_gid, whole_number};
use money::{Currency, Money};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Product {
    /// The analytics `gid`.
    pub id: String,
    /// The analytics handle, percent-decoded.
    pub handle: String,
    /// The ld+json `url`: the page's canonical URL, up to the case of its percent escapes. Each
    /// locale's page keeps its own URL; pages of one product under different prefixes are not
    /// merged.
    pub url: String,
    /// The URL's locale prefix, such as `de`, `de-at` or `en-se`. `None` on the unprefixed root
    /// (200 pages), which renders English for market `DE` (`Shopify.locale` `en`); no prefix
    /// names it, so it is left untagged rather than given a code it does not state.
    pub locale: Option<String>,
    /// The ld+json `name`, trimmed: 18 French pages end it in a space.
    pub title: String,
    // Commented out: the description as text inside the theme's carousel labels and runs of
    // blank lines, nothing a variation needs.
    // pub description: String,
    /// Free text per product: `UGREEN EU` on most pages, `UGREEN ES`, `UGREEN UK`, `Seel` and
    /// others elsewhere. The ld+json `brand` restates it.
    pub vendor: String,
    /// Translated per locale. `None` where the product is untyped (1,247 pages).
    pub product_type: Option<String>,
    /// The ld+json Google product category, translated per locale. `None` where empty (396
    /// pages).
    pub category: Option<String>,
    // Commented out: false on every page.
    // pub remote: bool,
    // Commented out: the analytics `page` describes this request, not the product.
    // pub page: Page,
    /// The main gallery's image slides, in slide order, without the display `width` the theme
    /// asks the CDN for. Video slides are left out: they render only poster thumbnails.
    pub images: Vec<String>,
    // Commented out: the product title or an English caption, nothing a variation needs.
    // pub image_alts: Vec<Option<String>>,
    /// The variant the page opens on: the first available one, or the first variant.
    pub selected_variant_id: String,
    /// Every variant, in the analytics' order, which is the ld+json's.
    pub variants: Vec<Variant>,
}

impl Product {
    pub(crate) fn from_raw(raw: raw::UgreenEuRawDocument) -> Result<Product, NormalizeError> {
        let currency = shop_currency(&raw.price.currency)?;
        let island = Island::from_raw(raw.ld_json)?;
        let meta = raw.analytics.meta.product;

        let page =
            PageUrl::parse(&island.url).ok_or_else(|| NormalizeError::UnexpectedProductUrl {
                url: island.url.clone(),
            })?;
        if page.handle != meta.handle {
            return Err(NormalizeError::ProductUrlMismatch {
                url: island.url,
                handle: meta.handle,
            });
        }
        if let Some(group_id) = &island.product_id {
            same_product("ld+json", group_id, meta.id)?;
        }
        same_product("price block", &raw.price.product_id, meta.id)?;

        if island.variants.len() != meta.variants.len() {
            return Err(NormalizeError::VariantCountMismatch {
                island: island.variants.len(),
                analytics: meta.variants.len(),
            });
        }
        for (offer, variant) in island.variants.iter().zip(&meta.variants) {
            check_offer(offer, variant, currency)?;
        }

        let selected_id: u64 = whole_number(&raw.selected_variant_id.value, "selected_variant_id")?;
        let (selected, (selected_offer, selected_variant)) = island
            .variants
            .iter()
            .zip(&meta.variants)
            .enumerate()
            .find(|(_, (_, variant))| variant.id == selected_id)
            .ok_or(NormalizeError::SelectedVariantNotListed { id: selected_id })?;

        let prices = SelectedPrices::from_raw(&raw.price, selected_variant.price)?;

        // Only a `ProductGroup` page renders the picker, and every one does.
        let record = match (island.product_id.is_some(), raw.selected_variant) {
            (true, Some(script)) => Some(script.data),
            (true, None) => return Err(NormalizeError::PickerRecordMissing),
            (false, Some(_)) => return Err(NormalizeError::UnexpectedPickerRecord),
            (false, None) => None,
        };
        if let Some(record) = &record {
            check_record(record, selected_offer, selected_variant, &prices)?;
        }

        let variants = island
            .variants
            .into_iter()
            .zip(meta.variants)
            .enumerate()
            .map(|(index, (offer, variant))| {
                let shown = (index == selected).then_some(&prices);
                Variant::from_raw(offer, variant, shown, currency)
            })
            .collect();

        Ok(Product {
            id: meta.gid,
            handle: meta.handle,
            url: island.url,
            locale: page.locale,
            title: island.name.trim().to_string(),
            vendor: meta.vendor,
            product_type: non_empty(meta.product_type),
            category: non_empty(island.category),
            images: raw
                .images
                .into_iter()
                .map(|image| gallery_image(&image.src))
                .collect(),
            selected_variant_id: variant_gid(selected_id),
            variants,
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

// The ld+json and the analytics render from the same Liquid product, one variant for one in the
// same order, so any disagreement means one of the two was misread.
fn check_offer(
    offer: &IslandVariant,
    variant: &raw::analytics::Variant,
    currency: Currency,
) -> Result<(), NormalizeError> {
    if offer.currency != currency.as_str() {
        return Err(NormalizeError::OfferCurrencyMismatch {
            offer: offer.offer_id.clone(),
            island: offer.currency.clone(),
            shop: currency.as_str().to_string(),
        });
    }

    let mismatch = |field: &str, island: String, analytics: String| {
        Err(NormalizeError::VariantMismatch {
            offer: offer.offer_id.clone(),
            field: field.to_string(),
            island,
            analytics,
        })
    };
    if offer.variant_id != variant.id {
        return mismatch("id", offer.variant_id.to_string(), variant.id.to_string());
    }
    if offer.cents != variant.price {
        return mismatch("price", offer.cents.to_string(), variant.price.to_string());
    }
    if Some(&offer.sku) != variant.sku.as_ref() {
        return mismatch("sku", offer.sku.clone(), format!("{:?}", variant.sku));
    }

    Ok(())
}

// The picker's record is Liquid's `variant | json` of the selected variant: it restates the
// island's and the price block's facts about it.
fn check_record(
    record: &raw::variant::Variant,
    offer: &IslandVariant,
    variant: &raw::analytics::Variant,
    prices: &SelectedPrices,
) -> Result<(), NormalizeError> {
    agree("id", record.id == variant.id)?;
    agree("price", record.price == variant.price)?;
    agree("sku", record.sku == variant.sku)?;
    agree("available", record.available == offer.available)?;
    agree(
        "barcode",
        record.barcode.clone().and_then(gtin) == offer.gtin.clone().and_then(gtin),
    )?;
    // Under an automatic discount the block strikes through the Shopify price, not the
    // record's compare-at price, so there is nothing to compare.
    if prices.discounted_cents.is_none() {
        let markdown = record
            .compare_at_price
            .filter(|&compare| compare > record.price);
        agree("compare_at_price", markdown == prices.compare_cents)?;
    }

    Ok(())
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
    /// The selected values joined with ` / `. `None` on a `Default Title` variant.
    pub title: Option<String>,
    /// The product title and `title`, joined with ` - `, trimmed.
    pub name: String,
    pub sku: Option<String>,
    /// The ld+json `gtin`, trimmed. `None` where the island states none (on every variant of
    /// 2,875 pages and some variants of 389 others) or where it is not all digits (no page).
    pub gtin: Option<String>,
    /// The price customers see. The Shopify price, except on the selected variant where the
    /// shop's automatic discount applies (2,237 pages): there it is the discounted price the
    /// price block displays. The other variants' discounted prices are not rendered. `None`
    /// where Shopify states 0, a placeholder rather than a price (no page so far).
    pub price: Option<Money>,
    /// The selected variant's compare-at price: its Shopify price where the automatic discount
    /// applies, otherwise its struck-through compare-at price where that is above the price.
    /// `None` on every other variant: no island the library can read states theirs.
    pub compare_at_price: Option<Money>,
    /// True on the selected variant where `price` is after the automatic discount.
    pub automatic_discount: bool,
    /// From the ld+json offer's `availability`.
    pub available: bool,
    /// The ld+json variant's image, without the display `width`.
    pub image: String,
}

impl Variant {
    fn from_raw(
        offer: IslandVariant,
        variant: raw::analytics::Variant,
        shown: Option<&SelectedPrices>,
        currency: Currency,
    ) -> Variant {
        let discounted = shown.and_then(|prices| prices.discounted_cents);
        let (price, compare) = match (shown, discounted) {
            // The decision on discounts: the offer is the price customers see, and the Shopify
            // price it is discounted from becomes its compare-at price.
            (_, Some(discounted)) => (discounted, Some(variant.price)),
            (Some(prices), None) => (variant.price, prices.compare_cents),
            (None, None) => (variant.price, None),
        };

        Variant {
            id: variant_gid(variant.id),
            title: variant.public_title,
            name: variant.name.trim().to_string(),
            sku: variant.sku.and_then(non_empty),
            gtin: offer.gtin.and_then(gtin),
            price: unsold_placeholder_price(price)
                .map(|cents| money_from_shopify_cents(cents, currency)),
            compare_at_price: compare.map(|cents| money_from_shopify_cents(cents, currency)),
            automatic_discount: discounted.is_some(),
            available: offer.available,
            image: gallery_image(&offer.image),
        }
    }
}

/// The price block's reading of the selected variant's prices against its Shopify price.
struct SelectedPrices {
    /// The displayed price, where the automatic discount puts it below the Shopify price.
    discounted_cents: Option<i64>,
    /// The struck-through price, where there is no automatic discount and it is above the
    /// Shopify price.
    compare_cents: Option<i64>,
}

impl SelectedPrices {
    // The block displays `base_price_cents` and strikes through `base_compare_cents`. Without an
    // automatic discount these are the Shopify price and the compare-at price (equal to the
    // price where the variant is not marked down). With one, the theme displays the discounted
    // price and strikes through the Shopify price on 2,233 pages; on 4 (`en-fi`, `en-hr` and
    // `hr-hr` power banks) it strikes through a compare-at price above the Shopify price
    // instead, which the discount decision leaves aside.
    fn from_raw(block: &raw::PriceBlock, listed: i64) -> Result<SelectedPrices, NormalizeError> {
        let displayed: i64 = whole_number(&block.base_price_cents, "base_price_cents")?;
        let compare: i64 = whole_number(&block.base_compare_cents, "base_compare_cents")?;

        if displayed > listed {
            return Err(NormalizeError::DisplayedPriceAboveListed { displayed, listed });
        }
        if compare < listed {
            return Err(NormalizeError::StruckPriceBelowListed { compare, listed });
        }

        Ok(SelectedPrices {
            discounted_cents: (displayed < listed).then_some(displayed),
            compare_cents: (displayed == listed && compare > listed).then_some(compare),
        })
    }
}

/// A barcode is a GTIN only where it is all digits once surrounding whitespace, including
/// no-break spaces, is trimmed. Every ld+json `gtin` in the corpus is 13 digits.
fn gtin(barcode: String) -> Option<String> {
    let trimmed = barcode.trim();
    (!trimmed.is_empty() && trimmed.bytes().all(|b| b.is_ascii_digit()))
        .then(|| trimmed.to_string())
}

// Shopify renders 0 for a product it announces but does not sell.
fn unsold_placeholder_price(cents: i64) -> Option<i64> {
    (cents != 0).then_some(cents)
}

// The gallery asks the CDN for `//eu.ugreen.com/cdn/shop/files/a.png?v=1&width=3840`, the
// ld+json for `https://…&width=1920`; the image is the file at its version, and the width only
// how large it is displayed.
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
    let trimmed = value.trim();
    (!trimmed.is_empty()).then(|| trimmed.to_string())
}

#[cfg(test)]
mod tests {
    use super::{gallery_image, gtin};

    #[test]
    fn only_the_width_parameter_is_dropped() {
        assert_eq!(
            gallery_image("//eu.ugreen.com/cdn/shop/files/a.png?v=1&width=3840"),
            "https://eu.ugreen.com/cdn/shop/files/a.png?v=1"
        );
        assert_eq!(
            gallery_image("https://eu.ugreen.com/a.png?width=1920"),
            "https://eu.ugreen.com/a.png"
        );
        assert_eq!(
            gallery_image("https://eu.ugreen.com/a.png"),
            "https://eu.ugreen.com/a.png"
        );
    }

    #[test]
    fn a_gtin_is_all_digits_after_trimming() {
        assert_eq!(
            gtin("6941876229192".to_string()),
            Some("6941876229192".to_string())
        );
        assert_eq!(
            gtin(" 6941876229192\u{a0}".to_string()),
            Some("6941876229192".to_string())
        );
        assert_eq!(gtin("6941876-229192".to_string()), None);
        assert_eq!(gtin("N/A".to_string()), None);
        assert_eq!(gtin(" ".to_string()), None);
    }
}
