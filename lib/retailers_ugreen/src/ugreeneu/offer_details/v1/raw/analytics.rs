/// The anonymous inline script that assigns Shopify's analytics `meta` object. It is the one
/// island that carries the Shopify product and variant ids, the handle, the vendor and the type.
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
    /// Translated per locale; empty on 1,247 pages.
    #[serde(rename = "type")]
    pub product_type: String,
    /// Decoded: a non-ASCII handle reads `100w-usb-c-кабел-…` here and percent-escaped in URLs.
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
    /// The product title and `public_title`, joined with ` - `. Ends in a space on 8 variants.
    pub name: String,
    /// Null on a `Default Title` variant.
    pub public_title: Option<String>,
    /// A string on every saved variant.
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

    /// The analytics of `de-at-products-pb561.html`.
    pub(crate) fn meta() -> Value {
        json!({
            "product": {
                "id": 8625471816028_u64,
                "gid": "gid://shopify/Product/8625471816028",
                "vendor": "UGREEN EU",
                "type": "Angebot",
                "handle": "pb561",
                "variants": [
                    {"id": 48981450031452_u64, "price": 5599,
                        "name": "UGREEN Kabelloses Magnetisches Powerbank 10000mAh - Schwarz",
                        "public_title": "Schwarz", "sku": "25919"},
                    {"id": 48981449998684_u64, "price": 4999,
                        "name": "UGREEN Kabelloses Magnetisches Powerbank 10000mAh - Weiß",
                        "public_title": "Weiß", "sku": "25208"}
                ],
                "remote": false
            },
            "page": {
                "pageType": "product",
                "resourceType": "product",
                "resourceId": 8625471816028_u64,
                "requestId": "3140f78d-72da-4472-bd3b-e2385f6dc5aa-1791997801"
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
        let mut input = meta();
        input["product"]["variants"][0]["compare_at_price"] = json!(1);

        let error = serde_json::from_value::<Meta>(input).expect_err("not in the corpus");
        assert!(
            error
                .to_string()
                .contains("unknown field `compare_at_price`")
        );
    }
}
