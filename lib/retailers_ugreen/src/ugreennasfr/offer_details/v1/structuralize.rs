//! One offer per Shopify variant, retaining that variant's own price and availability.

use super::normalized::UgreenNasFrNormalizedDocument;
use money::Money;
use retailer_sauce::identifier::{
    Gtin, IdentifierError, ShopifyHandle, ShopifyProductGid, ShopifyProductVariationGid, Sku,
};
use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum StructuralizeError {
    #[error(transparent)]
    Identifier(#[from] IdentifierError),
    #[error("invalid normalized product URL or variant set")]
    InvalidProduct,
}

#[derive(Debug, Serialize)]
pub struct UgreenNasFrDocument {
    pub variations: Vec<ProductVariation>,
}

#[derive(Debug, Serialize)]
pub struct ProductVariation {
    pub identifiers: ProductVariationIdentifier,
    pub title: String,
    pub price: Money,
    pub compare_at_price: Option<Money>,
    pub available: bool,
    pub url: String,
    pub images: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct ProductVariationIdentifier {
    pub gtin: Option<Gtin>,
    pub sku: Option<Sku>,
    pub shopify_product_gid: ShopifyProductGid,
    pub shopify_product_variation_gid: ShopifyProductVariationGid,
    pub shopify_handle: ShopifyHandle,
}

impl UgreenNasFrDocument {
    pub fn from_normalized(
        input: UgreenNasFrNormalizedDocument,
    ) -> Result<Self, StructuralizeError> {
        let p = input.product;
        let count = p.variants.len();
        if count == 0
            || !super::normalized::valid_product_url(&p.url, &p.handle)
            || p.purchase_protection != (p.handle == "worry-free-purchase")
            || p.title.trim().is_empty()
            || !p.variants.iter().any(|v| v.id == p.selected_variant_id)
        {
            return Err(StructuralizeError::InvalidProduct);
        }
        if p.purchase_protection {
            return Ok(Self {
                variations: Vec::new(),
            });
        }
        let mut seen = std::collections::HashSet::new();
        let mut variations = Vec::new();
        for v in p.variants {
            if !seen.insert(v.id.clone())
                || v.price.amount() < 0
                || v.price.currency() != money::Currency::EUR
                || v.compare_at_price.as_ref().is_some_and(|c| {
                    c.currency() != v.price.currency() || c.amount() <= v.price.amount()
                })
            {
                return Err(StructuralizeError::InvalidProduct);
            }
            let identifiers = ProductVariationIdentifier {
                gtin: v.barcode.map(Gtin::try_from).transpose()?,
                sku: v.sku.map(Sku::try_from).transpose()?,
                shopify_product_gid: p.id.clone().try_into()?,
                shopify_product_variation_gid: v.id.try_into()?,
                shopify_handle: p.handle.clone().try_into()?,
            };
            let id = identifiers.shopify_product_variation_gid.numeric_id()?;
            let url = if count == 1 {
                p.url.clone()
            } else {
                format!("{}?variant={id}", p.url)
            };
            let mut images = p.images.clone();
            if let Some(image) = v.image {
                images.retain(|url| url != &image);
                images.insert(0, image);
            }
            variations.push(ProductVariation {
                identifiers,
                title: p.title.clone(),
                price: v.price,
                compare_at_price: v.compare_at_price,
                available: v.available,
                url,
                images,
            });
        }
        Ok(Self { variations })
    }
}
