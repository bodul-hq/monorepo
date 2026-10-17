use super::super::raw;
use super::{NormalizeError, variant_gid, yen, yen_currency};
use chrono::NaiveDate;
use money::{Currency, Money};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LdJson {
    /// Older PowerBag pages render the Liquid record but no Product JSON-LD.
    pub product: Option<Product>,
    pub organization: Option<Organization>,
    pub breadcrumb_list: Option<BreadcrumbList>,
}

impl LdJson {
    pub(crate) fn from_raw(
        items: Vec<raw::ldjsonitem::LdJsonItem>,
    ) -> Result<LdJson, NormalizeError> {
        let mut product = None;
        let mut organization = None;
        let mut breadcrumb_list = None;

        for item in items {
            match item.data {
                raw::ldjsonitem::LdJson::Product(island) => {
                    if product.is_none() {
                        product = Some(Product::from_raw(island)?);
                    }
                }
                raw::ldjsonitem::LdJson::Organization(raw) => {
                    organization.get_or_insert_with(|| Organization::from_raw(raw));
                }
                raw::ldjsonitem::LdJson::BreadcrumbList(raw) => {
                    breadcrumb_list.get_or_insert_with(|| BreadcrumbList::from_raw(raw));
                }
            }
        }

        Ok(LdJson {
            product,
            organization,
            breadcrumb_list,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Product {
    // Commented out: the record's `title` behind a `【公式】` ("official") prefix on every page.
    // pub name: String,
    /// Plain text, whitespace collapsed and sometimes cut short: a rendering of the record's HTML
    /// `description`, not a copy of it.
    pub description: String,
    // Commented out: the record's `featured_image`, requested at `&width=1500`, on every page.
    // pub images: Vec<String>,
    pub url: String,
    pub brand: String,
    /// The offers' `priceCurrency`; the only statement of a currency the page's payloads make.
    pub currency: Currency,
    pub offers: Vec<Offer>,
    pub aggregate_rating: Option<AggregateRating>,
    pub reviews: Option<Vec<Review>>,
}

impl Product {
    fn from_raw(raw: raw::ldjsonitem::Product) -> Result<Product, NormalizeError> {
        let currency = offer_currency(&raw.offers)?;

        Ok(Product {
            // name: raw.name,
            description: raw.description,
            // images: raw.image,
            url: raw.url,
            brand: raw.brand.name,
            currency,
            offers: raw
                .offers
                .into_iter()
                .map(Offer::from_raw)
                .collect::<Result<_, _>>()?,
            aggregate_rating: raw.aggregate_rating.map(AggregateRating::from_raw),
            reviews: raw
                .review
                .map(|reviews| reviews.into_iter().map(Review::from_raw).collect())
                .transpose()?,
        })
    }
}

fn offer_currency(offers: &[raw::ldjsonitem::Offer]) -> Result<Currency, NormalizeError> {
    let mut codes: Vec<String> = offers
        .iter()
        .map(|offer| offer.price_currency.clone())
        .collect();
    codes.sort();
    codes.dedup();

    match codes.as_slice() {
        [] => Err(NormalizeError::MissingOfferCurrency),
        [code] => yen_currency(code, "offers.priceCurrency"),
        _ => Err(NormalizeError::MixedOfferCurrencies { currencies: codes }),
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Offer {
    /// Read from the `?variant=` the offer URL pins; the offer names its variant nowhere else.
    pub variant_id: String,
    // Commented out: the variant's record `price` in whole yen, on every offer.
    // pub price: Money,
    // Commented out: the variant's record `available`, in an `http://schema.org/` spelling that
    // `Availability::from_schema_org` does not read.
    // pub availability: Availability,
    // Commented out: the variant's record `sku`, absent where that is empty.
    // pub sku: Option<String>,
    // Commented out: the variant's record `compare_at_price` in whole yen, as a strikethrough
    // `UnitPriceSpecification`, on every marked-down offer.
    // pub price_specification: Option<PriceSpecification>,
    // Commented out: the product URL plus the pinned `?variant=`, lifted as `variant_id`.
    // pub url: String,
    pub return_policy: MerchantReturnPolicy,
    pub shipping_rate: Money,
}

impl Offer {
    fn from_raw(raw: raw::ldjsonitem::Offer) -> Result<Offer, NormalizeError> {
        let rate = raw.shipping_details.shipping_rate;

        Ok(Offer {
            variant_id: variant_gid(offered_variant_id(&raw.url)?),
            return_policy: MerchantReturnPolicy::from_raw(raw.has_merchant_return_policy),
            shipping_rate: yen(rate.value, &rate.currency, "shippingRate.value")?,
        })
    }
}

fn offered_variant_id(url: &str) -> Result<u64, NormalizeError> {
    url.split_once('?')
        .and_then(|(_, query)| {
            query
                .split('&')
                .find_map(|pair| pair.strip_prefix("variant="))
        })
        .and_then(|id| id.parse().ok())
        .ok_or_else(|| NormalizeError::OfferUrlWithoutVariant {
            url: url.to_string(),
        })
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MerchantReturnPolicy {
    pub applicable_country: String,
    pub return_policy_country: String,
    pub return_policy_category: String,
    pub merchant_return_days: i64,
    pub return_method: String,
    pub return_fees: String,
}

impl MerchantReturnPolicy {
    fn from_raw(raw: raw::ldjsonitem::MerchantReturnPolicy) -> MerchantReturnPolicy {
        MerchantReturnPolicy {
            applicable_country: raw.applicable_country,
            return_policy_country: raw.return_policy_country,
            return_policy_category: raw.return_policy_category,
            merchant_return_days: raw.merchant_return_days,
            return_method: raw.return_method,
            return_fees: raw.return_fees,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AggregateRating {
    pub rating_value: f64,
    pub review_count: i64,
}

impl AggregateRating {
    fn from_raw(raw: raw::ldjsonitem::AggregateRating) -> AggregateRating {
        AggregateRating {
            rating_value: raw.rating_value,
            review_count: raw.review_count,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Review {
    pub author: String,
    pub date_published: NaiveDate,
    pub review_body: String,
    pub review_rating: i64,
}

impl Review {
    fn from_raw(raw: raw::ldjsonitem::Review) -> Result<Review, NormalizeError> {
        let date_published =
            NaiveDate::parse_from_str(&raw.date_published, "%Y-%m-%d").map_err(|source| {
                NormalizeError::NotADate {
                    value: raw.date_published.clone(),
                    source,
                }
            })?;

        Ok(Review {
            author: raw.author.name,
            date_published,
            review_body: raw.review_body,
            review_rating: raw.review_rating.rating_value,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Organization {
    pub name: String,
    pub url: String,
    pub logo: String,
    pub same_as: Vec<String>,
}

impl Organization {
    fn from_raw(raw: raw::ldjsonitem::Organization) -> Organization {
        Organization {
            name: raw.name,
            url: raw.url,
            logo: raw.logo,
            same_as: raw.same_as,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BreadcrumbList {
    pub crumbs: Vec<Breadcrumb>,
}

impl BreadcrumbList {
    fn from_raw(raw: raw::ldjsonitem::BreadcrumbList) -> BreadcrumbList {
        BreadcrumbList {
            crumbs: raw
                .item_list_element
                .into_iter()
                .map(Breadcrumb::from_raw)
                .collect(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Breadcrumb {
    pub name: String,

    /// The final breadcrumb for the current page may have no link.
    pub url: Option<String>,
    pub position: i64,
}

impl Breadcrumb {
    fn from_raw(raw: raw::ldjsonitem::BreadcrumbItem) -> Breadcrumb {
        Breadcrumb {
            name: raw.name,
            url: raw.item,
            position: raw.position,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::offered_variant_id;
    use crate::ankerjapancom::offer_details::v1::normalized::NormalizeError;

    #[test]
    fn the_offer_names_its_variant_only_in_its_url() {
        assert_eq!(
            offered_variant_id("https://www.ankerjapan.com/products/a1340?variant=42"),
            Ok(42)
        );
        assert!(matches!(
            offered_variant_id("https://www.ankerjapan.com/products/a1340"),
            Err(NormalizeError::OfferUrlWithoutVariant { .. })
        ));
    }
}
