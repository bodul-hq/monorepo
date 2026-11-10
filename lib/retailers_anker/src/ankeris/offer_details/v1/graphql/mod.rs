//! Independent legacy GraphQL integration for ankeris.
//! HTML is Swedish-default even with the site's country cookie. Fetch the public
//! Storefront API with an explicit country and check its returned localization.

pub mod raw;

use crate::nordic::{Document, Variant};
use ::shared::NormalizedData;
use html_extractor::serde_json::{self, json};
use money::{Currency, Money};
use retailer_sauce::{
    identifier::{Gtin, ShopifyHandle, ShopifyProductGid, ShopifyProductVariationGid, Sku},
    localization::{Localization, TargetMarket},
    structuralized::StructuralizedData,
};

pub const ENDPOINT: &str = "https://ankerneu.myshopify.com/api/2026-10/graphql.json";
const ORIGIN: &str = "https://www.ankernordics.com";

#[derive(Debug, Clone, Copy)]
pub struct Store;
impl Store {
    pub fn country(self) -> &'static str {
        "IS"
    }
    fn currency(self) -> Currency {
        Currency::ISK
    }
    pub fn request(self, url: &str) -> Result<String, String> {
        let handle = product_handle(url)?;
        Ok(json!({"query":include_str!("product.graphql"), "variables":{"handle":handle,"country":self.country()}}).to_string())
    }

    /// Validate before saving a download: failed/partial API responses must retry.
    pub fn validate_download(self, url: &str, body: &str) -> Result<(), String> {
        let document = self.document(body)?;
        if document.handle.value != product_handle(url)? {
            return Err("Nordic response product does not match the requested handle".into());
        }
        Ok(())
    }

    pub fn extract(self, body: &str) -> Result<String, String> {
        self.document(body)?;
        Ok(body.to_owned())
    }

    pub fn normalize(self, body: &str) -> Result<NormalizedData, String> {
        let document = self.document(body)?;
        Ok(NormalizedData::just_document_json(
            serde_json::to_string(&document).map_err(|e| e.to_string())?,
        ))
    }

    fn document(self, body: &str) -> Result<Document, String> {
        let response: raw::Response = serde_json::from_str(body).map_err(|e| e.to_string())?;
        if !response.errors.is_empty() {
            return Err("Nordic Storefront API returned GraphQL errors".into());
        }
        let data = response
            .data
            .ok_or("Nordic Storefront API returned no data")?;
        let localization = &data.localization;
        if localization.country.iso_code != self.country()
            || localization.country.currency.iso_code != self.currency().as_str()
            || localization.language.iso_code != "EN"
        {
            return Err(format!(
                "Nordic response localization does not match {} / {:?} / EN",
                self.country(),
                self.currency()
            ));
        }
        let product = data
            .product
            .ok_or("Nordic product is unavailable in the requested market")?;
        if product.images.page_info.has_next_page || product.variants.page_info.has_next_page {
            return Err("Nordic product response is incomplete (pagination required)".into());
        }
        if product.variants.nodes.is_empty() {
            return Err("Nordic product has no variants".into());
        }
        let handle = ShopifyHandle::try_from(product.handle).map_err(|e| e.to_string())?;
        // Headless-channel products often have no Shopify onlineStoreUrl. The
        // verified Nordic sitemap uses /products/<handle> for these products.
        let canonical = product
            .online_store_url
            .unwrap_or_else(|| format!("{ORIGIN}/products/{}", handle.value));
        let canonical = canonical.replacen(
            "https://checkout.ankernordics.com/",
            "https://www.ankernordics.com/",
            1,
        );
        if product_handle(&canonical)? != handle.value {
            return Err("Nordic canonical URL does not match product handle".into());
        }
        let mut variants = Vec::new();
        let mut seen = std::collections::HashSet::new();
        for variant in product.variants.nodes {
            let id = ShopifyProductVariationGid::try_from(variant.id).map_err(|e| e.to_string())?;
            id.numeric_id().map_err(|e| e.to_string())?;
            if !seen.insert(id.value.clone()) {
                return Err("duplicate Nordic variant".into());
            }
            variants.push(Variant {
                id,
                sku: optional_identifier(variant.sku, Sku::try_from)?,
                gtin: optional_identifier(variant.barcode, Gtin::try_from)?,
                price: self.money(&variant.price)?,
                comparison: variant
                    .compare_at_price
                    .as_ref()
                    .map(|p| self.money(p))
                    .transpose()?,
                available: variant.available_for_sale,
            });
        }
        Ok(Document {
            country: self.country().into(),
            language: localization.language.iso_code.to_lowercase(),
            id: ShopifyProductGid::try_from(product.id).map_err(|e| e.to_string())?,
            handle,
            title: product.title,
            description: product.description_html,
            canonical,
            images: product
                .images
                .nodes
                .into_iter()
                .map(|image| image.url)
                .collect(),
            variants,
        })
    }

    fn money(self, price: &raw::Price) -> Result<Money, String> {
        if price.currency_code != self.currency().as_str() {
            return Err("Nordic variant currency does not match its market".into());
        }
        // Shopify returns decimals as strings. Parse exactly into minor units.
        let (whole, fraction) = price.amount.split_once('.').unwrap_or((&price.amount, ""));
        if whole.is_empty()
            || !whole.bytes().all(|b| b.is_ascii_digit())
            || !fraction.bytes().all(|b| b.is_ascii_digit())
        {
            return Err("invalid Nordic price amount".into());
        }
        let exponent = usize::from(self.currency().exponent());
        if fraction
            .get(exponent..)
            .is_some_and(|tail| tail.bytes().any(|b| b != b'0'))
        {
            return Err("Nordic price has fractional minor units".into());
        }
        let fraction = &fraction[..fraction.len().min(exponent)];
        let digits = format!("{whole}{fraction:0<exponent$}");
        let amount = digits
            .parse::<i64>()
            .map_err(|_| "Nordic price overflows minor units")?;
        Ok(Money::new(amount, self.currency()))
    }

    pub fn structuralize(self, body: &str) -> Result<StructuralizedData, String> {
        let document: Document = serde_json::from_str(body).map_err(|e| e.to_string())?;
        if document.country != self.country() || document.language != "en" {
            return Err("normalized Nordic document belongs to another market".into());
        }
        let localization = Localization {
            target_market: Some(TargetMarket::Country {
                code: document
                    .country
                    .parse()
                    .map_err(|e| format!("invalid Nordic country: {e}"))?,
            }),
            language: Some(document.language),
        };
        let mut variations = Vec::new();
        for variant in document.variants {
            if variant.price.currency() != self.currency()
                || variant
                    .comparison
                    .as_ref()
                    .is_some_and(|p| p.currency() != self.currency())
            {
                return Err("normalized Nordic variant currency does not match its market".into());
            }
            // The headless store has shared shopper URLs. Market identity is the
            // retailer code plus the API's localization, not an invented path.
            let url = format!(
                "{ORIGIN}/products/{}?variant={}",
                document.handle.value,
                variant.id.numeric_id().map_err(|e| e.to_string())?
            );
            variations.push(json!({
                "identifiers": {"gtin":variant.gtin,"sku":variant.sku,"shopify_product_gid":document.id,"shopify_product_variation_gid":variant.id,"shopify_handle":document.handle},
                "title":document.title,"price":variant.price,
                "was_price":crate::offer_facts::was_price(Some(&variant.price),variant.comparison.as_ref()),
                "availability":crate::offer_facts::purchase_available(variant.available),
                "item_condition":null,"content":{"description":document.description},
                "url":url,"images":document.images,"localization":localization,
            }));
        }
        Ok(StructuralizedData::just_document_json(
            json!({"variations":variations}).to_string(),
        ))
    }
}

