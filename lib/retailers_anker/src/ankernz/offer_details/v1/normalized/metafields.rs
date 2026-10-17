//! The storefront keeps its specification table in an untyped metafield blob holding dozens of
//! unrelated namespaces. It is lifted by its path instead of typing the whole blob, so a namespace
//! nobody reads cannot fail a document.

use super::NormalizeError;
use html_extractor::serde_json::Value;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Specification {
    pub name: String,
    pub value: String,
}

impl Specification {
    fn from_entry(entry: &Value) -> Result<Specification, NormalizeError> {
        Ok(Specification {
            name: string_field(entry, "name", "productSpecs")?,
            value: string_field(entry, "presentation", "productSpecs")?,
        })
    }
}

// The displayed specifications belong to the page's metafields, not the product's.
pub(super) fn specifications(
    page_metafields: &Value,
) -> Result<Vec<Specification>, NormalizeError> {
    let Some(items) = page_metafields
        .get("components")
        .and_then(|components| components.get("productSpecs"))
        .and_then(|specs| specs.get("items"))
    else {
        return Ok(vec![]);
    };

    let items = items.as_array().ok_or_else(|| NormalizeError::NotAList {
        path: "components.productSpecs.items".to_string(),
    })?;

    items.iter().map(Specification::from_entry).collect()
}

fn string_field(entry: &Value, name: &str, block: &str) -> Result<String, NormalizeError> {
    entry
        .get(name)
        .and_then(Value::as_str)
        .map(str::to_string)
        .ok_or_else(|| NormalizeError::EntryFieldNotAString {
            block: block.to_string(),
            field: name.to_string(),
            entry: entry.clone(),
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use html_extractor::serde_json::json;

    #[test]
    fn specifications_are_read_from_the_page_components() {
        let page = json!({"components": {"productSpecs": {
            "title": "Specs",
            "items": [{"name": "Total Capacity", "presentation": "10,000mAh"}]
        }}});

        assert_eq!(
            specifications(&page).unwrap(),
            vec![Specification {
                name: "Total Capacity".to_string(),
                value: "10,000mAh".to_string(),
            }]
        );
    }

    #[test]
    fn a_page_without_the_block_has_no_specifications() {
        assert_eq!(specifications(&json!({})).unwrap(), vec![]);
        assert_eq!(
            specifications(&json!({"components": {"faq": {}}})).unwrap(),
            vec![]
        );
    }

    #[test]
    fn a_specification_row_that_is_not_two_strings_fails() {
        let page = json!({"components": {"productSpecs": {"items": [{"name": "Weight"}]}}});

        let error = specifications(&page).unwrap_err();

        assert!(matches!(
            error,
            NormalizeError::EntryFieldNotAString { field, .. } if field == "presentation"
        ));
    }

    #[test]
    fn a_specification_block_that_is_not_a_list_fails() {
        let page = json!({"components": {"productSpecs": {"items": {}}}});

        assert!(matches!(
            specifications(&page).unwrap_err(),
            NormalizeError::NotAList { .. }
        ));
    }
}
