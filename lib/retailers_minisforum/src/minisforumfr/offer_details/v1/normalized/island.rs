//! Reduces the schema.org islands to the one that describes the product, in the shape the record
//! is checked against. Nothing here crosses into the normalized document but `category`. The
//! reviews app's `Product` island is skipped: it restates one offer of the record under a fixed
//! `UsedCondition`, and carries reviews nothing here uses.

use super::super::raw::ldjson::{self, LdJson, LdJsonScript, Product};
use super::NormalizeError;

/// The storefront's own `ProductGroup` or `Product` island.
pub(super) struct Island {
    /// `productGroupID`; a single `Product` island states none.
    pub product_id: Option<String>,
    pub url: String,
    pub category: String,
    pub offers: Vec<IslandOffer>,
}

pub(super) struct IslandOffer {
    /// The offer's `@id`, to name it in errors.
    pub id: String,
    pub variant_id: u64,
    /// The price in hundredths, as Liquid renders it.
    pub cents: i64,
    pub currency: String,
    pub sku: Option<String>,
    pub in_stock: bool,
}

impl Island {
    /// `None` where the page renders no storefront product island, as it does for a product
    /// without a price (2 of the 118 saved pages, one per language).
    pub(super) fn from_raw(scripts: Vec<LdJsonScript>) -> Result<Option<Island>, NormalizeError> {
        let mut islands: Vec<Island> = scripts
            .into_iter()
            .filter_map(|script| match script.data {
                LdJson::ProductGroup(group) => Some(Island::from_group(group)),
                LdJson::Product(Product::Native(product)) => Some(Island::from_product(*product)),
                LdJson::Product(Product::Reviewed(_)) | LdJson::Organization(_) => None,
            })
            .collect::<Result<_, _>>()?;

        match islands.len() {
            0 | 1 => Ok(islands.pop()),
            count => Err(NormalizeError::SeveralProductIslands { count }),
        }
    }

    fn from_group(group: ldjson::ProductGroup) -> Result<Island, NormalizeError> {
        Ok(Island {
            product_id: Some(group.product_group_id),
            url: group.url,
            category: group.category,
            offers: group
                .has_variant
                .into_iter()
                .map(|variant| IslandOffer::from_raw(variant.offers, variant.sku))
                .collect::<Result<_, _>>()?,
        })
    }

    fn from_product(product: ldjson::NativeProduct) -> Result<Island, NormalizeError> {
        Ok(Island {
            product_id: None,
            url: product.url,
            category: product.category,
            offers: vec![IslandOffer::from_raw(product.offers, product.sku)?],
        })
    }
}

impl IslandOffer {
    fn from_raw(offer: ldjson::Offer, sku: Option<String>) -> Result<IslandOffer, NormalizeError> {
        Ok(IslandOffer {
            variant_id: variant_id(&offer.url)?,
            cents: cents(&offer.price).ok_or_else(|| NormalizeError::OfferPriceNotDecimal {
                offer: offer.id.clone(),
                price: offer.price.clone(),
            })?,
            currency: offer.price_currency,
            sku,
            in_stock: offer.availability.ends_with("/InStock"),
            id: offer.id,
        })
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

// Read exactly, without a float: `1142.90` is 114290 hundredths, `1142.9` too.
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
    use super::{cents, variant_id};

    #[test]
    fn a_decimal_price_is_read_exactly() {
        assert_eq!(cents("1142.90"), Some(114290));
        assert_eq!(cents("1142.9"), Some(114290));
        assert_eq!(cents("1142"), Some(114200));
        assert_eq!(cents("0.00"), Some(0));
        assert_eq!(cents("6.399"), None);
        assert_eq!(cents("-1.00"), None);
        assert_eq!(cents(""), None);
    }

    #[test]
    fn the_variant_is_read_from_the_offer_url() {
        assert_eq!(
            variant_id("https://minisforumpc.fr/products/a?variant=42").ok(),
            Some(42)
        );
        assert!(variant_id("https://minisforumpc.fr/products/a").is_err());
    }
}
