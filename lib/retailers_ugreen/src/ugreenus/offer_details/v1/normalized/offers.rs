//! Reconcile the selling JSON-LD island with analytics and the selected product section.
use super::super::raw::ldjsonitem::{LdJson, LdJsonItem, Offer, ProductIsland};
use super::{
    NormalizeError,
    product::{OfferPrice, Product},
};
use money::{Currency, Money};
use std::collections::BTreeSet;

pub(super) fn invalid(field: &str, reason: impl Into<String>) -> NormalizeError {
    NormalizeError::InvalidOffer {
        field: field.into(),
        reason: reason.into(),
    }
}
pub(super) fn require(ok: bool, field: &str, reason: &str) -> Result<(), NormalizeError> {
    if ok {
        Ok(())
    } else {
        Err(invalid(field, reason))
    }
}

/// Parse a machine decimal exactly, including numeric JSON rendered with a trailing `.0`.
/// Excess nonzero digits fail; no rounding or float-to-integer cast is used.
pub(super) fn amount(value: &str, currency: Currency) -> Result<Money, NormalizeError> {
    let (whole, fractional) = value.split_once('.').unwrap_or((value, ""));
    let exponent = currency.exponent() as usize;
    require(
        !whole.is_empty()
            && whole.bytes().all(|c| c.is_ascii_digit())
            && fractional.bytes().all(|c| c.is_ascii_digit()),
        "price",
        format!("invalid decimal {value}").as_str(),
    )?;
    require(
        fractional
            .get(exponent..)
            .unwrap_or("")
            .bytes()
            .all(|c| c == b'0'),
        "price",
        "fractional minor unit",
    )?;
    let scale = 10_i64.pow(exponent as u32);
    let whole: i64 = whole
        .parse()
        .map_err(|_| invalid("price", "amount overflows"))?;
    let digits = &fractional[..fractional.len().min(exponent)];
    let fraction = if digits.is_empty() {
        0
    } else {
        digits
            .parse::<i64>()
            .map_err(|_| invalid("price", "invalid fraction"))?
            * 10_i64.pow((exponent - digits.len()) as u32)
    };
    let minor = whole
        .checked_mul(scale)
        .and_then(|v| v.checked_add(fraction))
        .ok_or_else(|| invalid("price", "amount overflows"))?;
    Ok(Money::new(minor, currency))
}
fn available(value: &str) -> Result<bool, NormalizeError> {
    match value
        .strip_prefix("https://schema.org/")
        .or_else(|| value.strip_prefix("http://schema.org/"))
    {
        Some("InStock") => Ok(true),
        Some("OutOfStock" | "SoldOut") => Ok(false),
        _ => Err(invalid("availability", value)),
    }
}
struct PublishedOffer {
    offer: Offer,
    sku: Option<String>,
    gtin: Option<String>,
    image: Option<String>,
}

