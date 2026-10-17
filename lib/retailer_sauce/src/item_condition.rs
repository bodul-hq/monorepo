//! Shared normalized item condition for retailer offers.

use serde::{Deserialize, Serialize};

/// Recognized item conditions. Other values retain their original text.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ItemCondition {
    New,
    Other(String),
}

impl ItemCondition {
    /// Builds item condition from a schema.org URL.
    /// Unrecognized values are preserved verbatim, without guessing their condition.
    pub fn from_schema_org(value: impl Into<String>) -> Self {
        let value = value.into();
        match value.as_str() {
            "https://schema.org/NewCondition" => Self::New,
            _ => Self::Other(value),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ItemCondition;
    use html_extractor::serde_json::{self, json};

    #[test]
    fn schema_org_condition_preserves_the_serialized_contract() {
        for (input, expected) in [
            ("https://schema.org/NewCondition", json!("new")),
            ("NewCondition", json!({"other": "NewCondition"})),
            (
                "https://example.com/NewCondition",
                json!({"other": "https://example.com/NewCondition"}),
            ),
            ("", json!({"other": ""})),
        ] {
            let condition = ItemCondition::from_schema_org(input);

            assert_eq!(serde_json::to_value(&condition).unwrap(), expected);
            // The structuralize stage reads a normalized document back off disk, so this has to
            // survive the round trip rather than only render.
            assert_eq!(
                serde_json::from_value::<ItemCondition>(expected).unwrap(),
                condition
            );
        }
    }
}
