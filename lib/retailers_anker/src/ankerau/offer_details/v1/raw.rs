//! Typed raw document for `ankerau`, deserialized from the extracted JSON. Every struct in this
//! tree is `deny_unknown_fields`, so a key the corpus survey did not see fails a document loudly
//! instead of being dropped silently.

pub mod ldjsonitem;
pub mod nextdata;

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AnkerAuRawDocument {
    // Extracted key is `ld-json`, hyphenated, because that is the architecture's structure name.
    #[serde(rename = "ld-json")]
    pub ld_json: Vec<ldjsonitem::LdJsonItem>,

    pub next_data: nextdata::NextDataScript,
}

#[cfg(test)]
mod tests {
    use super::AnkerAuRawDocument;
    use html_extractor::serde_json;

    #[test]
    fn both_islands_are_required() {
        assert!(serde_json::from_str::<AnkerAuRawDocument>(r#"{}"#).is_err());
        assert!(serde_json::from_str::<AnkerAuRawDocument>(r#"{"ld-json":[]}"#).is_err());
    }

    #[test]
    fn the_hyphenated_key_is_what_the_architecture_emits() {
        let error = serde_json::from_str::<AnkerAuRawDocument>(
            r#"{"ld_json":[],"next_data":{"id":"i","type":"t","data":{}}}"#,
        )
        .expect_err("the underscore spelling is not what extract produces");

        assert!(error.to_string().contains("unknown field `ld_json`"));
    }
}
