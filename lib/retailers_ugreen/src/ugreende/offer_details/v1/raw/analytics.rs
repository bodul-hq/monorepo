/// The inline script ahead of `script.analytics`, which assigns Shopify's analytics `meta`
/// object. It is the only island that lists every variant with its price, and the only one that
/// carries the product's handle, vendor and type.
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

/// A variant as Shopify reports it to analytics. `price` is in hundredths of the shop currency,
/// before any automatic discount.
#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Variant {
    pub id: u64,
    pub price: i64,
    pub name: String,
    /// Null on a product without options.
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
                "id": 10080883081559_u64,
                "gid": "gid://shopify/Product/10080883081559",
                "vendor": "UGREEN DE",
                "type": "Kabel",
                "handle": "100w-usb-c-kabel-mit-e-marker",
                "variants": [
                    {"id": 51557243388247_u64, "price": 1019,
                        "name": "UGREEN USB-C Kabel ( 100W ) mit E-Marker - 2M",
                        "public_title": "2M", "sku": "65833"},
                    {"id": 51557243421015_u64, "price": 1474,
                        "name": "UGREEN USB-C Kabel ( 100W ) mit E-Marker - 1M",
                        "public_title": "1M", "sku": "65834"}
                ],
                "remote": false
            },
            "page": {
                "pageType": "product",
                "resourceType": "product",
                "resourceId": 10080883081559_u64,
                "requestId": "a3a071c6-0063-4a34-b26a-f52b7e5f7496-1791965177"
            }
        })
    }

    #[test]
    fn the_meta_object_deserializes() {
        let meta = serde_json::from_value::<Meta>(meta()).expect("deserializes");
        assert_eq!(meta.product.variants.len(), 2);
    }

    #[test]
    fn a_variant_without_options_has_no_public_title() {
        let mut input = meta();
        input["product"]["variants"][0]["public_title"] = Value::Null;

        let meta = serde_json::from_value::<Meta>(input).expect("deserializes");
        assert!(meta.product.variants[0].public_title.is_none());
    }
}
