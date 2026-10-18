pub mod ldjsonitem;
pub mod nextdata;

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AnkerNordicsComRawDocument {
    #[serde(rename = "ld-json")]
    pub ld_json: Vec<ldjsonitem::LdJsonItem>,

    pub next_data: nextdata::NextDataScript,
}

#[cfg(test)]
mod tests {
    use super::AnkerNordicsComRawDocument;
    use html_extractor::serde_json;

    #[test]
    fn both_islands_are_required() {
        assert!(serde_json::from_str::<AnkerNordicsComRawDocument>(r#"{}"#).is_err());
        assert!(serde_json::from_str::<AnkerNordicsComRawDocument>(r#"{"ld-json":[]}"#).is_err());
    }

    #[test]
    fn the_hyphenated_key_is_what_the_architecture_emits() {
        let error = serde_json::from_str::<AnkerNordicsComRawDocument>(
            r#"{"ld_json":[],"next_data":{"id":"i","type":"t","data":{}}}"#,
        )
        .expect_err("the underscore spelling is not what extract produces");

        assert!(error.to_string().contains("unknown field `ld_json`"));
    }
}
