use super::super::raw;
use super::super::raw::ldjson::{self, LdJson};
use super::super::raw::section::PickerVariant;
use super::{
    NormalizeError, ORIGIN, https_from_protocol_relative, product_gid, variant_gid,
    whole_yen_from_float, whole_yen_from_shopify_cents, whole_yen_from_text, yen, yen_currency,
};
use money::Money;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Product {
    /// The analytics `gid`, checked against the analytics `id`.
    pub id: String,
    /// The analytics handle, raw: 15 saved handles hold Japanese text.
    pub handle: String,
    /// The island's `url`: the handle's product page, the handle percent-encoded.
    pub url: String,
    /// The island's `name`.
    pub title: String,
    /// The island's plain-text description, with its blank lines and each line's surrounding
    /// whitespace dropped. `None` where the product has none (5 of the 132 saved pages).
    pub description: Option<String>,
    /// The analytics `vendor`: `UGREEN-JP`, `株式会社ユーグリーン・ジャパン` or `UGREEN`.
    pub vendor: String,
    // Commented out: the analytics `vendor` again, on every page.
    // pub brand: String,
    /// `None` where the product is untyped (130 of the 132 saved pages).
    pub product_type: Option<String>,
    // Commented out: the selected variant's SKU, which its variant carries.
    // pub sku: Option<String>,
    /// The gallery's slides, made https and without the display `width` the theme asks the CDN
    /// for. The island's single `image` is the first of them.
    pub images: Vec<String>,
    // Commented out: false on every page.
    // pub remote: bool,
    // Commented out: the analytics `page` describes this request, not the product.
    // pub page: Page,
    // Commented out: the `Organization` island and the PayPal tag describe the shop; the tag's
    // currency is checked to be JPY.
    // Commented out: the slides' `alt` texts are empty on every page, and their ids name the
    // section as well as the media.
    // pub media: Vec<MediaSlide>,
    pub variants: Vec<Variant>,
}

