use html_extractor::serde_json::Value;

/// The variant picker's record of the selected variant.
#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VariantScript {
    #[serde(rename = "type")]
    pub script_type: String,
    pub data: Variant,
}

/// `script[data-variants-json]`: the variant picker's records of every variant.
#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VariantsScript {
    #[serde(rename = "type")]
    pub script_type: String,
    #[serde(rename = "data-variants-json")]
    pub data_variants_json: String,
    pub data: Vec<Variant>,
}

/// Liquid's `variant | json`. Its amounts are in hundredths of the shop currency, which it does
/// not name.
#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Variant {
    pub id: u64,
    pub title: String,
    pub option1: Option<String>,
    pub option2: Option<String>,
    pub option3: Option<String>,
    pub sku: Option<String>,
    pub requires_shipping: bool,
    pub taxable: bool,
    pub featured_image: Option<Value>,
    pub available: bool,
    pub name: String,
    pub public_title: Option<String>,
    pub options: Vec<String>,
    pub price: i64,
    pub weight: i64,
    pub compare_at_price: Option<i64>,
    pub inventory_management: Option<String>,
    pub barcode: Option<String>,
    pub requires_selling_plan: bool,
    pub selling_plan_allocations: Vec<Value>,
    pub quantity_rule: QuantityRule,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct QuantityRule {
    pub min: i64,
    pub max: Option<i64>,
    pub increment: i64,
}

#[cfg(test)]
pub(crate) mod tests {
    use super::Variant;
    use html_extractor::serde_json::{self, Value, json};

    /// An `AMD Ryzen™ AI Max+ 395 / <memory> / EU` variant of the MS-S1 MAX.
    pub(crate) fn variant(id: u64, price: i64, memory: &str) -> Value {
        let title = format!("AMD Ryzen™ AI Max+ 395 / {memory} / EU");
        json!({
            "id": id,
            "title": title,
            "option1": "AMD Ryzen™ AI Max+ 395",
            "option2": memory,
            "option3": "EU",
            "sku": null,
            "requires_shipping": true,
            "taxable": false,
            "featured_image": null,
            "available": true,
            "name": format!("MINISFORUM MS-S1 MAX - {title}"),
            "public_title": title,
            "options": ["AMD Ryzen™ AI Max+ 395", memory, "EU"],
            "price": price,
            "weight": 0,
            "compare_at_price": null,
            "inventory_management": "shopify",
            "barcode": null,
            "requires_selling_plan": false,
            "selling_plan_allocations": [],
            "quantity_rule": {"min": 1, "max": null, "increment": 1}
        })
    }

    #[test]
    fn the_unused_option_slots_read_as_null() {
        let mut single = variant(1, 25990, "unused");
        single["option2"] = Value::Null;
        single["option3"] = Value::Null;
        single["options"] = json!(["270×175×41mm"]);

        let variant = serde_json::from_value::<Variant>(single).expect("deserializes");
        assert!(variant.option2.is_none() && variant.option3.is_none());
    }

    #[test]
    fn an_unknown_key_fails() {
        let mut input = variant(1, 1, "m");
        input["featured_media"] = Value::Null;

        let error = serde_json::from_value::<Variant>(input).expect_err("not in the corpus");
        assert!(error.to_string().contains("unknown field `featured_media`"));
    }
}
