/// The anonymous inline script that assigns Shopify's analytics `meta` object. It is on every
/// page, and the only island that lists every variant with its price whether or not the product
/// renders a picker.
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
    /// Empty on every product page; `SEEL-WFP` on the `worry-free-purchase` add-on.
    #[serde(rename = "type")]
    pub product_type: String,
    pub handle: String,
    pub variants: Vec<Variant>,
    pub remote: bool,
}

/// A variant as Shopify reports it to analytics. `price` is in hundredths of the shop currency.
#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Variant {
    pub id: u64,
    pub price: i64,
    pub name: String,
    /// Null on a product without options.
    pub public_title: Option<String>,
    /// A string on all 120 saved variants, never empty.
    pub sku: String,
}

/// The request, not the product: a product page of `resourceId` the product id, under a fresh
/// `requestId`.
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
    use super::{AnalyticsScript, Meta};
    use html_extractor::serde_json::{self, Value, json};

    /// Trimmed from `products-ugreen-nasync-dxp2800-gt-2-bay-nas-storage.html`.
    pub(crate) fn meta() -> Value {
        json!({
            "product": {
                "id": 9248062341370_u64,
                "gid": "gid://shopify/Product/9248062341370",
                "vendor": "UGREEN NAS US",
                "type": "",
                "handle": "ugreen-nasync-dxp2800-gt-2-bay-nas-storage",
                "variants": [
                    {"id": 48970903978234_u64, "price": 59999,
                        "name": "UGREEN NASync DXP2800 GT - DXP2800GT",
                        "public_title": "DXP2800GT", "sku": "16050"},
                    {"id": 50446371094778_u64, "price": 73899,
                        "name": "UGREEN NASync DXP2800 GT - DXP2800GT + UPS3000",
                        "public_title": "DXP2800GT + UPS3000", "sku": "16050+45615"}
                ],
                "remote": false
            },
            "page": {
                "pageType": "product",
                "resourceType": "product",
                "resourceId": 9248062341370_u64,
                "requestId": "1b60e448-eafc-4ba3-86c5-fcdbabedbfa3-1792010410"
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

    #[test]
    fn an_unknown_key_fails() {
        let mut variant = meta();
        variant["product"]["variants"][0]["barcode"] = json!("6942752010507");
        assert!(serde_json::from_value::<Meta>(variant).is_err());

        let script = json!({"meta": meta(), "ShopifyAnalytics": {}});
        assert!(serde_json::from_value::<AnalyticsScript>(script).is_err());
    }
}
