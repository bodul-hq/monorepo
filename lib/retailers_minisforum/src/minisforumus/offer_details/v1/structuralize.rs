//! Validated variations from the storefront's normalized product.
use super::normalized::MinisforumUsNormalizedDocument;
use money::Money;
use retailer_sauce::identifier::{
    Gtin, IdentifierError, ShopifyHandle, ShopifyProductGid, ShopifyProductVariationGid, Sku,
};
use serde::Serialize;
#[derive(Debug, Serialize)]
pub struct Document {
    pub variations: Vec<ProductVariation>,
}
#[derive(Debug, Serialize)]
pub struct ProductVariation {
    pub identifiers: Identifiers,
    pub title: String,
    pub price: Option<Money>,
    pub compare_at_price: Option<Money>,
    pub available: Option<bool>,
    pub url: String,
    pub images: Vec<String>,
}
#[derive(Debug, Serialize)]
pub struct Identifiers {
    pub gtin: Option<Gtin>,
    pub sku: Option<Sku>,
    pub shopify_product_gid: ShopifyProductGid,
    pub shopify_product_variation_gid: ShopifyProductVariationGid,
    pub shopify_handle: ShopifyHandle,
}
impl Document {
    pub fn from_normalized(input: MinisforumUsNormalizedDocument) -> Result<Self, IdentifierError> {
        let product = input.product;
        let mut variations = Vec::with_capacity(product.variants.len());
        for variant in &product.variants {
            let identifiers = Identifiers {
                gtin: (variant.barcode.clone()).map(Gtin::try_from).transpose()?,
                sku: (variant.sku.clone()).map(Sku::try_from).transpose()?,
                shopify_product_gid: product.id.clone().try_into()?,
                shopify_product_variation_gid: variant.id.clone().try_into()?,
                shopify_handle: product.handle.clone().try_into()?,
            };
            // Preserve the product's market path; only multi-variant links need a selection.
            let base_url = product.url.clone();
            let url = if product.variants.len() > 1 {
                format!(
                    "{}?variant={}",
                    base_url,
                    identifiers.shopify_product_variation_gid.numeric_id()?
                )
            } else {
                base_url
            };
            let mut images = product.images.clone();
            if let Some(image) = None::<&String> {
                images.retain(|entry| entry != image);
                images.insert(0, image.clone());
            }
            variations.push(ProductVariation {
                identifiers,
                title: product.title.clone(),
                price: variant.price.clone(),
                compare_at_price: variant.compare_at_price.clone(),
                available: Some(variant.available),
                url,
                images,
            });
        }
        Ok(Self { variations })
    }
}
