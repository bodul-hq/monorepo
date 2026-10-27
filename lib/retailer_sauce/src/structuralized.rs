//! The canonical offer every retailer's normalized document converges on.
//!
//! Normalization stops at the retailer's own shape - `ankerau` alone keeps the schema.org island
//! and the storefront's product record side by side, because on its corpus they disagree.
//! Structuralization is where that becomes one offer. The type lives here rather than in a retailer
//! family crate for the reason the stage exists at all: it is the first document shape shared
//! across families.
//!
//! What a disagreement does **not** become is a single value. Where two renderings of one fact are
//! both on the page, both cross into [`Offer`] under names that say where they came from; see
//! [`Price`] and [`Offer::available_for_sale`].

use serde::Serialize;

/// What one call to a retailer's structuralizer hands back.
///
/// A deliberate mirror of `bodul_shared::NormalizedData`, and `None` means the same thing it does
/// there: this retailer is claimed by a family, but has no structuralized document yet, so the
/// stage writes no file and does not fail. Its two offer lists are not mirrored - no stage reads
/// them.
pub struct StructuralizedData {
    pub document_json: Option<String>,
}

impl StructuralizedData {
    pub fn just_document_json(document_json: impl Into<Option<String>>) -> Self {
        StructuralizedData {
            document_json: document_json.into(),
        }
    }
}

// ************************************************************************************************
// ************************************************************************************************
// ************************************************************************************************

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Offer {
    pub product: product::Product,
    pub content: content::Content,
}

pub mod product {
    use serde::Serialize;

    #[derive(Debug, Clone, PartialEq, Serialize)]
    pub struct Product {
        // TODO: not sure copying retailers name or title will make this good
        pub name: String,
        pub brand: Brand,
    }

    /////
    #[derive(Debug, Clone, PartialEq, Serialize)]
    pub struct Brand {}
}
pub mod content {
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, PartialEq, Default, Serialize)]
    pub struct Content {
        pub description: Option<String>,
    }

    impl<'de> Deserialize<'de> for Content {
        fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
            #[derive(Deserialize)]
            #[serde(deny_unknown_fields)]
            struct Fields {
                #[serde(deserialize_with = "crate::observation::required_nullable")]
                description: Option<String>,
            }
            let fields: Fields = crate::observation::object(deserializer)?;
            Ok(Self {
                description: fields.description,
            })
        }
    }

    impl Content {
        pub fn guard(&self) -> Result<(), String> {
            if self
                .description
                .as_ref()
                .is_some_and(|value| value.contains('\0'))
            {
                return Err("description must not contain NUL".into());
            }
            Ok(())
        }
    }

    pub fn observation<'de, D: serde::Deserializer<'de>>(
        deserializer: D,
    ) -> Result<Option<Content>, D::Error> {
        Content::deserialize(deserializer).map(Some)
    }
}

#[cfg(test)]
mod content_tests {
    use super::content::Content;
    use serde_json::json;
    #[test]
    fn content_requires_an_object_with_a_description_and_preserves_exact_text() {
        for invalid in [
            json!(null),
            json!({}),
            json!([null]),
            json!(["description"]),
            json!({"description":7}),
            json!({"description":null,"title":"moved"}),
        ] {
            assert!(serde_json::from_value::<Content>(invalid).is_err());
        }
        for description in [None, Some(""), Some(" \n\t"), Some(" <p> ž &amp; </p> ")] {
            let value = json!({"description":description});
            let content: Content = serde_json::from_value(value.clone()).unwrap();
            assert_eq!(serde_json::to_value(content).unwrap(), value);
        }
    }
}
