pub mod ldjsonitem;
pub mod productjson;

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AnkerJapanComRawDocument {
    #[serde(rename = "ld-json")]
    pub ld_json: Vec<ldjsonitem::LdJsonItem>,
    pub product_json: productjson::ProductJsonItem,
}

#[cfg(test)]
mod tests {
    use super::AnkerJapanComRawDocument;
    use html_extractor::serde_json;

    #[test]
    fn the_payload_is_the_liquid_record_not_next_data() {
        let document = format!(
            r#"{{"ld-json":[],"product_json":{{"data":{}}}}}"#,
            super::productjson::tests::PRODUCT
        );
        serde_json::from_str::<AnkerJapanComRawDocument>(&document)
            .expect("the Liquid record is this storefront's payload");

        let with_next_data = document.replacen('{', r#"{"next_data":{},"#, 1);
        let error = serde_json::from_str::<AnkerJapanComRawDocument>(&with_next_data)
            .expect_err("this storefront renders no __NEXT_DATA__");
        assert!(error.to_string().contains("unknown field `next_data`"));
    }

    #[test]
    fn the_islands_and_the_record_are_required() {
        assert!(serde_json::from_str::<AnkerJapanComRawDocument>(r#"{}"#).is_err());
        assert!(serde_json::from_str::<AnkerJapanComRawDocument>(r#"{"ld-json":[]}"#).is_err());
    }
}
