//! Reduces the ld+json islands to the theme's product island, in the shape the analytics
//! variants are checked against. The `Organization` and Judge.me's review island are recognised
//! and left behind.

use super::super::raw::ldjson::{self, LdJson, LdJsonScript, ProductIsland};
use super::NormalizeError;

/// The `class` Judge.me's review island carries; the theme's islands carry none.
const REVIEWS_CLASS: &str = "jdgm-server-jld";

/// The page's theme `ProductGroup` or `Product` island.
pub(super) struct Island {
    /// `productGroupID`; a single `Product` island states none.
    pub product_id: Option<String>,
    pub url: String,
    pub title: String,
    /// The Google product category; empty on 55 of the 306 saved pages.
    pub category: String,
    pub offers: Vec<IslandOffer>,
}

pub(super) struct IslandOffer {
    /// The offer's `@id`, to name it in errors.
    pub id: String,
    pub variant_id: u64,
    /// The Shopify variant price in hundredths, as Liquid renders it.
    pub cents: i64,
    pub currency: String,
    pub sku: Option<String>,
    /// As rendered, possibly with a trailing no-break space.
    pub gtin: Option<String>,
    pub in_stock: bool,
}

impl Island {
    pub(super) fn from_raw(scripts: Vec<LdJsonScript>) -> Result<Island, NormalizeError> {
        let mut islands = Vec::new();
        for script in scripts {
            let reviews = script.class.as_deref() == Some(REVIEWS_CLASS);
            let island = match script.data {
                LdJson::Organization(_) => None,
                LdJson::Product(ProductIsland::Reviews(_)) if reviews => None,
                LdJson::Product(ProductIsland::Offer(product)) if script.class.is_none() => {
                    Some(Island::from_product(*product)?)
                }
                LdJson::ProductGroup(group) if script.class.is_none() => {
                    Some(Island::from_group(group)?)
                }
                LdJson::Product(ProductIsland::Reviews(_)) => {
                    return Err(island_class_mismatch("review", script.class));
                }
                LdJson::Product(ProductIsland::Offer(_)) | LdJson::ProductGroup(_) => {
                    return Err(island_class_mismatch("product", script.class));
                }
            };
            islands.extend(island);
        }

        match islands.len() {
            1 => islands.pop().ok_or(NormalizeError::MissingProductIsland),
            0 => Err(NormalizeError::MissingProductIsland),
            count => Err(NormalizeError::SeveralProductIslands { count }),
        }
    }

    fn from_group(group: ldjson::ProductGroup) -> Result<Island, NormalizeError> {
        Ok(Island {
            product_id: Some(group.product_group_id),
            url: group.url,
            title: group.name,
            category: group.category,
            offers: group
                .has_variant
                .into_iter()
                .map(|variant| IslandOffer::from_raw(variant.offers, variant.sku, variant.gtin))
                .collect::<Result<_, _>>()?,
        })
    }

    fn from_product(product: ldjson::Product) -> Result<Island, NormalizeError> {
        Ok(Island {
            product_id: None,
            url: product.url,
            title: product.name,
            category: product.category,
            offers: vec![IslandOffer::from_raw(
                product.offers,
                Some(product.sku),
                product.gtin,
            )?],
        })
    }
}

fn island_class_mismatch(island: &str, class: Option<String>) -> NormalizeError {
    NormalizeError::IslandClassMismatch {
        island: island.to_string(),
        class,
    }
}

impl IslandOffer {
    fn from_raw(
        offer: ldjson::Offer,
        sku: Option<String>,
        gtin: Option<String>,
    ) -> Result<IslandOffer, NormalizeError> {
        Ok(IslandOffer {
            variant_id: variant_id(&offer.url)?,
            cents: cents(&offer.price).ok_or_else(|| NormalizeError::OfferPriceNotDecimal {
                offer: offer.id.clone(),
                price: offer.price.clone(),
            })?,
            in_stock: in_stock(&offer.availability).ok_or_else(|| {
                NormalizeError::UnknownAvailability {
                    offer: offer.id.clone(),
                    availability: offer.availability.clone(),
                }
            })?,
            currency: offer.price_currency,
            sku: sku.filter(|sku| !sku.is_empty()),
            gtin,
            id: offer.id,
        })
    }
}

// Only these two appear, on 437 and 161 of the 598 saved variants.
fn in_stock(availability: &str) -> Option<bool> {
    match availability {
        "http://schema.org/InStock" => Some(true),
        "http://schema.org/OutOfStock" => Some(false),
        _ => None,
    }
}

fn variant_id(url: &str) -> Result<u64, NormalizeError> {
    url.split_once('?')
        .into_iter()
        .flat_map(|(_, query)| query.split('&'))
        .find_map(|pair| pair.strip_prefix("variant="))
        .and_then(|id| id.parse().ok())
        .ok_or_else(|| NormalizeError::OfferUrlWithoutVariant {
            url: url.to_string(),
        })
}

// Read exactly, without a float: `134.99` is 13499 hundredths, `134.9` 13490.
fn cents(price: &str) -> Option<i64> {
    let (units, fraction) = price.split_once('.').unwrap_or((price, ""));
    if units.is_empty()
        || fraction.len() > 2
        || !units
            .bytes()
            .chain(fraction.bytes())
            .all(|b| b.is_ascii_digit())
    {
        return None;
    }

    let fraction: i64 = format!("{fraction:0<2}").parse().ok()?;
    units
        .parse::<i64>()
        .ok()?
        .checked_mul(100)?
        .checked_add(fraction)
}

#[cfg(test)]
mod tests {
    use super::{cents, in_stock, variant_id};

    #[test]
    fn a_decimal_price_is_read_exactly() {
        assert_eq!(cents("134.99"), Some(13499));
        assert_eq!(cents("134.9"), Some(13490));
        assert_eq!(cents("134"), Some(13400));
        assert_eq!(cents("0.98"), Some(98));
        assert_eq!(cents("43,99"), None);
        assert_eq!(cents("6.399"), None);
        assert_eq!(cents("-1.00"), None);
        assert_eq!(cents(""), None);
    }

    #[test]
    fn the_variant_is_read_from_the_offer_url() {
        assert_eq!(
            variant_id("https://fr.ugreen.com/products/45699?variant=42").ok(),
            Some(42)
        );
        assert!(variant_id("https://fr.ugreen.com/products/45699").is_err());
        assert!(variant_id("https://fr.ugreen.com/products/45699?variant=x").is_err());
    }

    #[test]
    fn only_the_two_seen_availabilities_are_read() {
        assert_eq!(in_stock("http://schema.org/InStock"), Some(true));
        assert_eq!(in_stock("http://schema.org/OutOfStock"), Some(false));
        assert_eq!(in_stock("http://schema.org/PreOrder"), None);
    }
}
