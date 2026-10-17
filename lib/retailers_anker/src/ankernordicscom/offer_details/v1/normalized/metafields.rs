//! Key features and specifications live in untyped product metafield blobs whose namespaces are
//! page-building content. Each list is lifted by its path instead of typing the whole blob, so a
//! content block nobody reads cannot fail a document.

use super::NormalizeError;
use super::raw::nextdata::product::ProductMetafields;
use html_extractor::serde_json::Value;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct KeyFeature {
    pub title: String,
    pub body: String,
}

pub(super) fn key_features(
    metafields: &ProductMetafields,
) -> Result<Vec<KeyFeature>, NormalizeError> {
    const PATH: &str = "product.infos.key_features.list";

    let Some(list) = metafields
        .product
        .as_ref()
        .and_then(|product| product.get("infos"))
        .and_then(|infos| infos.get("key_features"))
        .and_then(|features| features.get("list"))
    else {
        return Ok(vec![]);
    };

    list_at(list, PATH)?
        .iter()
        .map(|entry| {
            Ok(KeyFeature {
                title: string_field(entry, "key", PATH)?,
                body: string_field(entry, "value", PATH)?,
            })
        })
        .collect()
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Specification {
    pub name: String,
    pub value: String,
}

// Each page states its specifications in one of four shapes, written by different page
// templates. Rows from every shape present are kept, in this order.
pub(super) fn specifications(
    metafields: &ProductMetafields,
) -> Result<Vec<Specification>, NormalizeError> {
    let global = metafields.global.as_ref();
    let component = metafields.component.as_ref();

    let mut rows = keyed_specifications(global.and_then(|global| global.get("specifications")))?;
    rows.extend(global_specs(global.and_then(|global| global.get("specs")))?);
    rows.extend(product_specs(
        component
            .and_then(|component| component.get("productSpecs"))
            .and_then(|specs| specs.get("items")),
    )?);

    Ok(rows)
}

fn keyed_specifications(list: Option<&Value>) -> Result<Vec<Specification>, NormalizeError> {
    const PATH: &str = "global.specifications";

    let Some(list) = list else {
        return Ok(vec![]);
    };

    keyed_rows(list, PATH)
}

// `global.specs` is a flat list on some pages, its rows in one of several shapes, and a grouped
// table on others.
fn global_specs(specs: Option<&Value>) -> Result<Vec<Specification>, NormalizeError> {
    const PATH: &str = "global.specs";

    match specs {
        None => Ok(vec![]),
        Some(list @ Value::Array(_)) => list_at(list, PATH)?
            .iter()
            .map(|entry| flat_spec_row(entry, PATH))
            .collect(),
        Some(table) => grouped_rows(table),
    }
}

fn flat_spec_row(entry: &Value, path: &str) -> Result<Specification, NormalizeError> {
    if entry.get("key").is_some() {
        return Ok(Specification {
            name: string_field(entry, "key", path)?,
            value: string_field(entry, "value", path)?,
        });
    }

    let name = match entry.get("name").and_then(Value::as_str) {
        Some(name) => name.to_string(),
        None => string_field(entry, "type_name", path)?,
    };

    Ok(Specification {
        name,
        value: string_field(entry, "presentation", path)?,
    })
}

fn keyed_rows(list: &Value, path: &str) -> Result<Vec<Specification>, NormalizeError> {
    list_at(list, path)?
        .iter()
        .map(|entry| {
            Ok(Specification {
                name: string_field(entry, "key", path)?,
                value: string_field(entry, "value", path)?,
            })
        })
        .collect()
}

// The group headings are dropped: each row already names what it measures. The table's own
// `variant.sku` is not read, and can name a different product's SKU.
fn grouped_rows(table: &Value) -> Result<Vec<Specification>, NormalizeError> {
    const GROUPS: &str = "global.specs.data";
    const ROWS: &str = "global.specs.data[].children";

    let groups = table.get("data").ok_or_else(|| NormalizeError::NotAList {
        path: GROUPS.to_string(),
    })?;

    let mut rows = Vec::new();

    for group in list_at(groups, GROUPS)? {
        let children = group
            .get("children")
            .ok_or_else(|| NormalizeError::NotAList {
                path: ROWS.to_string(),
            })?;

        for child in list_at(children, ROWS)? {
            rows.push(Specification {
                name: string_field(child, "title", ROWS)?,
                value: string_field(child, "des", ROWS)?,
            });
        }
    }

    Ok(rows)
}

fn product_specs(items: Option<&Value>) -> Result<Vec<Specification>, NormalizeError> {
    const PATH: &str = "component.productSpecs.items";

    let Some(items) = items else {
        return Ok(vec![]);
    };

    list_at(items, PATH)?
        .iter()
        .map(|entry| {
            Ok(Specification {
                name: string_field(entry, "name", PATH)?,
                value: string_field(entry, "presentation", PATH)?,
            })
        })
        .collect()
}

fn list_at<'a>(value: &'a Value, path: &str) -> Result<&'a Vec<Value>, NormalizeError> {
    value.as_array().ok_or_else(|| NormalizeError::NotAList {
        path: path.to_string(),
    })
}

