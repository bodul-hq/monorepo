/// `meta[property="og:price:currency"]`'s attributes: the one statement on every page of the
/// currency the prices are rendered in, `INR` on all 1,408 saved pages.
#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OgCurrency {
    pub property: String,
    pub content: String,
}

#[cfg(test)]
pub(crate) mod tests {
    use super::OgCurrency;
    use html_extractor::serde_json::{self, Value, json};

    pub(crate) fn og_currency() -> Value {
        json!({"content": "INR", "property": "og:price:currency"})
    }

    #[test]
    fn the_attributes_deserialize() {
        let tag = serde_json::from_value::<OgCurrency>(og_currency()).expect("deserializes");
        assert_eq!(tag.content, "INR");
    }

    #[test]
    fn a_new_attribute_fails() {
        let mut input = og_currency();
        input["id"] = json!("currency");

        assert!(serde_json::from_value::<OgCurrency>(input).is_err());
    }
}