pub(super) fn enrich(product: &mut Product, items: Vec<LdJsonItem>) -> Result<(), NormalizeError> {
    let mut selling = Vec::new();
    for item in items {
        match item.data {
            LdJson::Product(ProductIsland::Offers(p)) => {
                let p = *p;
                let image = p.image.first().map(str::to_owned);
                let offers = p
                    .offers
                    .into_vec()
                    .into_iter()
                    .map(|offer| PublishedOffer {
                        offer,
                        sku: p.sku.clone(),
                        gtin: p.gtin.clone(),
                        image: image.clone(),
                    })
                    .collect::<Vec<_>>();
                selling.push((p.url, p.description, offers));
            }
            LdJson::ProductGroup(p) => {
                require(
                    product.id == format!("gid://shopify/Product/{}", p.product_group_id),
                    "productGroupID",
                    "does not match analytics",
                )?;
                let offers = p
                    .has_variant
                    .into_iter()
                    .map(|v| PublishedOffer {
                        offer: v.offers,
                        sku: v.sku,
                        gtin: v.gtin,
                        image: Some(v.image),
                    })
                    .collect();
                selling.push((p.url, p.description, offers));
            }
            // Reviews and organization details are typed but are not selling offers.
            LdJson::Product(ProductIsland::Reviews(_)) | LdJson::Organization(_) => {}
        }
    }
    require(
        selling.len() == 1,
        "ld-json",
        "expected exactly one selling Product or ProductGroup",
    )?;
    let (url, description, offers) = selling.remove(0);
    require(
        url == product.url,
        "url",
        "JSON-LD and canonical product URL disagree",
    )?;
    product.description = (!description.trim().is_empty()).then_some(description);
    let mut seen = BTreeSet::new();
    let mut variant_ids = BTreeSet::new();
    for variant in &product.variants {
        require(
            variant_ids.insert(variant.id.clone()),
            "variants",
            "duplicate analytics variant",
        )?;
    }
    for published in offers {
        let offer = published.offer;
        let id = if offer.url == product.url && product.variants.len() == 1 {
            product.variants[0].id.clone()
        } else {
            let numeric = offer
                .url
                .strip_prefix(&format!("{}?variant=", product.url))
                .and_then(|v| v.parse::<u64>().ok())
                .ok_or_else(|| {
                    invalid(
                        "offer.url",
                        format!("ambiguous or foreign URL {}", offer.url),
                    )
                })?;
            format!("gid://shopify/ProductVariant/{numeric}")
        };
        require(seen.insert(id.clone()), "offers", "duplicate variant offer")?;
        let variant = product
            .variants
            .iter_mut()
            .find(|v| v.id == id)
            .ok_or_else(|| invalid("offers", "variant absent from analytics"))?;
        let currency = variant.price.currency();
        require(
            offer.price_currency == currency.as_str(),
            "priceCurrency",
            "does not match shop currency",
        )?;
        let offer_price = amount(&offer.price.decimal(), currency)?;
        let agrees = offer_price == variant.price
            || (product.selected_variant.id == id
                && product.selected_variant.automatic_discount_price.as_ref()
                    == Some(&offer_price));
        // Seven custom US islands omit variant IDs. One bundle's hand-written price
        // is stale (156.99 versus the page's 90.99). Retain that evidence explicitly;
        // variant-addressed Shopify offers must still agree with their records.
        let custom_single_offer =
            offer.id.is_none() && offer.url == product.url && variant_ids.len() == 1;
        require(
            agrees || custom_single_offer,
            "price",
            "JSON-LD disagrees with both analytics and displayed price",
        )?;
        variant.offer_price_mismatch = !agrees;
        variant.offer_price = Some(offer_price);
        if let (Some(a), Some(b)) = (&variant.sku, &published.sku) {
            require(
                a == b || b.is_empty(),
                "sku",
                "JSON-LD and analytics disagree",
            )?;
        }
        if variant.sku.is_none() {
            variant.sku = published.sku.filter(|s| !s.is_empty());
        }
        variant.url = offer.url;
        variant.available = Some(available(&offer.availability)?);
        variant.gtin = published.gtin.filter(|s| !s.is_empty());
        variant.image = published
            .image
            .map(|src| super::product::gallery_image(&src));
        variant.offer_prices = offer
            .price_specification
            .unwrap_or_default()
            .into_iter()
            .map(|p| {
                require(
                    p.price_currency == currency.as_str(),
                    "priceSpecification.priceCurrency",
                    "does not match shop currency",
                )?;
                Ok(OfferPrice {
                    kind: p.price_type,
                    price: amount(&p.price.decimal(), currency)?,
                })
            })
            .collect::<Result<_, NormalizeError>>()?;
        variant.condition = offer.item_condition;
        if product.selected_variant.id == id {
            if let Some(selected) = product.selected_variant.available {
                require(
                    Some(selected) == variant.available,
                    "availability",
                    "JSON-LD and selected variant disagree",
                )?;
            }
            product.selected_variant.available = variant.available;
            if let (Some(a), Some(b)) = (&product.selected_variant.barcode, &variant.gtin) {
                require(a == b, "barcode", "picker and JSON-LD disagree")?;
            }
            if product.selected_variant.barcode.is_none() {
                product.selected_variant.barcode = variant.gtin.clone();
            }
            if variant.gtin.is_none() {
                variant.gtin = product.selected_variant.barcode.clone();
            }
            variant.compare_at_price = product.selected_variant.compare_at_price.clone();
        }
    }
    require(
        seen == variant_ids,
        "offers",
        "variant set differs from analytics",
    )?;
    Ok(())
}
