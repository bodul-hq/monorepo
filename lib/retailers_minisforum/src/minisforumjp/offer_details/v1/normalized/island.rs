//! Reduces the schema.org islands to the one that describes the product, in the shape the record
//! is checked against. Nothing here crosses into the normalized document but `category`.
//!
//! The Organization and FAQ islands, and the reviews app's `Product` island (see
//! [`ldjson::ReviewsProduct`]), say nothing about the offer and are skipped.

use super::super::raw::ldjson::{self, LdJson, LdJsonScript};
use super::{NormalizeError, non_empty};
use money::Currency;

/// The page's `ProductGroup` or theme `Product` island.
pub(super) struct Island {
    /// `productGroupID`; a single `Product` island states none.
    pub product_id: Option<String>,
    pub url: String,
    /// `None` where the island states an empty category, as the one single-`Product` island
    /// does.
    pub category: Option<String>,
    pub offers: Vec<IslandOffer>,
}

pub(super) struct IslandOffer {
    /// The offer's `@id`, to name it in errors.
    pub id: String,
    pub variant_id: u64,
    /// The price as the island states it: a decimal string in major units.
    pub price: String,
    pub currency: String,
    pub sku: Option<String>,
    pub in_stock: bool,
}

impl Island {
    /// `None` where the page renders no product island, as it does for a product without a price
    /// and for the one with 98 variants.
    pub(super) fn from_raw(scripts: Vec<LdJsonScript>) -> Result<Option<Island>, NormalizeError> {
        let mut islands: Vec<Island> = scripts
            .into_iter()
            .filter_map(|script| match script.data {
                LdJson::ProductGroup(group) => Some(Island::from_group(group)),
                LdJson::Product(ldjson::Product::Theme(product)) => {
                    Some(Island::from_product(*product))
                }
                LdJson::Product(ldjson::Product::Reviews(_))
                | LdJson::Organization(_)
                | LdJson::FaqPage(_) => None,
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
            category: non_empty(group.category),
            offers: group
                .has_variant
                .into_iter()
                .map(|variant| IslandOffer::from_raw(variant.offers, variant.sku))
                .collect::<Result<_, _>>()?,
        })
    }

    fn from_product(product: ldjson::ThemeProduct) -> Result<Island, NormalizeError> {
        Ok(Island {
            product_id: None,
            url: product.url,
            category: non_empty(product.category),
            offers: vec![IslandOffer::from_raw(product.offers, product.sku)?],
        })
    }
}

impl IslandOffer {
    fn from_raw(offer: ldjson::Offer, sku: Option<String>) -> Result<IslandOffer, NormalizeError> {
        let in_stock = match offer.availability.as_str() {
            "http://schema.org/InStock" => true,
            "http://schema.org/OutOfStock" => false,
            _ => {
                return Err(NormalizeError::UnknownAvailability {
                    offer: offer.id,
                    value: offer.availability,
                });
            }
        };

        Ok(IslandOffer {
            variant_id: variant_id(&offer.url)?,
            price: offer.price,
            currency: offer.price_currency,
            sku,
            in_stock,
            id: offer.id,
        })
    }

    /// The offer's price in minor units of `currency`: yen as whole yen, an `AUD`-like currency
    /// in hundredths.
    pub(super) fn minor_units(&self, currency: Currency) -> Result<i64, NormalizeError> {
        minor_units(&self.price, currency.exponent()).ok_or_else(|| {
            NormalizeError::OfferPriceNotDecimal {
                offer: self.id.clone(),
                price: self.price.clone(),
                currency: currency.as_str().to_string(),
            }
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

// Read exactly, without a float, and no finer than the currency's minor unit: with an exponent of
// 2, `1142.90` and `1142.9` are 114290; with 0, `130399` is 130399 and `130399.0` is not a price.
fn minor_units(price: &str, exponent: u8) -> Option<i64> {
    let exponent = usize::from(exponent);
    let (units, fraction) = price.split_once('.').unwrap_or((price, ""));
    if units.is_empty()
        || fraction.len() > exponent
        || (price.contains('.') && fraction.is_empty())
        || !units
            .bytes()
            .chain(fraction.bytes())
            .all(|b| b.is_ascii_digit())
    {
        return None;
    }

    let fraction: i64 = if exponent == 0 {
        0
    } else {
        format!("{fraction:0<exponent$}").parse().ok()?
    };
    units
        .parse::<i64>()
        .ok()?
        .checked_mul(10_i64.pow(u32::try_from(exponent).ok()?))?
        .checked_add(fraction)
}

#[cfg(test)]
mod tests {
    use super::{minor_units, variant_id};

    #[test]
    fn a_whole_yen_price_is_read_exactly() {
        assert_eq!(minor_units("130399", 0), Some(130399));
        assert_eq!(minor_units("100", 0), Some(100));
        assert_eq!(minor_units("0", 0), Some(0));
        assert_eq!(minor_units("130399.0", 0), None);
        assert_eq!(minor_units("130399.", 0), None);
        assert_eq!(minor_units("130.399", 0), None);
        assert_eq!(minor_units("130,399", 0), None);
        assert_eq!(minor_units("-1", 0), None);
        assert_eq!(minor_units("", 0), None);
    }

    #[test]
    fn a_decimal_price_is_read_exactly() {
        assert_eq!(minor_units("1142.90", 2), Some(114290));
        assert_eq!(minor_units("1142.9", 2), Some(114290));
        assert_eq!(minor_units("1142", 2), Some(114200));
        assert_eq!(minor_units("6.399", 2), None);
        assert_eq!(minor_units("-1.00", 2), None);
        assert_eq!(minor_units("", 2), None);
    }

    #[test]
    fn the_variant_is_read_from_the_offer_url() {
        assert_eq!(
            variant_id("https://www.minisforum.jp/products/a?variant=42").ok(),
            Some(42)
        );
        assert!(variant_id("https://www.minisforum.jp/products/a").is_err());
    }
}
