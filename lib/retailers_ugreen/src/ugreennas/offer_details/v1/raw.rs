//! Strict schemas for the Shopify islands used by this storefront.

use analytics::AnalyticsScript;
use serde::Deserialize;
use shop::ShopScript;
use variant::Variant;
pub mod analytics;
pub mod shop;
pub mod variant;

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UgreenNasRawDocument {
    pub analytics: AnalyticsScript,
    pub shop: ShopScript,
    pub variants: Script<Vec<Variant>>,
    pub selected_variant: Script<Variant>,
    pub title: Script<String>,
    pub canonical: Canonical,
    pub images: Vec<Image>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Script<T> {
    pub id: String,
    #[serde(rename = "type")]
    pub script_type: String,
    pub data: T,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Canonical {
    pub href: String,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Image {
    pub src: String,
    pub alt: Option<String>,
}
