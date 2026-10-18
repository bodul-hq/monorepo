//! Typed raw document for `ugreennasjp`, deserialized from the extracted JSON. Every struct in
//! this tree is `deny_unknown_fields`, so a key the corpus survey did not see fails a document
//! loudly instead of being dropped silently.

pub mod analytics;
pub mod ldjsonitem;

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UgreenNasJpRawDocument {
    // Extracted key is `ld-json`, hyphenated, because that is the architecture's structure name.
    #[serde(rename = "ld-json")]
    pub ld_json: Vec<ldjsonitem::LdJsonItem>,

    pub analytics: analytics::AnalyticsScript,
}

#[cfg(test)]
pub(crate) mod tests {
    use super::UgreenNasJpRawDocument;
    use html_extractor::serde_json::{self, Value, json};

    /// A two-variant page, trimmed from `products-air-45w-usb-c-charger.html`.
    pub(crate) fn document() -> Value {
        json!({
            "ld-json": [
                {"type": "application/ld+json", "data": super::ldjsonitem::tests::web_page()},
                {"type": "application/ld+json", "data": super::ldjsonitem::tests::theme_product()},
                {"type": "application/ld+json", "class": "jdgm-server-jld",
                    "data": super::ldjsonitem::tests::reviews_product()}
            ],
            "analytics": {"meta": super::analytics::tests::meta()}
        })
    }

    #[test]
    fn the_fixture_deserializes() {
        serde_json::from_value::<UgreenNasJpRawDocument>(document())
            .expect("the raw fixture deserializes");
    }

    #[test]
    fn both_islands_are_required() {
        assert!(serde_json::from_str::<UgreenNasJpRawDocument>(r#"{}"#).is_err());
        assert!(serde_json::from_str::<UgreenNasJpRawDocument>(r#"{"ld-json":[]}"#).is_err());
    }

    #[test]
    fn the_hyphenated_key_is_what_the_architecture_emits() {
        let mut input = document();
        let items = input
            .as_object_mut()
            .expect("an object")
            .remove("ld-json")
            .expect("ld-json");
        input["ld_json"] = items;

        let error = serde_json::from_value::<UgreenNasJpRawDocument>(input)
            .expect_err("the underscore spelling is not what extract produces");
        assert!(error.to_string().contains("unknown field `ld_json`"));
    }
}
