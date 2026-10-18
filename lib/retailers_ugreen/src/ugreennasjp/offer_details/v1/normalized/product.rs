use super::super::raw;
use super::super::raw::ldjsonitem::{LdJson, ProductIsland};
use super::{NormalizeError, variant_gid, whole_yen_from_shopify_cents, yen};
use money::Money;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Product {
    /// The analytics `gid`.
    pub id: String,
    pub handle: String,
    /// The island's `url`, which is the handle's product page.
    pub url: String,
    /// The island's `name`.
    pub title: String,
    /// The island's description, a plain-text rendering of the product page, with its blank lines
    /// and each line's surrounding whitespace dropped. It keeps the page's carousel captions, such
    /// as `Previous page`. `None` where the product has none.
    pub description: Option<String>,
    /// The analytics `vendor`, `UGREEN-JP` or `UGREEN NAS JP`.
    pub vendor: String,
    // Commented out: the analytics `vendor` again, on every page.
    // pub brand: String,
    /// `None` where the product is untyped.
    pub product_type: Option<String>,
    // Commented out: the first offer's `sku` on every page.
    // pub sku: String,
    /// The island's images, without the display `width` the theme asks the CDN for. One per page
    /// so far: the featured image.
    pub images: Vec<String>,
    /// judge.me's aggregate rating. `None` where the product has no reviews.
    pub rating: Option<Rating>,
    // Commented out: judge.me's reviews, by customer, nothing reads them.
    // pub reviews: Vec<Review>,
    // Commented out: false on every page.
    // pub remote: bool,
    // Commented out: the analytics `page` describes this request, not the product.
    // pub page: Page,
    // Commented out: the `Organization`, `WebPage` and `WebSite` islands describe the shop and the
    // page; the `WebPage` restates the island's name and URL.
    pub variants: Vec<Variant>,
}

impl Product {
    pub(crate) fn from_raw(raw: raw::UgreenNasJpRawDocument) -> Result<Product, NormalizeError> {
        let mut islands = Vec::new();
        let mut reviews = Vec::new();
        for item in raw.ld_json {
            match item.data {
                LdJson::Product(ProductIsland::Offers(island)) => islands.push(island),
                LdJson::Product(ProductIsland::Reviews(island)) => reviews.push(island),
                LdJson::Organization(_) | LdJson::WebPage(_) | LdJson::WebSite(_) => {}
            }
        }

        let island = match islands.len() {
            0 => return Err(NormalizeError::MissingProductIsland),
            1 => islands.remove(0),
            count => return Err(NormalizeError::MultipleProductIslands { count }),
        };
        let rating = match reviews.len() {
            0 => None,
            1 => Some(Rating::from_raw(reviews.remove(0).aggregate_rating)?),
            count => return Err(NormalizeError::MultipleReviewIslands { count }),
        };

        let meta = raw.analytics.meta.product;
        if island.url != format!("{}/products/{}", origin(&island.url), meta.handle) {
            return Err(NormalizeError::ProductUrlMismatch {
                url: island.url,
                handle: meta.handle,
            });
        }

        let offers: Vec<u64> = island
            .offers
            .iter()
            .map(|offer| offer_variant_id(&island.url, &offer.url))
            .collect::<Result<_, _>>()?;
        let analytics: Vec<u64> = meta.variants.iter().map(|variant| variant.id).collect();
        if analytics != offers {
            return Err(NormalizeError::VariantIdsMismatch { analytics, offers });
        }

        let variants = meta
            .variants
            .into_iter()
            .zip(island.offers)
            .map(|(variant, offer)| Variant::from_raw(variant, offer))
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
            images: island.image.iter().map(|src| without_width(src)).collect(),
            rating,
            variants,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Rating {
    /// Out of 5, the best rating; 1 is the worst.
    pub value: f64,
    pub count: i64,
}

impl Rating {
    fn from_raw(raw: raw::ldjsonitem::AggregateRating) -> Result<Rating, NormalizeError> {
        let value =
            raw.rating_value
                .parse::<f64>()
                .map_err(|_| NormalizeError::RatingNotANumber {
                    value: raw.rating_value.clone(),
                })?;

        Ok(Rating {
            value,
            count: raw.review_count,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Variant {
    pub id: String,
    /// The analytics `public_title`, the selected values joined with ` / `. `None` on a product
    /// with a single variant.
    pub title: Option<String>,
    /// The product title and `title`, joined with ` - `.
    pub name: String,
    pub sku: String,
    /// The offer's `gtin13`, zero-padded to thirteen digits. `None` where the barcode is not
    /// filled in.
    pub gtin: Option<String>,
    /// The offer's price; the analytics state the same in hundredths of a yen.
    pub price: Money,
    pub available: bool,
    pub url: String,
}

impl Variant {
    fn from_raw(
        variant: raw::analytics::Variant,
        offer: raw::ldjsonitem::Offer,
    ) -> Result<Variant, NormalizeError> {
        let price = yen(offer.price, &offer.price_currency, "offers.price")?;
        let analytics = whole_yen_from_shopify_cents(variant.price, "variants.price")?;
        if analytics != price.amount() {
            return Err(NormalizeError::PriceMismatch {
                variant_id: variant.id,
                analytics,
                offer: price.amount(),
            });
        }
        if variant.sku.as_deref() != Some(offer.sku.as_str()) {
            return Err(NormalizeError::SkuMismatch {
                variant_id: variant.id,
                analytics: variant.sku,
                offer: offer.sku,
            });
        }

        Ok(Variant {
            id: variant_gid(variant.id),
            title: variant.public_title,
            name: variant.name,
            sku: offer.sku,
            gtin: offer.gtin13.map(|gtin| format!("{gtin:013}")),
            price,
            available: in_stock(&offer.availability)?,
            url: offer.url,
        })
    }
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

// `https://store.ugreen.jp/products/a` is on `https://store.ugreen.jp`.
fn origin(url: &str) -> &str {
    url.find("/products/").map_or(url, |index| &url[..index])
}

// The island asks the CDN for `.../a.png?v=1&width=1920`; the image is the file at its version,
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
    use super::{offer_variant_id, text_lines, without_width};

    #[test]
    fn only_the_width_parameter_is_dropped() {
        assert_eq!(
            without_width("https://store.ugreen.jp/cdn/shop/files/a.png?v=1&width=1920"),
            "https://store.ugreen.jp/cdn/shop/files/a.png?v=1"
        );
        assert_eq!(
            without_width("https://store.ugreen.jp/a.png?width=1920"),
            "https://store.ugreen.jp/a.png"
        );
        assert_eq!(
            without_width("https://store.ugreen.jp/a.png"),
            "https://store.ugreen.jp/a.png"
        );
    }

    #[test]
    fn a_blank_description_is_absent() {
        assert_eq!(text_lines(""), None);
        assert_eq!(text_lines("\n \n\t\n"), None);
    }

    #[test]
    fn an_offer_names_its_variant_in_the_query() {
        let product = "https://store.ugreen.jp/products/a";

        assert_eq!(
            offer_variant_id(product, &format!("{product}?variant=42")),
            Ok(42)
        );
        assert!(offer_variant_id(product, &format!("{product}?variant=x")).is_err());
        assert!(
            offer_variant_id(product, "https://store.ugreen.jp/products/b?variant=42").is_err()
        );
    }
}
