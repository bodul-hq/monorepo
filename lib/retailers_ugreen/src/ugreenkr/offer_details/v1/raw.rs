//! Strict Korean extraction schema. Prices here are displayed text; analytics and JSON-LD
//! retain their distinct numeric units until normalization reconciles them.
pub mod analytics;
pub mod ldjsonitem;
use serde::Deserialize;
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UgreenKrRawDocument {
    #[serde(rename = "ld-json")]
    pub ld_json: Vec<ldjsonitem::LdJsonItem>,
    pub analytics: analytics::AnalyticsScript,
    pub canonical: Canonical,
    pub title: Text,
    pub selected_variant_id: SelectedVariantId,
    pub displayed_price: Text,
    pub compare_price: Text,
    pub images: Vec<Image>,
}
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Canonical {
    pub href: String,
}
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Text {
    pub text: String,
}
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SelectedVariantId {
    pub value: String,
}
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Image {
    pub src: String,
    pub alt: Option<String>,
}
