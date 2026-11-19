//! Project the nested v2 Shopify artifact onto the existing offers import contract.

use super::super::entity::SCHEMA_VERSION;
use super::{ConversionError, OfferImport, Variation, VariationIdentifiers};
use chrono::{DateTime, Utc};
use retailer_sauce::prelude::{
    Availability, ExternalIdentifiers, Gtin, Localization, Observation, ShopifyHandle,
    ShopifyProductGid, ShopifyProductVariationGid, Sku, StructuralizedContent, TargetMarket,
    source_locale,
};
use retailer_sauce::shopify::extracted::prelude::Pricing;
use serde::Deserialize;
use serde_json::{from_str, json};
use shared::retailer::RetailerCode;
use std::collections::{HashMap, HashSet};

#[derive(Deserialize)]
struct Document {
    images: HashMap<u64, Image>,
    variations: Vec<ShopifyVariation>,
}

#[derive(Deserialize)]
struct Image {
    url: String,
}

#[derive(Deserialize)]
struct ShopifyVariation {
    market: String,
    content: Content,
    identifiers: Identifiers,
    flags: Flags,
    pricing: Pricing,
}

#[derive(Deserialize)]
struct Flags {
    available_for_sale: bool,
}

#[derive(Deserialize)]
struct Content {
    url: Option<String>,
    title: String,
    description: String,
    language: String,
    images: Vec<u64>,
    featured_image: Option<u64>,
}

#[derive(Deserialize)]
struct Identifiers {
    id: u64,
    product_id: u64,
    handle: ShopifyHandle,
    sku: Option<Sku>,
    barcode: Option<Gtin>,
}

/// Offers identifies a listing by retailer and URL and requires a known price.
pub(crate) fn shopify_imports(
    body: &str,
    retailer: RetailerCode,
    scraped_at: Option<DateTime<Utc>>,
) -> Result<Vec<(u64, OfferImport)>, ConversionError> {
    let document: Document = from_str(body).map_err(|error| ConversionError(error.to_string()))?;
    let mut imports = Vec::new();
    let mut urls = HashSet::new();
    for (index, item) in document.variations.into_iter().enumerate() {
        let Some(url) = item.content.url else {
            continue;
        };
        let (price, was_price) = match item.pricing {
            Pricing::Unknown => continue,
            Pricing::Standard(price) => (price, None),
            Pricing::Discounted(price, was) => (price, Some(was)),
        };
        if !urls.insert(url.clone()) {
            return Err(ConversionError(format!(
                "variation {index} repeats a retailer/URL identity"
            )));
        }
        let mut images = Vec::new();
        for id in item
            .content
            .featured_image
            .into_iter()
            .chain(item.content.images)
        {
            let image = document
                .images
                .get(&id)
                .ok_or_else(|| ConversionError(format!("missing Shopify image {id}")))?;
            images.push(&image.url);
        }
        let ids = item.identifiers;
        let country = item
            .market
            .parse()
            .map_err(|_| ConversionError(format!("invalid Shopify market {}", item.market)))?;
        let language = source_locale(&item.content.language.replace('_', "-"))
            .map_err(ConversionError)?
            .canonical;
        let variation = Variation {
            title: item.content.title,
            url,
            images: json!(images),
            identifiers: VariationIdentifiers {
                gtin: ids.barcode.filter(|v| !v.value.is_empty()),
                sku: ids.sku.filter(|v| !v.value.is_empty()),
                external: ExternalIdentifiers {
                    shopify_product_gid: Some(ShopifyProductGid {
                        value: format!("gid://shopify/Product/{}", ids.product_id),
                    }),
                    shopify_product_variation_gid: Some(ShopifyProductVariationGid {
                        value: format!("gid://shopify/ProductVariant/{}", ids.id),
                    }),
                    shopify_handle: Some(ids.handle),
                },
            },
            price: Some(price),
            was_price: Observation::Observed(was_price),
            availability: Observation::Observed(Some(if item.flags.available_for_sale {
                Availability::Available
            } else {
                Availability::Unavailable
            })),
            item_condition: Observation::Missing,
            content: Some(StructuralizedContent {
                description: Some(item.content.description),
            }),
            localization: Localization {
                target_market: Some(TargetMarket::Country { code: country }),
                language: Some(language),
            },
        };
        if let Some(payload) = variation.import(retailer, scraped_at, SCHEMA_VERSION, index)? {
            imports.push((ids.id, payload));
        }
    }
    Ok(imports)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{Value, to_value};

    #[test]
    fn prepares_discounted_import_and_skips_variants_without_price_or_url() {
        let price = json!({"amount": 1200, "currency":"CAD"});
        let comparison = json!({"amount": 1500, "currency":"CAD"});
        let url = "https://ca.example/products/hub?variant=3";
        let variant = json!({
            "market":"CA", "flags":{"available_for_sale":false},
            "pricing":{"Discounted":[price,comparison]},
            "identifiers":{"id":3,"product_id":2,"handle":"hub","sku":"A3","barcode":"0012"},
            "content":{"url":url,"images":[1],"featured_image":1,"language":"FR","description":"Texte","title":"Hub"}
        });
        let mut unknown = variant.clone();
        unknown["pricing"] = json!("Unknown");
        let mut unlisted = variant.clone();
        unlisted["content"]["url"] = Value::Null;
        let doc = json!({"images":{"1":{"url":"https://ca.example/a.jpg"}}, "variations":[unknown, unlisted, variant]});
        let entries = shopify_imports(&doc.to_string(), RetailerCode::AnkerCa, None).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].0, 3);
        let payload = to_value(&entries[0].1).unwrap();
        assert_eq!(payload["price"], price);
        assert_eq!(payload["was_price"], comparison);
        assert_eq!(payload["availability"], "unavailable");
        assert_eq!(payload["url"], url);
        assert_eq!(payload["localization"]["language"], "fr");
        assert_eq!(
            payload["external_identifiers"]["shopify_product_variation_gid"],
            "gid://shopify/ProductVariant/3"
        );
        assert_eq!(payload["images"], json!(["https://ca.example/a.jpg"]));
    }
}
