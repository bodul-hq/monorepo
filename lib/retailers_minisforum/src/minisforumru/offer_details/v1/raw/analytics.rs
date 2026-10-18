/// The inline script ahead of `script.analytics`, which assigns Shopify's analytics `meta`
/// object. It is the only island that carries the product's id, handle, vendor and type.
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

/// A summary of a variant record; every field restates `variant::Variant`.
#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Variant {
    pub id: u64,
    pub price: i64,
    pub name: String,
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
                "id": 8640842891442_u64,
                "gid": "gid://shopify/Product/8640842891442",
                "vendor": "Minisforum RU",
                "type": "Mini PC",
                "handle": "minisforum-ms-s1-max",
                "variants": [
                    {"id": 47567232336050_u64, "price": 259900,
                        "name": "MINISFORUM MS-S1 MAX - AMD Ryzen™ AI Max+ 395 / 64GB RAM+2TB SSD / EU",
                        "public_title": "AMD Ryzen™ AI Max+ 395 / 64GB RAM+2TB SSD / EU",
                        "sku": null},
                    {"id": 47567232368818_u64, "price": 299900,
                        "name": "MINISFORUM MS-S1 MAX - AMD Ryzen™ AI Max+ 395 / 128GB RAM+2TB SSD / EU",
                        "public_title": "AMD Ryzen™ AI Max+ 395 / 128GB RAM+2TB SSD / EU",
                        "sku": ""}
                ],
                "remote": false
            },
            "page": {
                "pageType": "product",
                "resourceType": "product",
                "resourceId": 8640842891442_u64,
                "requestId": "f0aeef04-f426-4609-960e-02a5e9466553-1791962846"
            }
        })
    }

    #[test]
    fn the_meta_object_deserializes() {
        let meta = serde_json::from_value::<Meta>(meta()).expect("deserializes");
        assert_eq!(meta.product.variants.len(), 2);
    }
}
