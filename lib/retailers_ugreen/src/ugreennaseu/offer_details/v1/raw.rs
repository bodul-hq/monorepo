//! Strict mirrors of the product islands. Marketing and app configuration are not offer data.

pub mod analytics;
pub mod paypal;
pub mod variant;

use html_extractor::serde_json::{self, Value};
use serde::Deserialize;

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UgreenNasEuRawDocument {
    pub analytics: analytics::AnalyticsScript,
    pub paypal: paypal::PayPalMetadata,
    pub glood: GloodScript,
    pub product_url: ProductUrl,
    pub title: Text,
    pub selected_variant_id: SelectedVariant,
    pub images: Vec<Image>,
    /// The theme omits its picker on some products, including a two-variant French cable.
    #[serde(default)]
    pub variants: Option<VariantScript<Vec<variant::Variant>>>,
    /// The EU and UK app renders these on every page; France and Italy do not have the app.
    #[serde(default)]
    pub em_product_variants: Option<VariantScript<Vec<variant::Variant>>>,
    #[serde(default)]
    pub em_selected_variant: Option<VariantScript<variant::Variant>>,
    /// Only purchase-protection pages omit the price block.
    #[serde(default)]
    pub price: Option<Price>,
    #[serde(default)]
    pub price_regular: Option<Text>,
    #[serde(default)]
    pub price_sale: Option<Text>,
    #[serde(default)]
    pub price_compare: Option<Text>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GloodScript {
    pub id: String,
    #[serde(rename = "page-enabled")]
    pub page_enabled: String,
    #[serde(rename = "data-cmp-ab")]
    pub cmp_ab: String,
    pub text: String,
}

impl GloodScript {
    /// The loader embeds one JSON product as a line starting `product:` inside
    /// `window.glood`. Deserialize that JSON prefix without evaluating any JavaScript.
    /// Newlines inside JSON strings are escaped, so they cannot become property anchors.
    pub fn product(&self) -> Result<GloodProduct, String> {
        let mut properties = self
            .text
            .lines()
            .filter_map(|line| line.trim_start().strip_prefix("product:"));
        let value = properties.next().ok_or("missing Glood product property")?;
        if properties.next().is_some() {
            return Err("ambiguous Glood product properties".into());
        }
        let mut stream = serde_json::Deserializer::from_str(value).into_iter::<GloodProduct>();
        let product = stream
            .next()
            .ok_or("missing Glood product JSON")?
            .map_err(|e| e.to_string())?;
        if value[stream.byte_offset()..].trim() != "," {
            return Err("unexpected text after Glood product JSON".into());
        }
        Ok(product)
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GloodProduct {
    pub id: u64,
    pub title: String,
    pub handle: String,
    pub vendor: String,
    #[serde(rename = "type")]
    pub product_type: String,
    pub variants: Vec<variant::Variant>,
    pub available: bool,
    pub price: i64,
    pub price_min: i64,
    pub price_max: i64,
    pub price_varies: bool,
    pub compare_at_price: Option<i64>,
    pub compare_at_price_min: i64,
    pub compare_at_price_max: i64,
    pub compare_at_price_varies: bool,
    pub featured_image: Option<String>,
    pub images: Vec<String>,
    pub options: Vec<String>,
    pub created_at: String,
    pub published_at: String,
    pub requires_selling_plan: bool,
    /// No selling plans are present; a new one must be surveyed before accepting it.
    pub selling_plan_groups: Vec<()>,
    /// Rendered marketing HTML, not offer facts. Kept typed as strings but not normalized.
    pub description: String,
    pub content: String,
    /// Campaign labels are not product identifiers or offer prices.
    pub tags: Vec<String>,
    /// Gallery media includes presentation/video metadata; product and variant images supply URLs.
    pub media: Value,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VariantScript<T> {
    /// Picker scripts have no ID; back-in-stock scripts do.
    #[serde(default)]
    pub id: Option<String>,
    #[serde(rename = "type")]
    pub script_type: String,
    pub data: T,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProductUrl {
    pub content: String,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Text {
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SelectedVariant {
    pub value: String,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Price {
    pub class: String,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Image {
    pub src: String,
    pub alt: Option<String>,
}
