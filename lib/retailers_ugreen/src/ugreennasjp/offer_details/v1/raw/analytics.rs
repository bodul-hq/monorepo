/// The inline script ahead of `script.analytics`, which assigns Shopify's analytics `meta`
/// object. It is the only island that carries the product's id, handle and type, and the
/// variant ids the schema.org offers state only inside their URLs.
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

/// Liquid's summary of a variant. `price` is in hundredths of the shop currency, yen included.
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
                "id": 15994687127633_u64,
                "gid": "gid://shopify/Product/15994687127633",
                "vendor": "UGREEN-JP",
                "type": "Charger",
                "handle": "air-45w-usb-c-charger",
                "variants": [
                    {"id": 60268391956561_u64, "price": 299600,
                        "name": "UGREEN Air 45W USB-C 当社最小充電器 GaNII 急速充電器 - グレー",
                        "public_title": "グレー", "sku": "35993"},
                    {"id": 60268391989329_u64, "price": 299600,
                        "name": "UGREEN Air 45W USB-C 当社最小充電器 GaNII 急速充電器 - ホワイト",
                        "public_title": "ホワイト", "sku": "35994"}
                ],
                "remote": false
            },
            "page": {
                "pageType": "product",
                "resourceType": "product",
                "resourceId": 15994687127633_u64,
                "requestId": "2537208b-1ea7-4e8f-9d4c-ead62add5f82-1791965322"
            }
        })
    }

    #[test]
    fn the_meta_object_deserializes() {
        let meta = serde_json::from_value::<Meta>(meta()).expect("deserializes");
        assert_eq!(meta.product.variants.len(), 2);
    }

    #[test]
    fn a_single_variant_has_no_public_title() {
        let mut input = meta();
        input["product"]["variants"][0]["public_title"] = Value::Null;

        let meta = serde_json::from_value::<Meta>(input).expect("deserializes");
        assert!(meta.product.variants[0].public_title.is_none());
    }
}
