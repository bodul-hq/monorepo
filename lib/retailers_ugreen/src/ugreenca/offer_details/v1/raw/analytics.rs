/// The inline script ahead of `script.analytics`, which assigns Shopify's analytics `meta`
/// object. It is the one island that lists every variant on every page, picker or not.
#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AnalyticsScript {
    pub meta: Meta,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Meta {
    pub product: Product,
    pub page: Page,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Product {
    pub id: u64,
    pub gid: String,
    pub vendor: String,
    #[serde(rename = "type")]
    pub product_type: String,
    pub handle: String,
    pub variants: Vec<Variant>,
    pub remote: bool,
}

/// A summary of a variant: id, price, name and SKU.
#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Variant {
    pub id: u64,
    pub price: i64,
    pub name: String,
    /// Null on a `Default Title` variant.
    pub public_title: Option<String>,
    pub sku: Option<String>,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Page {
    pub page_type: String,
    pub resource_type: String,
    pub resource_id: u64,
    pub request_id: String,
}

#[cfg(test)]
pub(crate) mod tests {
    use super::Meta;
    use html_extractor::serde_json::{self, Value, json};

    pub(crate) fn meta() -> Value {
        json!({
            "product": {
                "id": 8006139642144_u64,
                "gid": "gid://shopify/Product/8006139642144",
                "vendor": "UGREEN STORE",
                "type": "GaN Charger",
                "handle": "100w-3c1a-gan-fast-charger",
                "variants": [
                    {"id": 44061363634464_u64, "price": 5999,
                        "name": "UGREEN Nexode 100W USB C Charger - Black",
                        "public_title": "Black", "sku": "40737"},
                    {"id": 44061396533536_u64, "price": 6499,
                        "name": "UGREEN Nexode 100W USB C Charger - White",
                        "public_title": "White", "sku": "40738"}
                ],
                "remote": false
            },
            "page": {
                "pageType": "product",
                "resourceType": "product",
                "resourceId": 8006139642144_u64,
                "requestId": "afbacd2c-f883-49e1-86dd-2bf113cdcc6e-1791965173"
            }
        })
    }

    #[test]
    fn the_meta_object_deserializes() {
        let meta = serde_json::from_value::<Meta>(meta()).expect("deserializes");
        assert_eq!(meta.product.variants.len(), 2);
    }
}