fn optional_identifier<T>(
    value: Option<String>,
    convert: impl FnOnce(String) -> Result<T, retailer_sauce::identifier::IdentifierError>,
) -> Result<Option<T>, String> {
    value
        .filter(|v| !v.trim().is_empty())
        .map(convert)
        .transpose()
        .map_err(|e| e.to_string())
}

fn product_handle(url: &str) -> Result<&str, String> {
    let path = url
        .strip_prefix("https://www.ankernordics.com/products/")
        .ok_or("not a Nordic product URL")?;
    let handle = path.split(['?', '#']).next().unwrap_or_default();
    if handle.is_empty()
        || !handle
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
    {
        return Err("invalid Nordic product handle".into());
    }
    Ok(handle)
}

impl crate::nordic::Integration for Store {
    fn endpoint(&self) -> &'static str {
        ENDPOINT
    }
    fn country(&self) -> &'static str {
        Store::country(*self)
    }
    fn request(&self, url: &str) -> Result<String, String> {
        Store::request(*self, url)
    }
    fn validate_download(&self, url: &str, body: &str) -> Result<(), String> {
        Store::validate_download(*self, url, body)
    }
    fn extract(&self, body: &str) -> Result<String, String> {
        Store::extract(*self, body)
    }
    fn normalize(&self, body: &str) -> Result<NormalizedData, String> {
        Store::normalize(*self, body)
    }
    fn structuralize(&self, body: &str) -> Result<StructuralizedData, String> {
        Store::structuralize(*self, body)
    }
}
#[cfg(test)]
mod source_tests {
    use super::*;
    #[test]
    fn local_fixture_and_query_match_the_retailer() {
        let body = include_str!("response.json");
        let _: raw::Response = serde_json::from_str(body).unwrap();
        Store
            .validate_download("https://www.ankernordics.com/products/t90r0121", body)
            .unwrap();
        let request: serde_json::Value = serde_json::from_str(
            &Store
                .request("https://www.ankernordics.com/products/t90r0121")
                .unwrap(),
        )
        .unwrap();
        assert_eq!(request["query"], include_str!("product.graphql"));
        assert_eq!(request["variables"]["country"], Store.country());
    }
}
