//! Reduces the ld+json islands to the one that describes the product, with one entry per
//! variant in the island's order.

use super::super::raw::ldjson::{self, LdJson, LdJsonScript};
use super::NormalizeError;

/// The page's `ProductGroup` or `Product` island.
pub(super) struct Island {
    /// `productGroupID`; a single `Product` island states none.
    pub product_id: Option<String>,
    pub url: String,
    pub name: String,
    pub category: String,
    pub variants: Vec<IslandVariant>,
}

pub(super) struct IslandVariant {
    /// The offer's `@id`, to name it in errors.
    pub offer_id: String,
    pub variant_id: u64,
    /// The Shopify price in hundredths.
    pub cents: i64,
    pub currency: String,
    pub sku: String,
    pub gtin: Option<String>,
    pub available: bool,
    pub image: String,
}

impl Island {
    /// Every page renders the shop's `Organization` and exactly one product island.
    pub(super) fn from_raw(scripts: Vec<LdJsonScript>) -> Result<Island, NormalizeError> {
        let mut islands: Vec<Island> = scripts
            .into_iter()
            .filter_map(|script| match script.data {
                LdJson::ProductGroup(group) => Some(Island::from_group(*group)),
                LdJson::Product(product) => Some(Island::from_product(*product)),
                LdJson::Organization(_) => None,
            })
            .collect::<Result<_, _>>()?;

        match (islands.pop(), islands.len()) {
            (Some(island), 0) => Ok(island),
            (None, _) => Err(NormalizeError::MissingProductIsland),
            (Some(_), others) => Err(NormalizeError::SeveralProductIslands { count: others + 1 }),
        }
    }

    fn from_group(group: ldjson::ProductGroup) -> Result<Island, NormalizeError> {
        Ok(Island {
            product_id: Some(group.product_group_id),
            url: group.url,
            name: group.name,
            category: group.category,
            variants: group
                .has_variant
                .into_iter()
                .map(|variant| {
                    IslandVariant::from_raw(
                        variant.offers,
                        variant.sku,
                        variant.gtin,
                        variant.image,
                    )
                })
                .collect::<Result<_, _>>()?,
        })
    }

    fn from_product(product: ldjson::Product) -> Result<Island, NormalizeError> {
        Ok(Island {
            product_id: None,
            url: product.url,
            name: product.name,
            category: product.category,
            variants: vec![IslandVariant::from_raw(
                product.offers,
                product.sku,
                product.gtin,
                product.image,
            )?],
        })
    }
}

impl IslandVariant {
    fn from_raw(
        offer: ldjson::Offer,
        sku: String,
        gtin: Option<String>,
        image: String,
    ) -> Result<IslandVariant, NormalizeError> {
        Ok(IslandVariant {
            variant_id: variant_id(&offer.url)?,
            cents: cents(&offer.price).ok_or_else(|| NormalizeError::OfferPriceNotDecimal {
                offer: offer.id.clone(),
                price: offer.price.clone(),
            })?,
            available: available(&offer.id, &offer.availability)?,
            currency: offer.price_currency,
            sku,
            gtin,
            image,
            offer_id: offer.id,
        })
    }
}

// The corpus states only these two; anything else is a state the survey did not see.
fn available(offer: &str, availability: &str) -> Result<bool, NormalizeError> {
    match availability {
        "http://schema.org/InStock" => Ok(true),
        "http://schema.org/OutOfStock" => Ok(false),
        _ => Err(NormalizeError::UnknownAvailability {
            offer: offer.to_string(),
            availability: availability.to_string(),
        }),
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

// Read exactly, without a float: `55.99` is 5599 hundredths, `55.9` 5590.
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
    use super::{available, cents, variant_id};
    use crate::ugreeneu::offer_details::v1::normalized::NormalizeError;

    #[test]
    fn a_decimal_price_is_read_exactly() {
        assert_eq!(cents("55.99"), Some(5599));
        assert_eq!(cents("55.9"), Some(5590));
        assert_eq!(cents("55"), Some(5500));
        assert_eq!(cents("55,99"), None);
        assert_eq!(cents("6.399"), None);
        assert_eq!(cents("-1.00"), None);
        assert_eq!(cents(""), None);
    }

    #[test]
    fn the_variant_is_read_from_the_offer_url() {
        assert_eq!(
            variant_id("https://eu.ugreen.com/de-at/products/pb561?variant=42").ok(),
            Some(42)
        );
        assert!(variant_id("https://eu.ugreen.com/de-at/products/pb561").is_err());
    }

    #[test]
    fn only_in_stock_and_out_of_stock_are_read() {
        assert_eq!(available("o", "http://schema.org/InStock"), Ok(true));
        assert_eq!(available("o", "http://schema.org/OutOfStock"), Ok(false));
        assert!(matches!(
            available("o", "http://schema.org/PreOrder"),
            Err(NormalizeError::UnknownAvailability { availability, .. })
                if availability == "http://schema.org/PreOrder"
        ));
    }
}
