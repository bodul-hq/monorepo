use ldjsonitem::LdJsonItem;
use productjson::ProductJsonItem;
use serde::Deserialize;
pub mod ldjsonitem;
pub mod productjson;

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AnkerJapanComRawDocument {
    #[serde(rename = "ld-json")]
    pub ld_json: Vec<LdJsonItem>,
    pub product_json: ProductJsonItem,
}

#[cfg(test)]
mod tests {
    use serde_json::from_str;

    use super::productjson::tests::PRODUCT;

    use super::AnkerJapanComRawDocument;
    use html_extractor::serde_json;

    #[test]
    fn the_payload_is_the_liquid_record_not_next_data() {
        let document = format!(r#"{{"ld-json":[],"product_json":{{"data":{}}}}}"#, PRODUCT);
        from_str::<AnkerJapanComRawDocument>(&document)
            .expect("the Liquid record is this storefront's payload");

        let with_next_data = document.replacen('{', r#"{"next_data":{},"#, 1);
        let error = from_str::<AnkerJapanComRawDocument>(&with_next_data)
            .expect_err("this storefront renders no __NEXT_DATA__");
        assert!(error.to_string().contains("unknown field `next_data`"));
    }

    #[test]
    fn the_islands_and_the_record_are_required() {
        assert!(from_str::<AnkerJapanComRawDocument>(r#"{}"#).is_err());
        assert!(from_str::<AnkerJapanComRawDocument>(r#"{"ld-json":[]}"#).is_err());
    }
}
