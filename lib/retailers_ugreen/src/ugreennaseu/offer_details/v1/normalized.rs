//! Reconcile Shopify analytics, Glood's complete Liquid product, and the main product DOM.
//! App recommendations, marketing HTML, tags, timestamps and media presentation stay raw.

use super::raw::{self, UgreenNasEuRawDocument};
use money::{Currency, Money};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use thiserror::Error;

const ORIGIN: &str = "https://ai-eu.ugreen.com";
const CURRENCY: Currency = Currency::EUR;
const HAS_BACK_IN_STOCK: bool = true;
const PURCHASE_PROTECTION: &str = "worry-free-purchase";

#[derive(Debug, Clone, PartialEq, Error)]
pub enum NormalizeError {
    #[error("missing {0}")]
    Missing(String),
    #[error("invalid {0}")]
    Invalid(String),
    #[error("inconsistent {0}")]
    Conflict(String),
    #[error("cannot read Glood product: {0}")]
    Glood(String),
    #[error("invalid rendered price `{text}` in {field}")]
    RenderedPrice { field: String, text: String },
}

fn require(valid: bool, field: impl Into<String>) -> Result<(), NormalizeError> {
    if valid {
        Ok(())
    } else {
        Err(NormalizeError::Invalid(field.into()))
    }
}

