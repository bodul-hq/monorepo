/// The inline script ahead of `script.analytics`, which assigns Shopify's analytics `meta`
/// object. It is the only island on every page, landing pages included, and the only one that
/// lists every variant with its id, price in hundredths and SKU.
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

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Variant {
    pub id: u64,
    /// In hundredths of the shop currency, yen-like currencies included.
    pub price: i64,
    pub name: String,
    /// Null where the product has no options.
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
                "id": 7128100143150_u64,
                "gid": "gid://shopify/Product/7128100143150",
                "vendor": "Minisforum KR",
                "type": "Mini PC",
                "handle": "minisforum-um890-pro",
                "variants": [
                    {"id": 41332824080430_u64, "price": 75690000,
                        "name": "MINISFORUM UM890 Pro 고성능 미니PC - UM890Pro00EU",
                        "public_title": "AMD Ryzen™ 9 8945HS / FreeDOS / EU",
                        "sku": "UM890Pro00EU"},
                    {"id": 41332824113198_u64, "price": 107199200,
                        "name": "MINISFORUM UM890 Pro 고성능 미니PC - UM890Pro32EU",
                        "public_title": "AMD Ryzen™ 9 8945HS / 32GB RAM+1TB SSD / EU",
                        "sku": "UM890Pro32EU"}
                ],
                "remote": false
            },
            "page": {
                "pageType": "product",
                "resourceType": "product",
                "resourceId": 7128100143150_u64,
                "requestId": "0b1bd2b7-d2c8-4bca-9f91-1386de2daf81-1791962831"
            }
        })
    }

    #[test]
    fn the_meta_object_deserializes() {
        let meta = serde_json::from_value::<Meta>(meta()).expect("deserializes");
        assert_eq!(meta.product.variants.len(), 2);
    }
}
