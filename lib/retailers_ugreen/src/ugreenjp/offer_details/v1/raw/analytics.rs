/// The inline script that assigns Shopify's analytics `meta` object. It is the only island that
/// carries the product's id, handle and type, and the variant ids the schema.org offers state only
/// inside their URLs.
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
    /// Empty on 130 of the 132 saved pages.
    #[serde(rename = "type")]
    pub product_type: String,
    /// Raw, not percent-encoded: 15 saved handles hold Japanese text.
    pub handle: String,
    pub variants: Vec<Variant>,
    pub remote: bool,
}

/// Liquid's summary of a variant. `price` is in hundredths of the shop currency, yen included:
/// `1398000` is ¥13,980.
#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Variant {
    pub id: u64,
    pub price: i64,
    pub name: String,
    /// `null` on the 63 `Default Title` products.
    pub public_title: Option<String>,
    /// `null` on 19 of the 163 saved variants.
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

    /// Trimmed from `products-lightning-usb-cable.html`.
    pub(crate) fn meta() -> Value {
        json!({
            "product": {
                "id": 9456052437272_u64,
                "gid": "gid://shopify/Product/9456052437272",
                "vendor": "UGREEN-JP",
                "type": "",
                "handle": "lightning-usb-cable",
                "variants": [
                    {"id": 49142586999064_u64, "price": 139900,
                        "name": "UGREENライトニング USB ケーブル - 1Ｍ",
                        "public_title": "1Ｍ", "sku": "60521"},
                    {"id": 49154830631192_u64, "price": 159900,
                        "name": "UGREENライトニング USB ケーブル - 2M",
                        "public_title": "2M", "sku": "70733"}
                ],
                "remote": false
            },
            "page": {
                "pageType": "product",
                "resourceType": "product",
                "resourceId": 9456052437272_u64,
                "requestId": "7e46a4b1-7592-4d74-8f54-4cec565f1d94-1792012815"
            }
        })
    }

    #[test]
    fn the_meta_object_deserializes() {
        let meta = serde_json::from_value::<Meta>(meta()).expect("deserializes");
        assert_eq!(meta.product.variants.len(), 2);
        assert_eq!(meta.product.variants[0].price, 139900);
    }

    #[test]
    fn a_default_title_variant_has_no_public_title_and_maybe_no_sku() {
        let mut input = meta();
        input["product"]["variants"][0]["public_title"] = Value::Null;
        input["product"]["variants"][0]["sku"] = Value::Null;

        let meta = serde_json::from_value::<Meta>(input).expect("deserializes");
        assert!(meta.product.variants[0].public_title.is_none());
        assert!(meta.product.variants[0].sku.is_none());
    }

    #[test]
    fn an_unknown_key_fails() {
        let mut input = meta();
        input["product"]["variants"][0]["barcode"] = json!("1");

        assert!(serde_json::from_value::<Meta>(input).is_err());
    }
}
