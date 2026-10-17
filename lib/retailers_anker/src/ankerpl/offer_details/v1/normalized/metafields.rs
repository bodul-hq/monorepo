//! The storefront keeps key features and specifications in untyped metafield blobs holding dozens
//! of unrelated namespaces. Each fact is lifted by its path instead of typing the
//! whole blob, so a namespace nobody reads cannot fail a document.

use super::NormalizeError;
use html_extractor::serde_json::Value;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct KeyFeature {
    pub title: String,
    pub body: String,
}

impl KeyFeature {
    fn from_entry(entry: &Value) -> Result<KeyFeature, NormalizeError> {
        Ok(KeyFeature {
            title: string_field(entry, "key", "key_features")?,
            body: string_field(entry, "value", "key_features")?,
        })
    }
}

pub(super) fn key_features(product_metafields: &Value) -> Result<Vec<KeyFeature>, NormalizeError> {
    let Some(list) = product_metafields
        .get("productInfos")
        .and_then(|infos| infos.get("key_features"))
        .and_then(|features| features.get("list"))
    else {
        return Ok(vec![]);
    };

    list_at(list, "productInfos.key_features.list")?
        .iter()
        .map(KeyFeature::from_entry)
        .collect()
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Specification {
    /// Text-only specification rows carry no separate label.
    pub name: Option<String>,
    pub value: String,
}

impl Specification {
    fn from_entry(entry: &Value) -> Result<Specification, NormalizeError> {
        Ok(Specification {
            name: specification_name(entry)?,
            value: string_field(entry, "presentation", "productSpecs")?,
        })
    }
}

// The CMS supports `name`, `type_name`, and unlabeled presentation-only rows.
// Preserve the full presentation instead of guessing a label from punctuation in the copy.
fn specification_name(entry: &Value) -> Result<Option<String>, NormalizeError> {
    match entry.get("name").or_else(|| entry.get("type_name")) {
        None => Ok(None),
        Some(Value::String(label)) => Ok(Some(label.clone())),
        Some(_) => Err(NormalizeError::SpecificationWithoutName {
            entry: entry.clone(),
        }),
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
    fn a_specification_labelled_type_name_crosses_under_that_label() {
        let specifications = specifications(&json!({"components": {"productSpecs": {"items": [
            {"name": "Size", "presentation": "63 x 35 x 35mm"},
            {"type_name": "Weight", "presentation": "30 g/1.1 oz"}
        ]}}}))
        .unwrap();

        assert_eq!(specifications[0].name.as_deref(), Some("Size"));
        assert_eq!(specifications[1].name.as_deref(), Some("Weight"));
        assert_eq!(specifications[1].value, "30 g/1.1 oz");
    }

    #[test]
    fn unlabeled_specifications_keep_the_complete_presentation() {
        let specifications = specifications(&json!({"components": {"productSpecs": {"items": [
            {"presentation": "Total Wattage: 20W "},
            {"presentation": "iPhone 13 / iPhone 13 Pro"}
        ]}}}))
        .unwrap();
        assert_eq!(specifications[0].name, None);
        assert_eq!(specifications[0].value, "Total Wattage: 20W ");
        assert_eq!(specifications[1].name, None);
        assert_eq!(specifications[1].value, "iPhone 13 / iPhone 13 Pro");
    }

    #[test]
    fn a_non_string_specification_label_is_still_an_error() {
        let error = specifications(&json!({"components": {"productSpecs": {"items": [
            {"name": 3, "presentation": "30 g"}
        ]}}}))
        .unwrap_err();
        assert!(matches!(
            error,
            NormalizeError::SpecificationWithoutName { .. }
        ));
    }

    #[test]
    fn specifications_are_read_from_the_page_and_key_features_from_the_product() {
        let page = json!({"components": {"productSpecs": {
            "title": "Specs",
            "items": [{"name": "Total Capacity", "presentation": "10,000mAh"}]
        }}});
        let product = json!({"productInfos": {"key_features": {"list": [
            {"key": "Ultra-Fast Charging", "value": "Charges a phone to 50% in 30 minutes."}
        ]}}});

        assert_eq!(
            specifications(&page).unwrap(),
            vec![Specification {
                name: Some("Total Capacity".to_string()),
                value: "10,000mAh".to_string(),
            }]
        );
        assert_eq!(
            key_features(&product).unwrap(),
            vec![KeyFeature {
                title: "Ultra-Fast Charging".to_string(),
                body: "Charges a phone to 50% in 30 minutes.".to_string(),
            }]
        );
        assert_eq!(specifications(&product).unwrap(), vec![]);
        assert_eq!(key_features(&page).unwrap(), vec![]);
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
}