fn agrees(valid: bool, field: impl Into<String>) -> Result<(), NormalizeError> {
    if valid {
        Ok(())
    } else {
        Err(NormalizeError::Conflict(field.into()))
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UgreenNasEuNormalizedDocument {
    pub product: Product,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Product {
    pub id: String,
    pub handle: String,
    pub title: String,
    pub vendor: String,
    pub product_type: String,
    /// Regional og:url, including the market locale, rather than the cross-store canonical URL.
    pub url: String,
    pub purchase_protection: bool,
    pub images: Vec<String>,
    pub selected_variant_id: String,
    pub variants: Vec<Variant>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Variant {
    pub id: String,
    pub title: String,
    pub sku: Option<String>,
    pub barcode: Option<String>,
    pub options: Vec<String>,
    pub price: Money,
    pub compare_at_price: Option<Money>,
    pub available: bool,
    pub image: Option<String>,
}

impl UgreenNasEuNormalizedDocument {
    pub fn from_raw(raw: UgreenNasEuRawDocument) -> Result<Self, NormalizeError> {
        require(
            raw.glood.id == "glood-loader"
                && raw.glood.page_enabled == "true"
                && raw.glood.cmp_ab == "1",
            "Glood script attributes",
        )?;
        let glood = raw.glood.product().map_err(NormalizeError::Glood)?;
        let meta = raw.analytics.meta;
        let product = meta.product;
        require(
            product.id > 0 && product.gid == format!("gid://shopify/Product/{}", product.id),
            "product ID",
        )?;
        require(
            meta.page.page_type == "product"
                && meta.page.resource_type == "product"
                && meta.page.resource_id == product.id
                && !product.remote,
            "analytics page identity",
        )?;
        require(valid_handle(&product.handle), "product handle")?;
        agrees(
            glood.id == product.id && glood.handle == product.handle,
            "Glood product identity",
        )?;
        agrees(
            glood.vendor == product.vendor && glood.product_type == product.product_type,
            "Glood vendor/type",
        )?;
        require(
            !glood.requires_selling_plan && glood.selling_plan_groups.is_empty(),
            "product selling plans",
        )?;
        let title = raw.title.text.trim().to_owned();
        require(!title.is_empty(), "product title")?;
        // DOM titles and analytics titles differ on several products; the main heading is displayed.
        let currency = raw
            .paypal
            .currency
            .parse::<Currency>()
            .map_err(|_| NormalizeError::Invalid("PayPal currency".into()))?;
        require(
            raw.paypal.id == "in-context-paypal-metadata" && currency == CURRENCY,
            "PayPal currency/storefront",
        )?;
        require(
            valid_product_url(&raw.product_url.content, &product.handle),
            "regional product URL/handle",
        )?;
        let selected = raw
            .selected_variant_id
            .value
            .parse::<u64>()
            .map_err(|_| NormalizeError::Invalid("selected variant ID".into()))?;
        let records: HashMap<_, _> = glood.variants.iter().map(|v| (v.id, v)).collect();
        require(
            !records.is_empty() && records.len() == glood.variants.len(),
            "Glood variant set",
        )?;
        require(
            records.len() == product.variants.len(),
            "analytics variant count",
        )?;
        let selected_record = records
            .get(&selected)
            .ok_or_else(|| NormalizeError::Missing("selected variant in Glood".into()))?;
        if let Some(picker) = &raw.variants {
            script(picker, None)?;
            match_records(&picker.data, &records, "picker")?;
        }
        require(
            raw.em_product_variants.is_some() == HAS_BACK_IN_STOCK
                && raw.em_selected_variant.is_some() == HAS_BACK_IN_STOCK,
            "back-in-stock islands",
        )?;
        if let Some(bis) = &raw.em_product_variants {
            script(bis, Some("em_product_variants"))?;
            match_records(&bis.data, &records, "back-in-stock")?;
        }
        if let Some(bis) = &raw.em_selected_variant {
            script(bis, Some("em_product_selected_or_first_available_variant"))?;
            agrees(
                &bis.data == *selected_record,
                "back-in-stock selected variant",
            )?;
        }
        let protection = product.handle == PURCHASE_PROTECTION;
        check_price_block(
            &raw.price,
            &raw.price_regular,
            &raw.price_sale,
            &raw.price_compare,
            selected_record,
            protection,
        )?;
        let mut seen = HashSet::new();
        let mut variants = Vec::new();
        // Analytics order is stable even if app/picker records are reordered.
        for a in product.variants {
            require(
                a.id > 0 && seen.insert(a.id),
                format!("analytics variant {} ID", a.id),
            )?;
            let v = records
                .get(&a.id)
                .ok_or_else(|| NormalizeError::Missing(format!("Glood variant {}", a.id)))?;
            for (field, equal) in [
                ("price", a.price == v.price),
                ("sku", a.sku == v.sku),
                ("name", a.name == v.name),
                ("public_title", a.public_title == v.public_title),
            ] {
                agrees(equal, format!("variant {} analytics {field}", a.id))?;
            }
            let options: Vec<_> = [&v.option1, &v.option2, &v.option3]
                .into_iter()
                .flatten()
                .cloned()
                .collect();
            agrees(
                !options.is_empty() && options == v.options,
                format!("variant {} options", a.id),
            )?;
            require(
                v.price >= 0 && v.compare_at_price.is_none_or(|p| p >= 0),
                format!("variant {} prices", a.id),
            )?;
            require(
                !v.requires_selling_plan && v.selling_plan_allocations.is_empty(),
                format!("variant {} selling plans", a.id),
            )?;
            if let Some(image) = &v.featured_image {
                agrees(
                    image.product_id == product.id && image.variant_ids.contains(&a.id),
                    format!("variant {} image identity", a.id),
                )?;
            }
            variants.push(Variant {
                id: format!("gid://shopify/ProductVariant/{}", a.id),
                title: v.title.clone(),
                sku: nonempty(v.sku.clone()),
                barcode: nonempty(v.barcode.clone()),
                options,
                price: Money::new(v.price, currency),
                compare_at_price: v
                    .compare_at_price
                    .filter(|p| *p > v.price)
                    .map(|p| Money::new(p, currency)),
                available: v.available,
                image: v
                    .featured_image
                    .as_ref()
                    .map(|image| image_url(&image.src))
                    .transpose()?,
            });
        }
        let mut images = Vec::new();
        for image in raw.images {
            let url = image_url(&image.src)?;
            if !images.contains(&url) {
                images.push(url);
            }
        }
        require(!images.is_empty(), "product gallery")?;
        Ok(Self {
            product: Product {
                id: product.gid,
                handle: product.handle,
                title,
                vendor: product.vendor,
                product_type: product.product_type,
                url: raw.product_url.content,
                purchase_protection: protection,
                images,
                selected_variant_id: format!("gid://shopify/ProductVariant/{selected}"),
                variants,
            },
        })
    }
}

fn script<T>(script: &raw::VariantScript<T>, id: Option<&str>) -> Result<(), NormalizeError> {
    require(
        script.script_type == "application/json" && script.id.as_deref() == id,
        format!("variant script {id:?}"),
    )
}

fn match_records(
    records: &[raw::variant::Variant],
    expected: &HashMap<u64, &raw::variant::Variant>,
    label: &str,
) -> Result<(), NormalizeError> {
    let mut seen = HashSet::new();
    agrees(
        records.len() == expected.len(),
        format!("{label} variant count"),
    )?;
    for v in records {
        agrees(
            seen.insert(v.id) && expected.get(&v.id).is_some_and(|other| *other == v),
            format!("{label} variant {}", v.id),
        )?;
    }
    Ok(())
}

fn check_price_block(
    price: &Option<raw::Price>,
    regular: &Option<raw::Text>,
    sale: &Option<raw::Text>,
    compare: &Option<raw::Text>,
    selected: &raw::variant::Variant,
    protection: bool,
) -> Result<(), NormalizeError> {
    if protection && price.is_none() && regular.is_none() && sale.is_none() && compare.is_none() {
        return Ok(());
    }
    let price = price
        .as_ref()
        .ok_or_else(|| NormalizeError::Missing("price block".into()))?;
    let classes: Vec<_> = price.class.split_whitespace().collect();
    require(classes.contains(&"price"), "price block class")?;
    let on_sale = selected
        .compare_at_price
        .is_some_and(|p| p > selected.price);
    agrees(
        classes.contains(&"price--sold-out") != selected.available,
        "price availability",
    )?;
    agrees(
        classes.contains(&"price--on-sale") == on_sale,
        "price markdown",
    )?;
    for (field, block) in [("price_regular", regular), ("price_sale", sale)] {
        let text = block
            .as_ref()
            .ok_or_else(|| NormalizeError::Missing(field.into()))?;
        agrees(rendered_price(&text.text, field)? == selected.price, field)?;
    }
    let compare = compare
        .as_ref()
        .ok_or_else(|| NormalizeError::Missing("price_compare".into()))?;
    if !on_sale && compare.text.trim().is_empty() {
        return Ok(());
    }
    let amount = rendered_price(&compare.text, "price_compare")?;
    agrees(
        amount
            == if on_sale {
                selected.compare_at_price.unwrap_or(selected.price)
            } else {
                selected.price
            },
        "price_compare",
    )
}

/// Read the surveyed locale format, rejecting malformed separators and overflow instead of
/// removing arbitrary characters. EUR uses dot grouping/comma decimals, GBP the reverse.
fn rendered_price(text: &str, field: &str) -> Result<i64, NormalizeError> {
    let invalid = || NormalizeError::RenderedPrice {
        field: field.into(),
        text: text.into(),
    };
    let (symbol, code, decimal, group) = if CURRENCY == Currency::GBP {
        ('£', "GBP", '.', ',')
    } else {
        ('€', "EUR", ',', '.')
    };
    let value = text.trim().strip_suffix(code).unwrap_or(text.trim()).trim();
    let value = value
        .strip_prefix(symbol)
        .or_else(|| value.strip_suffix(symbol))
        .ok_or_else(invalid)?
        .trim();
    let (whole, fractional) = value.rsplit_once(decimal).ok_or_else(invalid)?;
    if fractional.len() != 2 || !fractional.bytes().all(|b| b.is_ascii_digit()) {
        return Err(invalid());
    }
    let groups: Vec<_> = whole.split(group).collect();
    if groups
        .iter()
        .any(|g| g.is_empty() || !g.bytes().all(|b| b.is_ascii_digit()))
        || (groups.len() > 1 && (groups[0].len() > 3 || groups[1..].iter().any(|g| g.len() != 3)))
    {
        return Err(invalid());
    }
    let whole = groups.concat().parse::<i64>().map_err(|_| invalid())?;
    let fraction = fractional.parse::<i64>().map_err(|_| invalid())?;
    whole
        .checked_mul(100)
        .and_then(|n| n.checked_add(fraction))
        .ok_or_else(invalid)
}

fn valid_handle(handle: &str) -> bool {
    !handle.is_empty()
        && handle
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-')
}

pub(super) fn valid_product_url(url: &str, handle: &str) -> bool {
    if !valid_handle(handle) {
        return false;
    }
    let Some(path) = url.strip_prefix(ORIGIN) else {
        return false;
    };
    let Some(prefix) = path.strip_suffix(&format!("/products/{handle}")) else {
        return false;
    };
    // EU also serves localized markets. Other storefronts have no locale prefix in the corpus.
    matches!(
        prefix,
        "" | "/de" | "/en-be" | "/en-es" | "/en-fr" | "/en-it" | "/en-nl"
    )
}

fn nonempty(value: Option<String>) -> Option<String> {
    value.map(|s| s.trim().to_owned()).filter(|s| !s.is_empty())
}

fn image_url(src: &str) -> Result<String, NormalizeError> {
    let url = if src.starts_with("//") {
        format!("https:{src}")
    } else {
        src.to_owned()
    };
    require(
        url.starts_with("https://") && url.len() > 8 && !url.chars().any(char::is_whitespace),
        "image URL",
    )?;
    Ok(url)
}