impl Product {
    pub(crate) fn from_raw(raw: raw::UgreenJpRawDocument) -> Result<Product, NormalizeError> {
        let mut islands: Vec<ldjson::Product> = raw
            .ld_json
            .into_iter()
            .filter_map(|script| match script.data {
                LdJson::Product(product) => Some(product),
                LdJson::Organization(_) => None,
            })
            .collect();
        let island = match islands.len() {
            0 => return Err(NormalizeError::MissingProductIsland),
            1 => islands.remove(0),
            count => return Err(NormalizeError::MultipleProductIslands { count }),
        };

        let meta = raw.analytics.meta.product;
        if meta.gid != product_gid(meta.id) {
            return Err(NormalizeError::ProductGidMismatch {
                gid: meta.gid,
                id: meta.id,
            });
        }
        if island.url != format!("{ORIGIN}/products/{}", percent_encoded(&meta.handle)) {
            return Err(NormalizeError::ProductUrlMismatch {
                url: island.url,
                handle: meta.handle,
            });
        }

        let analytics: Vec<u64> = meta.variants.iter().map(|variant| variant.id).collect();
        let offers: Vec<u64> = island
            .offers
            .iter()
            .map(|offer| offer_variant_id(&island.url, &offer.url))
            .collect::<Result<_, _>>()?;
        if analytics != offers {
            return Err(NormalizeError::VariantIdsMismatch {
                island: "the offers".to_string(),
                analytics,
                other: offers,
            });
        }

        let selected = raw
            .selected_variant_id
            .value
            .parse::<u64>()
            .ok()
            .filter(|id| analytics.contains(id))
            .ok_or_else(|| NormalizeError::SelectedVariantNotOffered {
                value: raw.selected_variant_id.value.clone(),
            })?;

        // The picker states each variant's availability and compare-at price. A `Default Title`
        // product renders none, and its only compare-at price is the rendered price block's,
        // which is real only under `price--on-sale`: off sale its `<s>` is empty, repeats the
        // price or reads `¥0`. Two saved NAS pages are on sale this way (¥52,700 against
        // ¥65,880, ¥29,500 against ¥36,880).
        let (pickers, rendered_compare_at): (Vec<Option<PickerVariant>>, Option<i64>) =
            match raw.variants {
                Some(script) => {
                    let picker: Vec<u64> = script.data.iter().map(|variant| variant.id).collect();
                    if picker != analytics {
                        return Err(NormalizeError::VariantIdsMismatch {
                            island: "the picker".to_string(),
                            analytics,
                            other: picker,
                        });
                    }
                    (script.data.into_iter().map(Some).collect(), None)
                }
                None if is_default_title(&meta.variants) => {
                    let on_sale = raw
                        .price
                        .class
                        .split_whitespace()
                        .any(|class| class == "price--on-sale");
                    let compare_at = if on_sale {
                        Some(whole_yen_from_text(
                            &raw.price.compare_at.text,
                            "price.compare_at",
                        )?)
                    } else {
                        None
                    };
                    (vec![None], compare_at)
                }
                None => {
                    return Err(NormalizeError::MissingVariantPicker {
                        count: meta.variants.len(),
                    });
                }
            };

        let rendered = whole_yen_from_text(&raw.price.sale.text, "price.sale")?;
        let variants = meta
            .variants
            .into_iter()
            .zip(island.offers)
            .zip(pickers)
            .map(|((variant, offer), picker)| {
                let id = variant.id;
                let (variant, whole_yen) =
                    Variant::from_raw(variant, offer, picker, rendered_compare_at)?;
                if id == selected && whole_yen != rendered {
                    return Err(NormalizeError::RenderedPriceMismatch {
                        rendered: raw.price.sale.text.clone(),
                        price: whole_yen,
                    });
                }
                Ok(variant)
            })
            .collect::<Result<_, _>>()?;

        Ok(Product {
            id: meta.gid,
            handle: meta.handle,
            url: island.url,
            title: island.name,
            description: text_lines(&island.description),
            vendor: meta.vendor,
            // brand: island.brand.name,
            product_type: non_empty(meta.product_type),
            // sku: island.sku,
            images: raw
                .media
                .iter()
                .map(|slide| without_width(&https_from_protocol_relative(&slide.src)))
                .collect(),
            variants,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Variant {
    pub id: String,
    /// The analytics `public_title`, the selected values joined with ` / `. `None` on a
    /// `Default Title` product.
    pub title: Option<String>,
    /// The product title and `title`, joined with ` - `; the product title alone on a
    /// `Default Title` product.
    pub name: String,
    /// `None` on the 19 saved variants that have no SKU anywhere on the page.
    pub sku: Option<String>,
    /// The picker's `barcode` where it is all digits once trimmed. The barcode is `null` on every
    /// saved variant, so no barcode is rejected; a `Default Title` product renders none.
    pub gtin: Option<String>,
    /// The offer's whole yen; the analytics and the picker state the same in hundredths. `None`
    /// where it is ¥0, Shopify's price for a product announced but not sold. No saved variant is.
    pub price: Option<Money>,
    /// The picker's `compare_at_price`, or on a `Default Title` product the rendered on-sale
    /// compare-at price. `None` where it is missing, `0`, or not above the price (on 9 picker
    /// variants it is `0` or the price itself).
    pub compare_at_price: Option<Money>,
    /// The offer's availability, checked against the picker's `available`.
    pub available: bool,
    /// The picker's option values, one per product option; empty on a `Default Title` product.
    /// The page names the options only in the picker's labels, which are not read.
    pub options: Vec<String>,
    /// The picker's `featured_image`, made https. `None` where the variant has no image of its
    /// own (82 of the 100 picker variants) and on a `Default Title` product.
    pub image: Option<String>,
    // Commented out: the picker's `option1` to `option3` restate `options`, its `title` and
    // `public_title` the analytics `public_title`.
    // pub option1: Option<String>,
    // Commented out: picker-only shipping facts, false or 0 on almost every variant and missing
    // on `Default Title` products.
    // pub requires_shipping: bool,
    // pub taxable: bool,
    // pub weight: i64,
    // Commented out: `null` on every variant, so stock is not tracked.
    // pub inventory_management: Option<String>,
    // Commented out: one unit, no maximum, in steps of one, on every variant.
    // pub quantity_rule: QuantityRule,
    // Commented out: no selling plans on any page.
    // pub selling_plan_allocations: Vec<Value>,
    // Commented out: the offer URL; the variation URL is built from the product URL instead.
    // pub url: String,
}

impl Variant {
    /// The variant, and its price in whole yen for the check against the price block.
    fn from_raw(
        variant: raw::analytics::Variant,
        offer: ldjson::Offer,
        picker: Option<PickerVariant>,
        rendered_compare_at: Option<i64>,
    ) -> Result<(Variant, i64), NormalizeError> {
        let id = variant.id;
        let mismatch = |field: &str, island: &str, analytics: String, other: String| {
            NormalizeError::VariantFieldMismatch {
                variant_id: id,
                field: field.to_string(),
                island: island.to_string(),
                analytics,
                other,
            }
        };

        yen_currency(&offer.price_currency, "offers.priceCurrency")?;
        let price = whole_yen_from_float(offer.price, "offers.price")?;
        let analytics_price = whole_yen_from_shopify_cents(variant.price, "meta.variants.price")?;
        if analytics_price != price {
            return Err(mismatch(
                "price",
                "its offer",
                analytics_price.to_string(),
                price.to_string(),
            ));
        }

        let sku = variant.sku.and_then(non_empty);
        let offer_sku = offer.sku.and_then(non_empty);
        if offer_sku != sku {
            return Err(mismatch(
                "sku",
                "its offer",
                format!("{sku:?}"),
                format!("{offer_sku:?}"),
            ));
        }

        let available = in_stock(&offer.availability)?;

        let (compare_at, gtin, options, image) = match picker {
            Some(picker) => {
                let picker_price = whole_yen_from_shopify_cents(picker.price, "variants.price")?;
                if picker_price != price {
                    return Err(mismatch(
                        "price",
                        "the picker",
                        analytics_price.to_string(),
                        picker_price.to_string(),
                    ));
                }
                let picker_sku = picker.sku.and_then(non_empty);
                if picker_sku != sku {
                    return Err(mismatch(
                        "sku",
                        "the picker",
                        format!("{sku:?}"),
                        format!("{picker_sku:?}"),
                    ));
                }
                if picker.available != available {
                    return Err(mismatch(
                        "available",
                        "the picker",
                        available.to_string(),
                        picker.available.to_string(),
                    ));
                }
                let compare_at = picker
                    .compare_at_price
                    .map(|cents| whole_yen_from_shopify_cents(cents, "variants.compare_at_price"))
                    .transpose()?;
                (
                    compare_at,
                    picker.barcode.and_then(gtin_from_barcode),
                    picker.options,
                    picker
                        .featured_image
                        .map(|image| https_from_protocol_relative(&image.src)),
                )
            }
            None => (rendered_compare_at, None, Vec::new(), None),
        };

        let variant = Variant {
            id: variant_gid(variant.id),
            title: variant.public_title,
            name: variant.name,
            sku,
            gtin,
            price: unsold_placeholder_price(price).map(yen),
            compare_at_price: compare_at
                .filter(|compare_at| price > 0 && *compare_at > price)
                .map(yen),
            available,
            options,
            image,
        };

        Ok((variant, price))
    }
}

// Shopify's analytics list a product with no options as one variant without a public title.
fn is_default_title(variants: &[raw::analytics::Variant]) -> bool {
    matches!(variants, [variant] if variant.public_title.is_none())
}

fn in_stock(availability: &str) -> Result<bool, NormalizeError> {
    match availability {
        "http://schema.org/InStock" => Ok(true),
        "http://schema.org/OutOfStock" => Ok(false),
        other => Err(NormalizeError::UnknownAvailability {
            value: other.to_string(),
        }),
    }
}

// An offer names its variant only in its URL: the product URL with `?variant=<id>`.
fn offer_variant_id(product_url: &str, offer_url: &str) -> Result<u64, NormalizeError> {
    offer_url
        .strip_prefix(product_url)
        .and_then(|rest| rest.strip_prefix("?variant="))
        .and_then(|id| id.parse::<u64>().ok())
        .ok_or_else(|| NormalizeError::OfferUrlWithoutVariant {
            url: offer_url.to_string(),
        })
}

// The island's URL percent-encodes the handle's UTF-8 bytes in upper-case hex, keeping RFC 3986's
// unreserved characters: `急速` is `%E6%80%A5%E9%80%9F`.
fn percent_encoded(handle: &str) -> String {
    handle
        .bytes()
        .map(|byte| {
            if byte.is_ascii_alphanumeric() || b"-._~".contains(&byte) {
                char::from(byte).to_string()
            } else {
                format!("%{byte:02X}")
            }
        })
        .collect()
}

// The gallery asks the CDN for `.../a.png?v=1&width=1946`; the image is the file at its version,
// and the width only how large the theme displays it.
fn without_width(src: &str) -> String {
    let Some((path, query)) = src.split_once('?') else {
        return src.to_string();
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

// A barcode is a GTIN only if it is all digits once surrounding whitespace, `\u{a0}` included,
// is trimmed; anything else is not one, and the page still normalizes.
fn gtin_from_barcode(barcode: String) -> Option<String> {
    let trimmed = barcode.trim();
    (!trimmed.is_empty() && trimmed.bytes().all(|b| b.is_ascii_digit()))
        .then(|| trimmed.to_string())
}

// ¥0 is not a price the storefront sells at.
fn unsold_placeholder_price(whole_yen: i64) -> Option<i64> {
    (whole_yen != 0).then_some(whole_yen)
}

fn text_lines(description: &str) -> Option<String> {
    let lines: Vec<&str> = description
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect();
    (!lines.is_empty()).then(|| lines.join("\n"))
}

fn non_empty(value: String) -> Option<String> {
    (!value.is_empty()).then_some(value)
}

#[cfg(test)]
mod tests {
    use super::{gtin_from_barcode, offer_variant_id, percent_encoded, text_lines, without_width};

    #[test]
    fn only_the_width_parameter_is_dropped() {
        assert_eq!(
            without_width("https://ugreen.jp/cdn/shop/files/a.png?v=1&width=1946"),
            "https://ugreen.jp/cdn/shop/files/a.png?v=1"
        );
        assert_eq!(
            without_width("https://ugreen.jp/a.png?width=1946"),
            "https://ugreen.jp/a.png"
        );
        assert_eq!(
            without_width("https://ugreen.jp/a.png"),
            "https://ugreen.jp/a.png"
        );
    }

    #[test]
    fn a_blank_description_is_absent() {
        assert_eq!(text_lines(""), None);
        assert_eq!(text_lines("\n \n\t\n"), None);
        assert_eq!(text_lines("\n a \n\nb"), Some("a\nb".to_string()));
    }

    #[test]
    fn an_offer_names_its_variant_in_the_query() {
        let product = "https://ugreen.jp/products/a";
        assert_eq!(
            offer_variant_id(product, &format!("{product}?variant=42")),
            Ok(42)
        );
        assert!(offer_variant_id(product, &format!("{product}?variant=x")).is_err());
        assert!(offer_variant_id(product, "https://ugreen.jp/products/b?variant=42").is_err());
    }

    #[test]
    fn a_handle_is_percent_encoded_as_the_island_does() {
        assert_eq!(
            percent_encoded("lightning-usb-cable"),
            "lightning-usb-cable"
        );
        assert_eq!(
            percent_encoded("ugreen-モバイルバッテリー-20000mah"),
            "ugreen-%E3%83%A2%E3%83%90%E3%82%A4%E3%83%AB%E3%83%90%E3%83%83%E3%83%86%E3%83%AA%E3%83%BC-20000mah"
        );
    }

    #[test]
    fn only_a_digit_barcode_is_a_gtin() {
        assert_eq!(
            gtin_from_barcode("6957303860521".to_string()),
            Some("6957303860521".to_string())
        );
        assert_eq!(
            gtin_from_barcode("\u{a0}6957303860521 \t".to_string()),
            Some("6957303860521".to_string())
        );
        for barcode in ["", " ", "\u{a0}", "60521-B", "6957 303860521", "SKU60521"] {
            assert_eq!(gtin_from_barcode(barcode.to_string()), None, "{barcode:?}");
        }
    }
}
