//! Reduces the schema.org islands to the theme's, the one that describes the product, in the
//! shape the record is checked against. Nothing here crosses into the normalized document but
//! the page's URL, which carries its locale, and `category`.

use super::super::raw::ldjson::{self, LdJson, LdJsonScript, ProductIsland};
use super::NormalizeError;

/// The page's `ProductGroup` or theme `Product` island.
pub(super) struct Island {
    /// `productGroupID`; a single `Product` island states none.
    pub product_id: Option<String>,
    /// The page's own URL.
    pub url: String,
    pub category: String,
    pub offers: Vec<IslandOffer>,
}

pub(super) struct IslandOffer {
    /// The offer's `@id`, to name it in errors.
    pub id: String,
    pub variant_id: u64,
    /// The price in hundredths (paise), as Liquid renders it.
    pub cents: i64,
    pub currency: String,
    pub sku: String,
    pub gtin: Option<String>,
    pub in_stock: bool,
}

impl Island {
    /// Every saved page renders exactly one: Judge.me's review `Product` is not one.
    pub(super) fn from_raw(scripts: Vec<LdJsonScript>) -> Result<Island, NormalizeError> {
        let mut islands: Vec<Island> = scripts
            .into_iter()
            .filter_map(|script| match script.data {
                LdJson::ProductGroup(group) => Some(Island::from_group(group)),
                LdJson::Product(ProductIsland::Offers(product)) => {
                    Some(Island::from_product(*product))
                }
                LdJson::Product(ProductIsland::Reviews(_))
                | LdJson::Organization(_)
                | LdJson::BreadcrumbList(_)
                | LdJson::FaqPage(_) => None,
            })
            .collect::<Result<_, _>>()?;

        match islands.len() {
            0 => Err(NormalizeError::MissingProductIsland),
            1 => islands.pop().ok_or(NormalizeError::MissingProductIsland),
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
                .map(|variant| IslandOffer::from_raw(variant.offers, variant.sku, variant.gtin))
                .collect::<Result<_, _>>()?,
        })
    }

    fn from_product(product: ldjson::Product) -> Result<Island, NormalizeError> {
        Ok(Island {
            product_id: None,
            url: product.url,
            category: product.category,
            offers: vec![IslandOffer::from_raw(
                product.offers,
                product.sku,
                product.gtin,
            )?],
        })
    }
}

impl IslandOffer {
    fn from_raw(
        offer: ldjson::Offer,
        sku: String,
        gtin: Option<String>,
    ) -> Result<IslandOffer, NormalizeError> {
        Ok(IslandOffer {
            variant_id: variant_id(&offer.url)?,
            cents: cents(&offer.price).ok_or_else(|| NormalizeError::OfferPriceNotDecimal {
                offer: offer.id.clone(),
                price: offer.price.clone(),
            })?,
            currency: offer.price_currency,
            sku,
            gtin,
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

// Read exactly, without a float: `999.00` is 99900 paise, `999.0` too.
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
        assert_eq!(cents("999.00"), Some(99900));
        assert_eq!(cents("999.9"), Some(99990));
        assert_eq!(cents("999"), Some(99900));
        assert_eq!(cents("6.399"), None);
        assert_eq!(cents("-1.00"), None);
        assert_eq!(cents(""), None);
    }

    #[test]
    fn the_variant_is_read_from_the_offer_url() {
        assert_eq!(
            variant_id("https://www.ugreenindia.com/hi/products/a?variant=42").ok(),
            Some(42)
        );
        assert!(variant_id("https://www.ugreenindia.com/products/a").is_err());
    }
}
