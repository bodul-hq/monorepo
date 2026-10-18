use super::super::raw;
use super::island::{Island, IslandOffer};
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
    /// The product form's path on the storefront's origin; the ld+json `url` agrees.
    pub url: String,
    /// The theme ld+json `name`, equal to the main section's `h1` on every saved page.
    pub title: String,
    /// `UGREEN FR`, `UGREEN` or `Seel`.
    pub vendor: String,
    // Commented out: the ld+json `brand` restates `vendor` on every page.
    // pub brand: String,
    // Commented out: the ld+json `description` is the theme's whitespace or marketing HTML.
    // pub description: String,
    /// `None` where the product is untyped (62 of the 306 saved pages).
    pub product_type: Option<String>,
    /// The ld+json Google product category. `None` where it is empty (55 pages).
    pub category: Option<String>,
    // Commented out: false on every page.
    // pub remote: bool,
    // Commented out: the analytics `page` describes this request, not the product.
    // pub page: Page,
    /// The variant picker's legend titles without their trailing colon. Empty without a picker.
    pub options: Vec<String>,
    /// The main gallery's image slides, in slide order, without the display `width` the theme
    /// asks the CDN for. Video slides are left out: they render only poster thumbnails.
    pub images: Vec<String>,
    // Commented out: the product title, a French caption or `alt`, nothing a variation needs.
    // pub image_alts: Vec<String>,
    // Commented out: the ld+json's per-variant `image` is one of the gallery's.
    /// The product form's `data-ug-discount-code` (`DL<sku>`), where an automatic discount
    /// prices the selected variant (40 of the 306 saved pages).
    pub automatic_discount_code: Option<String>,
    /// Every variant, in the analytics' order, which is the island's.
    pub variants: Vec<Variant>,
    pub selected_variant: SelectedVariant,
}

