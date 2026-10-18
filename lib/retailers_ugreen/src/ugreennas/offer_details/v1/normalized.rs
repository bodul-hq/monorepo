//! Reconcile the two variant islands without inventing missing identifiers or prices.

use super::raw::UgreenNasRawDocument;
use money::{Currency, Money};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Error)]
pub enum NormalizeError {
    #[error("invalid or inconsistent {0}")]
    Invalid(String),
}

fn require(valid: bool, field: impl Into<String>) -> Result<(), NormalizeError> {
    if valid {
        Ok(())
    } else {
        Err(NormalizeError::Invalid(field.into()))
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UgreenNasNormalizedDocument {
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
    pub url: String,
    /// The two refurbished products canonicalize to the new models, which have other IDs.
    pub canonical_url: String,
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

impl UgreenNasNormalizedDocument {
    pub fn from_raw(raw: UgreenNasRawDocument) -> Result<Self, NormalizeError> {
        for (id, kind, expected) in [
            (
                &raw.shop.id,
                &raw.shop.script_type,
                "apple-pay-shop-capabilities",
            ),
            (
                &raw.variants.id,
                &raw.variants.script_type,
                "em_product_variants",
            ),
            (
                &raw.selected_variant.id,
                &raw.selected_variant.script_type,
                "em_product_selected_or_first_available_variant",
            ),
            (
                &raw.title.id,
                &raw.title.script_type,
                "ecomsend_bis_product_title",
            ),
        ] {
            require(
                id == expected && kind == "application/json",
                format!("script {expected}"),
            )?;
        }
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
        require(
            !product.handle.is_empty()
                && product
                    .handle
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b == b'-'),
            "product handle",
        )?;
        let title = raw.title.data.trim().to_owned();
        require(!title.is_empty(), "product title")?;
        // USD is the surveyed shop currency; accepting another exponent would mis-scale cents.
        require(
            raw.shop.data.currency_code == "USD",
            "shop currency (expected USD)",
        )?;
        let canonical = raw.canonical.href;
        let canonical_handle = canonical
            .strip_prefix("https://ai.ugreen.com/products/")
            .ok_or_else(|| NormalizeError::Invalid("canonical URL".into()))?;
        require(
            !canonical_handle.is_empty()
                && canonical_handle
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b == b'-'),
            "canonical handle",
        )?;
        require(
            canonical_handle == product.handle
                || matches!(
                    (product.handle.as_str(), canonical_handle),
                    (
                        "ugreen-nasync-dxp2800-refurbished",
                        "ugreen-nasync-dxp2800-nas-storage"
                    ) | (
                        "ugreen-nasync-dxp4800-plus-refurbished",
                        "ugreen-nasync-dxp4800-plus-nas-storage"
                    )
                ),
            "canonical/product identity",
        )?;
        let analytics: HashMap<_, _> = product.variants.iter().map(|v| (v.id, v)).collect();
        require(
            !raw.variants.data.is_empty()
                && analytics.len() == product.variants.len()
                && analytics.len() == raw.variants.data.len(),
            "variant set",
        )?;
        require(
            raw.variants.data.contains(&raw.selected_variant.data),
            "selected variant",
        )?;
        let mut seen = HashSet::new();
        let mut variants = Vec::new();
        for variant in raw.variants.data {
            let label = format!("variant {}", variant.id);
            require(
                variant.id > 0 && seen.insert(variant.id),
                format!("{label} ID"),
            )?;
            let a = analytics
                .get(&variant.id)
                .ok_or_else(|| NormalizeError::Invalid(format!("{label} absent from analytics")))?;
            require(
                a.price == variant.price
                    && a.name == variant.name
                    && a.public_title == variant.public_title
                    && a.sku == variant.sku,
                format!("{label} analytics"),
            )?;
            let name = match &variant.public_title {
                Some(t) => format!("{title} - {t}"),
                None => title.clone(),
            };
            require(variant.name == name, format!("{label} product title"))?;
            let options: Vec<_> = [&variant.option1, &variant.option2, &variant.option3]
                .into_iter()
                .flatten()
                .cloned()
                .collect();
            require(
                !options.is_empty() && options == variant.options,
                format!("{label} options"),
            )?;
            require(
                variant.price >= 0 && variant.compare_at_price.is_none_or(|p| p >= 0),
                format!("{label} price"),
            )?;
            require(
                !variant.requires_selling_plan && variant.selling_plan_allocations.is_empty(),
                format!("{label} selling plan"),
            )?;
            if let Some(image) = &variant.featured_image {
                require(
                    image.product_id == product.id && image.variant_ids.contains(&variant.id),
                    format!("{label} image identity"),
                )?;
            }
            variants.push(Variant {
                id: format!("gid://shopify/ProductVariant/{}", variant.id),
                title: variant.title,
                sku: nonempty(variant.sku),
                barcode: nonempty(variant.barcode),
                options,
                price: Money::new(variant.price, Currency::USD),
                compare_at_price: variant
                    .compare_at_price
                    .map(|p| Money::new(p, Currency::USD)),
                available: variant.available,
                image: variant
                    .featured_image
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
                url: format!("https://ai.ugreen.com/products/{}", product.handle),
                canonical_url: canonical,
                handle: product.handle,
                title,
                vendor: product.vendor,
                product_type: product.product_type,
                images,
                selected_variant_id: format!(
                    "gid://shopify/ProductVariant/{}",
                    raw.selected_variant.data.id
                ),
                variants,
            },
        })
    }
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
    require(url.starts_with("https://") && url.len() > 8, "image URL")?;
    Ok(url)
}
