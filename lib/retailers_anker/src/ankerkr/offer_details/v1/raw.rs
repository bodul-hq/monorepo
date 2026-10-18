#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AnkerKrRawDocument {}

#[cfg(test)]
mod tests {
    use super::AnkerKrRawDocument;
    use html_extractor::serde_json;

    #[test]
    fn first_round_document_is_empty() {
        serde_json::from_str::<AnkerKrRawDocument>(r#"{}"#).expect("empty document is valid");
    }

    #[test]
    fn unexpected_fields_are_rejected() {
        assert!(serde_json::from_str::<AnkerKrRawDocument>(r#"{"field":true}"#).is_err());
    }
}