impl Product {
    pub(crate) fn from_raw(raw: raw::UgreenFrRawDocument) -> Result<Product, NormalizeError> {
        let currency = shop_currency(&raw.price.currency)?;
        let meta = raw.analytics.meta.product;
        let island = Island::from_raw(raw.ld_json)?;

        let path = format!("/products/{}", meta.handle);
        let url = format!("{ORIGIN}{path}");
        same_product("price block", &raw.price.product_id, meta.id)?;
        same_product("product form", &raw.product_form.product_id, meta.id)?;
        if let Some(id) = &island.product_id {
            same_product("ld+json island", id, meta.id)?;
        }
        same_url("product form", &raw.product_form.product_url, &path)?;
        same_url("ld+json island", &island.url, &url)?;

        check_offers(&island.offers, &meta.variants, currency)?;

        let options: Vec<String> = raw
            .option_names
            .iter()
            .map(|legend| option_name(&legend.text))
            .collect();

        let selected_id: u64 = whole_number(&raw.selected_variant_id.value, "selected_variant_id")?;
        let Some(selected_index) = meta
            .variants
            .iter()
            .position(|variant| variant.id == selected_id)
        else {
            return Err(NormalizeError::SelectedVariantNotListed { id: selected_id });
        };
        let listed = meta.variants[selected_index].price;
        // Always found: the island's offers were checked against the analytics variants.
        let selected_offer = island.offers.get(selected_index);

        let prices = DisplayedPrices::from_raw(&raw.price, &raw.product_form, listed, currency)?;

        let record = match (options.is_empty(), raw.selected_variant) {
            (_, Some(script)) => Some(script.data),
            (false, None) => return Err(NormalizeError::PickerWithoutSelectedVariant),
            // Without a picker nothing renders the record, so there is nothing to check.
            (true, None) => None,
        };
        let selected_options = match record {
            Some(record) => {
                check_record(
                    &record,
                    selected_id,
                    listed,
                    &meta.variants[selected_index].sku,
                    selected_offer,
                    &prices,
                )?;
                variant_options(record.id, &options, record.options)?
            }
            None => Vec::new(),
        };

        let variants = meta
            .variants
            .into_iter()
            .zip(island.offers)
            .map(|(variant, offer)| {
                let selected = variant.id == selected_id;
                Variant {
                    id: variant_gid(variant.id),
                    title: variant.public_title,
                    name: variant.name,
                    sku: non_empty(variant.sku),
                    gtin: gtin(offer.gtin),
                    price: if selected {
                        prices.price.clone()
                    } else {
                        money_from_shopify_cents(variant.price, currency)
                    },
                    compare_at_price: if selected {
                        prices.compare_at_price.clone()
                    } else {
                        None
                    },
                    available: offer.in_stock,
                }
            })
            .collect();

        Ok(Product {
            id: meta.gid,
            handle: meta.handle,
            url,
            title: island.title,
            vendor: meta.vendor,
            product_type: non_empty(meta.product_type),
            category: non_empty(island.category),
            options,
            images: raw
                .images
                .into_iter()
                .map(|image| gallery_image(&image.src))
                .collect(),
            automatic_discount_code: prices.automatic_discount_code,
            variants,
            selected_variant: SelectedVariant {
                id: variant_gid(selected_id),
                options: selected_options,
            },
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

fn same_url(island: &str, url: &str, expected: &str) -> Result<(), NormalizeError> {
    if url == expected {
        return Ok(());
    }

    Err(NormalizeError::ProductUrlMismatch {
        island: island.to_string(),
        url: url.to_string(),
        expected: expected.to_string(),
    })
}

// The island and the analytics render from the same Liquid product, in the same order on all
// 306 saved pages, so any disagreement means one of the two was misread.
fn check_offers(
    offers: &[IslandOffer],
    variants: &[raw::analytics::Variant],
    currency: Currency,
) -> Result<(), NormalizeError> {
    let island_ids: Vec<u64> = offers.iter().map(|offer| offer.variant_id).collect();
    let analytics_ids: Vec<u64> = variants.iter().map(|variant| variant.id).collect();
    if island_ids != analytics_ids {
        return Err(NormalizeError::VariantIdsMismatch {
            island: island_ids,
            analytics: analytics_ids,
        });
    }

    for (offer, variant) in offers.iter().zip(variants) {
        if offer.currency != currency.as_str() {
            return Err(NormalizeError::OfferCurrencyMismatch {
                offer: offer.id.clone(),
                island: offer.currency.clone(),
                shop: currency.as_str().to_string(),
            });
        }

        let mismatch = |field: &str, island: String, analytics: String| {
            Err(NormalizeError::VariantFieldMismatch {
                variant_id: variant.id,
                field: field.to_string(),
                island,
                analytics,
            })
        };
        if offer.cents != variant.price {
            return mismatch("price", offer.cents.to_string(), variant.price.to_string());
        }
        // The gift cards' variants have no `sku` on the island and an empty one in analytics.
        let analytics_sku = non_empty(variant.sku.clone());
        if offer.sku != analytics_sku {
            return mismatch(
                "sku",
                format!("{:?}", offer.sku),
                format!("{analytics_sku:?}"),
            );
        }
    }

    Ok(())
}

// The picker's record restates the selected variant; checked so a picker describing another
// variant than the form can't pass unnoticed.
fn check_record(
    record: &raw::variant::Variant,
    selected_id: u64,
    listed: i64,
    analytics_sku: &str,
    offer: Option<&IslandOffer>,
    prices: &DisplayedPrices,
) -> Result<(), NormalizeError> {
    agree("id", record.id == selected_id)?;
    agree("price", record.price == listed)?;
    agree("sku", record.sku == analytics_sku)?;
    agree(
        "available",
        offer.is_some_and(|offer| offer.in_stock == record.available),
    )?;
    let island_gtin = offer.and_then(|offer| offer.gtin.as_deref()).map(str::trim);
    let record_barcode = record.barcode.as_deref().map(str::trim);
    agree("barcode", island_gtin == record_barcode)?;
    // Under an automatic discount the block strikes through the Shopify price, not the
    // record's compare-at price; no saved page has both a discount and a picker.
    if prices.automatic_discount_code.is_none() {
        let record_compare = record.compare_at_price.filter(|&compare| compare > listed);
        let block_compare = prices.compare_at_price.as_ref().map(Money::amount);
        agree("compare_at_price", record_compare == block_compare)?;
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
    /// The selected values joined with ` / `. `None` on a product without options.
    pub title: Option<String>,
    /// The product title and `title`, joined with ` - `, or the product title alone.
    pub name: String,
    /// `None` on the three gift cards (`ugreen-carte-cadeau-*`).
    pub sku: Option<String>,
    /// The ld+json `gtin`, trimmed: six values on 16 variants end in a no-break space. `None`
    /// where the island has none (400 of the 598 saved variants), or where the trimmed value is
    /// not all digits (none of the saved variants).
    pub gtin: Option<String>,
    /// The price customers see: the Shopify variant price, except for the selected variant
    /// under an automatic discount, whose price is the discounted one.
    pub price: Money,
    /// Only the selected variant's: no island renders the others'. The struck-through price
    /// where the variant is marked down, or its Shopify price under an automatic discount.
    pub compare_at_price: Option<Money>,
    /// The ld+json offer's `InStock`.
    pub available: bool,
}

/// What the page states about the variant it opens on, beyond its prices.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SelectedVariant {
    /// The product form's `id` input; the first variant on all but 3 saved pages.
    pub id: String,
    // Commented out: `requires_shipping` and `taxable` are true on every record.
    // Commented out: the variant's image is one of the gallery's.
    // pub featured_image: Option<FeaturedImage>,
    // pub featured_media: Option<FeaturedMedia>,
    // Commented out: no quantity is rendered; one unit, no maximum, in steps of one; no
    // selling plans; unweighed.
    /// The picker's record's values paired with the legends. Empty without a picker.
    pub options: Vec<VariantOption>,
}

/// The price block's reading of the selected variant's prices against its Shopify price.
struct DisplayedPrices {
    price: Money,
    compare_at_price: Option<Money>,
    automatic_discount_code: Option<String>,
}

impl DisplayedPrices {
    // The block displays `base_price_cents` and strikes through `base_compare_cents`. Without an
    // automatic discount these are the variant's Shopify price and its compare-at price (equal
    // to the price where it is not marked down; 209 of 266 pages show a higher one). With one,
    // flagged on the form, the theme displays the discounted price and strikes through the
    // Shopify price, as it does on all 40 such pages. The offer is the price customers see, so
    // the discounted price is the price and the Shopify price its compare-at price.
    fn from_raw(
        block: &raw::PriceBlock,
        form: &raw::ProductForm,
        listed: i64,
        currency: Currency,
    ) -> Result<DisplayedPrices, NormalizeError> {
        let displayed: i64 = whole_number(&block.base_price_cents, "base_price_cents")?;
        let compare: i64 = whole_number(&block.base_compare_cents, "base_compare_cents")?;

        let flagged = match form.auto_discount.as_deref() {
            None => false,
            Some("true") => true,
            Some(value) => {
                return Err(NormalizeError::AutomaticDiscountFlag {
                    value: value.to_string(),
                });
            }
        };
        if flagged != form.discount_code.is_some() {
            return Err(NormalizeError::DiscountCodeMismatch {
                flagged,
                code: form.discount_code.clone(),
            });
        }

        if displayed > listed {
            return Err(NormalizeError::DisplayedPriceAboveListed { displayed, listed });
        }
        if flagged != (displayed < listed) {
            return Err(NormalizeError::AutomaticDiscountMismatch {
                flagged,
                displayed,
                listed,
            });
        }

        if flagged {
            if compare != listed {
                return Err(NormalizeError::DiscountedFromAnotherPrice {
                    displayed,
                    listed,
                    compare,
                });
            }

            return Ok(DisplayedPrices {
                price: money_from_shopify_cents(displayed, currency),
                compare_at_price: Some(money_from_shopify_cents(listed, currency)),
                automatic_discount_code: form.discount_code.clone(),
            });
        }

        Ok(DisplayedPrices {
            price: money_from_shopify_cents(listed, currency),
            compare_at_price: (compare > listed)
                .then(|| money_from_shopify_cents(compare, currency)),
            automatic_discount_code: None,
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

// Six barcodes on 16 variants end in a no-break space (`6938624439712\u{a0}`), which `trim`
// removes. A value that is still not all digits is not a GTIN; none of the saved pages has one.
fn gtin(barcode: Option<String>) -> Option<String> {
    let trimmed = barcode?.trim().to_string();
    (!trimmed.is_empty() && trimmed.bytes().all(|b| b.is_ascii_digit())).then_some(trimmed)
}

// A legend title reads `Pack:`.
fn option_name(legend: &str) -> String {
    legend.trim().trim_end_matches(':').trim_end().to_string()
}

// The gallery asks the CDN for `//fr.ugreen.com/cdn/shop/files/a.png?v=1&width=3840`; the image
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
    use super::{gallery_image, gtin, option_name};

    #[test]
    fn a_legend_loses_its_colon() {
        assert_eq!(option_name("Pack:"), "Pack");
        assert_eq!(option_name(" Longueur : "), "Longueur");
    }

    #[test]
    fn a_barcode_is_trimmed_and_kept_only_as_digits() {
        assert_eq!(
            gtin(Some("6938624439712\u{a0}".to_string())),
            Some("6938624439712".to_string())
        );
        assert_eq!(
            gtin(Some("6941876246991".to_string())),
            Some("6941876246991".to_string())
        );
        assert_eq!(gtin(Some("N/A".to_string())), None);
        assert_eq!(gtin(Some("\u{a0}".to_string())), None);
        assert_eq!(gtin(None), None);
    }

    #[test]
    fn only_the_width_parameter_is_dropped() {
        assert_eq!(
            gallery_image("//fr.ugreen.com/cdn/shop/files/a.png?v=1&width=3840"),
            "https://fr.ugreen.com/cdn/shop/files/a.png?v=1"
        );
        assert_eq!(
            gallery_image("https://fr.ugreen.com/a.png?width=3840"),
            "https://fr.ugreen.com/a.png"
        );
        assert_eq!(
            gallery_image("https://fr.ugreen.com/a.png"),
            "https://fr.ugreen.com/a.png"
        );
    }
}
