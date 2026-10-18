use super::super::raw::{analytics, media::MediaItem, shopnow::ShopNow};
use super::{NormalizeError, ORIGIN, money_from_shopify_cents, product_gid, variant_gid};
use money::{Currency, Money};
use serde::{Deserialize, Serialize};

/// The facts the page states outside the analytics record.
pub(super) struct Page {
    /// `og:title`.
    pub title: String,
    /// `link[rel="canonical"]`.
    pub canonical: String,
    pub shop_now: Option<ShopNow>,
    pub media: Vec<MediaItem>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Product {
    pub id: String,
    pub handle: String,
    /// The canonical URL, checked to be the handle's page on the brand site.
    pub url: String,
    /// `og:title`, verbatim: one title spaces its words with `\u{a0}`.
    pub title: String,
    /// Free text: `UGREEN Brandsite` on 65 pages, `UGREEN Official Site` on one.
    pub vendor: String,
    /// `None` where the product is untyped, as all 66 saved products are.
    pub product_type: Option<String>,
    // Commented out: false on every page.
    // pub remote: bool,
    /// The main gallery, 1 to 7 images per page, as https URLs without the display width.
    pub images: Vec<String>,
    // Commented out: the gallery's media ids and alt texts, which nothing reads.
    // pub media: Vec<MediaItem>,
    /// Where the product is sold: the regional stores (`us.ugreen.com`, `ai.ugreen.com`) the
    /// "Shop Now" button links to, one per page. Empty on `products-us-85185.html`, which has no
    /// button.
    pub sales_channel_urls: Vec<String>,
    // Commented out from the button: `data-market` (`USA` throughout), its tracking attributes,
    // and `data-product-sku`, checked against the variants instead.
    /// Exactly one on all 66 saved pages.
    pub variants: Vec<Variant>,
}

impl Product {
    pub(super) fn from_raw(
        meta: analytics::Meta,
        page: Page,
        currency: Currency,
    ) -> Result<Product, NormalizeError> {
        let record = meta.product;

        if record.gid != product_gid(record.id) {
            return Err(NormalizeError::ProductGidMismatch {
                gid: record.gid,
                id: record.id,
            });
        }
        if meta.page.resource_type != "product" || meta.page.resource_id != record.id {
            return Err(NormalizeError::PageResourceMismatch {
                resource_type: meta.page.resource_type,
                resource_id: meta.page.resource_id,
                product: record.id,
            });
        }
        if page.canonical != format!("{ORIGIN}/products/{}", record.handle) {
            return Err(NormalizeError::CanonicalUrlMismatch {
                url: page.canonical,
                handle: record.handle,
            });
        }

        let variants: Vec<Variant> = record
            .variants
            .into_iter()
            .map(|variant| Variant::from_raw(variant, &page.title, currency))
            .collect::<Result<_, _>>()?;

        let sales_channel_urls = match page.shop_now {
            Some(button) => {
                let sku = Some(button.product_sku.as_str());
                if !variants.iter().any(|variant| variant.sku.as_deref() == sku) {
                    return Err(NormalizeError::ShopNowSkuNotListed {
                        sku: button.product_sku,
                    });
                }
                button.channels.channel_url_list
            }
            None => Vec::new(),
        };

        Ok(Product {
            id: product_gid(record.id),
            handle: record.handle,
            url: page.canonical,
            title: page.title,
            vendor: record.vendor,
            product_type: non_empty(record.product_type),
            // remote: record.remote,
            images: page.media.into_iter().map(gallery_image).collect(),
            // media: page.media,
            sales_channel_urls,
            variants,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Variant {
    pub id: String,
    /// The product title, followed by ` - ` and the option values where the variant has any.
    pub name: String,
    // Commented out: null on every saved variant, each its product's only one.
    // pub public_title: Option<String>,
    pub sku: Option<String>,
    /// The analytics price, in hundredths of the PayPal currency. On this site it is a
    /// placeholder, not an offer: USD 1,300.00 on 65 pages and USD 5.00 on one, while the
    /// product sells on a regional store at its own price. The user decided to keep priced
    /// offers anyway, so it crosses as the price. `None` only where it is 0, which no saved page
    /// renders.
    pub price: Option<Money>,
    // Not on the page: a barcode or GTIN, a compare-at price and the availability.
}

impl Variant {
    fn from_raw(
        raw: analytics::Variant,
        title: &str,
        currency: Currency,
    ) -> Result<Variant, NormalizeError> {
        // A variant without option values is named by the product title alone: on every saved
        // page the analytics name and `og:title` agree, so a disagreement means one was misread.
        if raw.public_title.is_none() && raw.name != title {
            return Err(NormalizeError::TitleMismatch {
                variant_id: raw.id,
                name: raw.name,
                title: title.to_string(),
            });
        }

        Ok(Variant {
            id: variant_gid(raw.id),
            name: raw.name,
            // public_title: raw.public_title,
            sku: raw.sku.and_then(non_empty),
            price: unsold_placeholder_price(raw.price)
                .map(|cents| money_from_shopify_cents(cents, currency)),
        })
    }
}

// Shopify's $0 is a product announced but not sold. No saved page renders it; the 130000 and 500
// placeholders are kept as prices.
fn unsold_placeholder_price(cents: i64) -> Option<i64> {
    (cents != 0).then_some(cents)
}

// The gallery asks the CDN for `//www.ugreen.com/cdn/shop/files/a.png?v=1&width=1445`; the image
// is the file at its version, and the width only how large the theme displays it.
fn gallery_image(item: MediaItem) -> String {
    let MediaItem::Image { src, .. } = item;
    let url = match src.strip_prefix("//") {
        Some(rest) => format!("https://{rest}"),
        None => src,
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
    use super::gallery_image;
    use crate::ugreencom::offer_details::v1::raw::media::MediaItem;

    fn image(src: &str) -> MediaItem {
        MediaItem::Image {
            id: "1".to_string(),
            src: src.to_string(),
            alt: String::new(),
        }
    }

    #[test]
    fn a_gallery_image_is_https_without_its_display_width() {
        assert_eq!(
            gallery_image(image(
                "//www.ugreen.com/cdn/shop/files/a.png?v=1762322082&width=1445"
            )),
            "https://www.ugreen.com/cdn/shop/files/a.png?v=1762322082"
        );
        assert_eq!(
            gallery_image(image("https://www.ugreen.com/a.png?width=1445")),
            "https://www.ugreen.com/a.png"
        );
        assert_eq!(
            gallery_image(image("https://www.ugreen.com/a.png")),
            "https://www.ugreen.com/a.png"
        );
    }
}
