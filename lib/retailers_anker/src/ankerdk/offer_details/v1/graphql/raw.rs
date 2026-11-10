//! Strict schema for the fields selected by product.graphql. Diagnostic GraphQL
//! errors/extensions are retained without interpreting them as product facts.
use html_extractor::serde_json::Value;
use serde::Deserialize;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Response {
    pub data: Option<Data>,
    #[serde(default)]
    pub errors: Vec<Value>,
    #[serde(rename = "extensions")]
    pub _extensions: Option<Value>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Data {
    pub localization: Localization,
    pub product: Option<Product>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Localization {
    pub country: Country,
    pub language: Code,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Country {
    pub iso_code: String,
    pub currency: Code,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Code {
    pub iso_code: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Product {
    pub id: String,
    pub handle: String,
    pub title: String,
    pub description_html: String,
    pub online_store_url: Option<String>,
    pub images: Connection<Image>,
    pub variants: Connection<Variant>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Connection<T> {
    pub nodes: Vec<T>,
    pub page_info: PageInfo,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct PageInfo {
    pub has_next_page: bool,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Image {
    pub url: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Variant {
    pub id: String,
    pub sku: Option<String>,
    pub barcode: Option<String>,
    pub available_for_sale: bool,
    pub price: Price,
    pub compare_at_price: Option<Price>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Price {
    pub amount: String,
    pub currency_code: String,
}
