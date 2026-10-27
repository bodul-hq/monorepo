//! Shared normalized availability for retailer offers.

use serde::{Deserialize, Serialize};

/// Recognized stock states. Other values retain their original text.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Availability {
    Available,
    Unavailable,
    InStock,
    OutOfStock,
    SoldOut,
    Other(String),
}

impl Availability {
    pub fn guard(&self) -> Result<(), String> {
        if let Self::Other(raw) = self
            && (raw.trim().is_empty() || raw.contains('\0'))
        {
            return Err("source status must be nonblank and contain no NUL".into());
        }
        Ok(())
    }

    /// Builds availability from a schema.org URL.
    /// Unrecognized values are preserved verbatim, without guessing their stock state.
    ///
    /// This source interpretation accepts both schema.org URL schemes. Deserializing
    /// historical Other values preserves them without applying the source policy again.
    pub fn from_schema_org(value: impl Into<String>) -> Self {
        let value = value.into();
        match value.as_str() {
            "https://schema.org/InStock" | "http://schema.org/InStock" => Self::InStock,
            "https://schema.org/OutOfStock" | "http://schema.org/OutOfStock" => Self::OutOfStock,
            "https://schema.org/SoldOut" | "http://schema.org/SoldOut" => Self::SoldOut,
            _ => Self::Other(value),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Availability;
    use html_extractor::serde_json::{self, json};

    #[test]
    fn schema_org_availability_preserves_the_serialized_contract() {
        for (input, expected) in [
            ("https://schema.org/InStock", json!("in_stock")),
            ("https://schema.org/OutOfStock", json!("out_of_stock")),
            ("https://schema.org/SoldOut", json!("sold_out")),
            ("InStock", json!({"other": "InStock"})),
            ("http://schema.org/OutOfStock", json!("out_of_stock")),
            (
                "https://example.com/InStock",
                json!({"other": "https://example.com/InStock"}),
            ),
            ("", json!({"other": ""})),
        ] {
            let availability = Availability::from_schema_org(input);

            assert_eq!(serde_json::to_value(&availability).unwrap(), expected);
            // The structuralize stage reads a normalized document back off disk, so this has to
            // survive the round trip rather than only render.
            assert_eq!(
                serde_json::from_value::<Availability>(expected).unwrap(),
                availability
            );
        }
    }
}