fn string_field(entry: &Value, field: &str, path: &str) -> Result<String, NormalizeError> {
    entry
        .get(field)
        .and_then(Value::as_str)
        .map(str::to_string)
        .ok_or_else(|| NormalizeError::EntryFieldNotAString {
            path: path.to_string(),
            field: field.to_string(),
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use html_extractor::serde_json::{self, json};

    fn metafields(value: Value) -> ProductMetafields {
        serde_json::from_value(value).unwrap()
    }

    fn row(name: &str, value: &str) -> Specification {
        Specification {
            name: name.to_string(),
            value: value.to_string(),
        }
    }

    #[test]
    fn key_features_lift_from_the_product_namespace() {
        let metafields = metafields(json!({"product": {"infos": {"key_features": {"list": [
            {"key": "300W Total Output Power", "value": "Offers 300W max output."}
        ]}}}}));

        assert_eq!(
            key_features(&metafields).unwrap(),
            vec![KeyFeature {
                title: "300W Total Output Power".to_string(),
                body: "Offers 300W max output.".to_string(),
            }]
        );
    }

    #[test]
    fn a_product_without_the_lists_lifts_nothing() {
        let metafields = metafields(json!({"product": {"data": {}}, "global": {}}));

        assert!(key_features(&metafields).unwrap().is_empty());
        assert!(specifications(&metafields).unwrap().is_empty());
    }

    #[test]
    fn the_global_specifications_list_lifts() {
        let metafields = metafields(json!({"global": {"specifications": [
            {"key": "Playtime", "value": "10H/30H"}
        ]}}));

        assert_eq!(
            specifications(&metafields).unwrap(),
            vec![row("Playtime", "10H/30H")]
        );
    }

    #[test]
    fn the_flat_global_specs_list_lifts() {
        let metafields = metafields(json!({"global": {"specs": [
            {"key": "Suction Power", "value": "15,000 pa"}
        ]}}));

        assert_eq!(
            specifications(&metafields).unwrap(),
            vec![row("Suction Power", "15,000 pa")]
        );
    }

    #[test]
    fn the_flat_global_specs_rows_label_by_name_or_type_name() {
        let named = metafields(json!({"global": {"specs": [
            {"name": "Wattage", "presentation": "20 watts"}
        ]}}));
        let typed = metafields(json!({"global": {"specs": [
            {"type_name": "Model Number", "presentation": "D2426"}
        ]}}));
        let typed_with_null_name = metafields(json!({"global": {"specs": [
            {"type_name": "Impedance", "name": null, "presentation": "16Ω"}
        ]}}));

        assert_eq!(
            specifications(&named).unwrap(),
            vec![row("Wattage", "20 watts")]
        );
        assert_eq!(
            specifications(&typed).unwrap(),
            vec![row("Model Number", "D2426")]
        );
        assert_eq!(
            specifications(&typed_with_null_name).unwrap(),
            vec![row("Impedance", "16Ω")]
        );
    }

    #[test]
    fn the_component_product_specs_lift() {
        let metafields = metafields(json!({"component": {"productSpecs": {
            "title": "<strong>Specifications</strong>",
            "items": [{"name": "Total Capacity", "presentation": "26,250mAh"}]
        }}}));

        assert_eq!(
            specifications(&metafields).unwrap(),
            vec![row("Total Capacity", "26,250mAh")]
        );
    }

    #[test]
    fn the_grouped_specs_table_flattens_without_its_headings_or_its_sku() {
        let metafields = metafields(json!({"global": {"specs": {
            "items": [], "title": "Specs", "variant": {"sku": "T88943W1"},
            "data": [
                {"name": "Video", "children": [
                    {"title": "Max Resolution", "des": "4K(3840*2160)"},
                    {"title": "Sensor", "des": "1/1.8-inch Stacked CMOS"}
                ]},
                {"name": "Power", "children": [{"title": "Battery", "des": "13,000mAh"}]}
            ]
        }}}));

        let rows = specifications(&metafields).unwrap();

        assert_eq!(
            rows,
            vec![
                row("Max Resolution", "4K(3840*2160)"),
                row("Sensor", "1/1.8-inch Stacked CMOS"),
                row("Battery", "13,000mAh"),
            ]
        );
        assert!(!serde_json::to_string(&rows).unwrap().contains("T88943W1"));
    }

    #[test]
    fn two_shapes_on_one_page_concatenate_in_path_order() {
        let metafields = metafields(json!({
            "global": {"specifications": [{"key": "a", "value": "1"}]},
            "component": {"productSpecs": {"items": [{"name": "b", "presentation": "2"}]}}
        }));

        assert_eq!(
            specifications(&metafields).unwrap(),
            vec![row("a", "1"), row("b", "2")]
        );
    }

    #[test]
    fn a_row_without_a_string_value_fails_with_its_path() {
        let metafields =
            metafields(json!({"global": {"specifications": [{"key": "a", "value": 1}]}}));

        assert_eq!(
            specifications(&metafields).unwrap_err(),
            NormalizeError::EntryFieldNotAString {
                path: "global.specifications".to_string(),
                field: "value".to_string(),
            }
        );
    }
}
