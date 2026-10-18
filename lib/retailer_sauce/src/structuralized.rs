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
    use serde::Serialize;

    #[derive(Debug, Clone, PartialEq, Serialize)]
    pub struct Content {
        pub title: String,
        pub description: String,
        pub images: Vec<String>,
    }
}
