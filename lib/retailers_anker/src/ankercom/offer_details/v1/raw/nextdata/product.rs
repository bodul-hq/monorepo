use html_extractor::serde_json::Value;
use std::collections::BTreeMap;

pub type MetafieldsMap = BTreeMap<String, BTreeMap<String, String>>;

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Product {
    pub id: String,
    pub handle: String,
    pub slug: String,
    pub path: String,
    pub name: String,
    pub title: String,

    pub description: Option<String>,

    pub description_html: Option<String>,
    pub vendor: String,

    pub product_type: Option<String>,
    pub published_at: String,

    pub price: Price,

    pub list_price: f64,
    pub available_for_sale: bool,
    pub total_inventory: i64,
    pub images: Vec<Image>,
    pub tags: Vec<String>,

    pub options: Vec<ProductOption>,
    pub collections: Collections,

    pub seo: ProductSeo,

    pub online_store_url: Option<String>,
    pub requires_selling_plan: bool,

    pub selling_plan_groups: Vec<Value>,

    pub variants: Vec<Variant>,

    pub metafields: Value,
    pub metafields_map: MetafieldsMap,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Price {
    pub value: f64,
    pub currency_code: String,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Image {
    pub url: String,

    pub alt_text: Option<String>,
    pub width: i64,
    pub height: i64,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ProductOption {
    #[serde(rename = "__typename")]
    pub typename: String,
    pub id: String,
    pub name: String,
    pub display_name: String,
    pub values: Vec<OptionValue>,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct OptionValue {
    pub label: String,

    pub hex_colors: Option<Vec<String>>,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Collections {
    pub nodes: Vec<CollectionNode>,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CollectionNode {
    pub id: String,
    pub handle: String,
    pub title: String,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ProductSeo {
    pub title: Option<String>,

    pub description: Option<String>,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Variant {
    pub id: String,

    pub sku: String,

    pub barcode: Option<String>,
    pub name: String,
    pub price: f64,

    pub list_price: Option<f64>,
    pub quantity_available: i64,
    pub available_for_sale: bool,

    pub currently_not_in_stock: bool,
    pub requires_shipping: bool,

    pub weight: f64,
    pub image: Image,
    pub options: Vec<ProductOption>,

    pub selling_plan_allocations: Vec<Value>,

    pub metafields: Value,

    pub metafields_map: MetafieldsMap,
}
