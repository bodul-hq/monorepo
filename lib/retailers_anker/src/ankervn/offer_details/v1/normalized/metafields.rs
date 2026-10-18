//! The storefront keeps its specification table in an untyped page metafield blob holding dozens
//! of unrelated namespaces. The table is lifted by its path instead of typing the whole blob, so a
//! namespace nobody reads cannot fail a document.

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

    list_at(items, "components.productSpecs.items")?
        .iter()
        .map(Specification::from_entry)
        .collect()
}

fn list_at<'a>(value: &'a Value, path: &str) -> Result<&'a Vec<Value>, NormalizeError> {
    value.as_array().ok_or_else(|| NormalizeError::NotAList {
        path: path.to_string(),
    })
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
    fn specifications_are_read_from_the_page_metafields() {
        let page = json!({"components": {"productSpecs": {
            "title": "Specs",
            "items": [{"name": "Dung lượng", "presentation": "10,000mAh"}]
        }}});

        assert_eq!(
            specifications(&page).unwrap(),
            vec![Specification {
                name: "Dung lượng".to_string(),
                value: "10,000mAh".to_string(),
            }]
        );
    }

    #[test]
    fn a_page_without_the_table_has_no_specifications() {
        assert!(
            specifications(&json!({"components": {}}))
                .unwrap()
                .is_empty()
        );
    }

    #[test]
    fn a_row_without_a_string_presentation_fails() {
        let page =
            json!({"components": {"productSpecs": {"items": [{"name": "a", "presentation": 1}]}}});

        assert!(matches!(
            specifications(&page).unwrap_err(),
            NormalizeError::EntryFieldNotAString { field, .. } if field == "presentation"
        ));
    }
}
