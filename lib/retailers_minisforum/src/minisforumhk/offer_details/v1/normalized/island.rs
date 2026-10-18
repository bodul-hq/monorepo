//! Reduces the schema.org islands to the one that describes the product, in the shape the variant
//! list is checked against. Only the product island is read: the breadcrumbs and the
//! organization restate the product's name and the shop.

use super::super::raw::ldjson::{self, LdJson, LdJsonScript};
use super::NormalizeError;
use retailer_sauce::availability::Availability;

/// The page's `Product` island.
pub(super) struct Island {
    pub product_id: u64,
    /// The product's path, such as `/products/minisforum-ms-a1`.
    pub path: String,
    pub name: String,
    pub brand: String,
    pub description: String,
    pub category: String,
    /// The SKU of the variant the page opens on.
    pub sku: String,
    /// The primary image, at the display width the theme asks the CDN for.
    pub image: String,
    pub offers: Vec<IslandOffer>,
}

pub(super) struct IslandOffer {
    pub variant_id: u64,
    /// In major units, as the island renders it.
    pub price: f64,
    pub currency: String,
    pub sku: Option<String>,
    pub in_stock: bool,
}

impl Island {
    /// Exactly one product island: a page without one is not a product page this reads.
    pub(super) fn from_raw(scripts: Vec<LdJsonScript>) -> Result<Island, NormalizeError> {
        let mut islands: Vec<ldjson::Product> = scripts
            .into_iter()
            .filter_map(|script| match script.data {
                LdJson::Product(product) => Some(*product),
                LdJson::BreadcrumbList(_) | LdJson::Organization(_) => None,
            })
            .collect();

        match islands.len() {
            0 => Err(NormalizeError::MissingProductIsland),
            1 => Island::from_product(islands.remove(0)),
            count => Err(NormalizeError::SeveralProductIslands { count }),
        }
    }

    fn from_product(product: ldjson::Product) -> Result<Island, NormalizeError> {
        let offers = product
            .offers
            .into_iter()
            .map(|offer| IslandOffer::from_raw(offer, &product.url))
            .collect::<Result<_, _>>()?;

        Ok(Island {
            product_id: product.product_id,
            path: product.url,
            name: product.name,
            brand: product.brand.name,
            description: product.description,
            category: product.category,
            sku: product.sku,
            image: product.image.url,
            offers,
            // Commented out: the review summary, present on 11 of the 29 saved pages; it is the
            // shop's reviews, not the offer.
            // aggregate_rating: product.aggregate_rating,
        })
    }
}

impl IslandOffer {
    fn from_raw(offer: ldjson::Offer, product_path: &str) -> Result<IslandOffer, NormalizeError> {
        Ok(IslandOffer {
            variant_id: variant_id(&offer.url, product_path)?,
            price: offer.price,
            currency: offer.price_currency,
            sku: offer.sku,
            in_stock: in_stock(&offer.availability)?,
            // Commented out: `priceValidUntil` is a date a fortnight after the page was
            // rendered, the same for every offer of a page.
            // price_valid_until: offer.price_valid_until,
            // Commented out: `name` is the variant's `public_title`, which the list has.
            // name: offer.name,
        })
    }
}

// An offer URL reads `/products/<handle>?variant=<id>`, with the handle as the product path
// spells it.
fn variant_id(url: &str, product_path: &str) -> Result<u64, NormalizeError> {
    let (path, query) =
        url.split_once('?')
            .ok_or_else(|| NormalizeError::OfferUrlWithoutVariant {
                url: url.to_string(),
            })?;
    if path != product_path {
        return Err(NormalizeError::OfferUrlForAnotherProduct {
            url: url.to_string(),
            product: product_path.to_string(),
        });
    }

    query
        .split('&')
        .find_map(|pair| pair.strip_prefix("variant="))
        .and_then(|id| id.parse().ok())
        .ok_or_else(|| NormalizeError::OfferUrlWithoutVariant {
            url: url.to_string(),
        })
}

// Only the two values the corpus shows are read; anything else is a fact to learn first.
fn in_stock(value: &str) -> Result<bool, NormalizeError> {
    match Availability::from_schema_org(value) {
        Availability::InStock => Ok(true),
        Availability::OutOfStock => Ok(false),
        Availability::SoldOut | Availability::Other(_) => {
            Err(NormalizeError::UnknownAvailability {
                value: value.to_string(),
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{in_stock, variant_id};

    #[test]
    fn the_variant_is_read_from_the_offer_url() {
        assert_eq!(
            variant_id("/products/a?variant=42", "/products/a").ok(),
            Some(42)
        );
        assert!(variant_id("/products/a", "/products/a").is_err());
        assert!(variant_id("/products/a?variant=x", "/products/a").is_err());
        assert!(variant_id("/products/b?variant=42", "/products/a").is_err());
    }

    #[test]
    fn only_in_stock_and_out_of_stock_are_read() {
        assert_eq!(in_stock("https://schema.org/InStock").ok(), Some(true));
        assert_eq!(in_stock("https://schema.org/OutOfStock").ok(), Some(false));
        assert!(in_stock("https://schema.org/SoldOut").is_err());
        assert!(in_stock("InStock").is_err());
    }
}
