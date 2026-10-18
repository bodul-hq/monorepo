/// `script#seal_product`, which a subscriptions app renders on every product page (29 of 29).
#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SealProductScript {
    pub id: String,
    #[serde(rename = "type")]
    pub script_type: String,
    pub data: SealProduct,
}

/// The product id and title, which the variant list does not carry.
#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SealProduct {
    pub id: u64,
    pub title: String,
    /// Protocol-relative, and the 300 pixel wide rendition of the product's primary image.
    pub image_url: String,
}

#[cfg(test)]
pub(crate) mod tests {
    use super::{SealProduct, SealProductScript};
    use html_extractor::serde_json::{self, Value, json};

    /// Trimmed from `products-minisforum-ms-s1-max.html`.
    pub(crate) fn seal_product() -> Value {
        json!({
            "id": 8611751919794_u64,
            "title": "MS-S1 Max",
            "image_url": "//minisforum.hk/cdn/shop/files/MS-S1-Max.png?v=1758849971&width=300"
        })
    }

    #[test]
    fn the_record_deserializes() {
        let product = serde_json::from_value::<SealProduct>(seal_product()).expect("deserializes");
        assert_eq!(product.id, 8611751919794);
    }

    #[test]
    fn a_new_key_in_the_record_fails() {
        let mut input = seal_product();
        input["handle"] = json!("minisforum-ms-s1-max");

        assert!(serde_json::from_value::<SealProduct>(input).is_err());
    }

    #[test]
    fn a_new_script_attribute_fails() {
        let input: Value = json!({
            "id": "seal_product",
            "type": "application/json",
            "data-cmp-ab": "1",
            "data": seal_product()
        });

        assert!(serde_json::from_value::<SealProductScript>(input).is_err());
    }
}
