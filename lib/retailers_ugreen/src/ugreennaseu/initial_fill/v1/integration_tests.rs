use super::*;
use serde_json::json;
use std::cell::RefCell;
#[test]
fn operation_documents_and_variables_are_local() {
    let id = "gid://shopify/Product/1";
    let mut operations = vec![
        Operation::Catalog {
            first: 250,
            after: Some("cursor".into()),
        },
        Operation::Details { id: id.into() },
        Operation::SellingPlans {
            id: id.into(),
            group_after: None,
            group_cursor: "group".into(),
            after: None,
        },
    ];
    for field in [
        "collections",
        "images",
        "media",
        "variants",
        "sellingPlanGroups",
    ] {
        operations.push(Operation::ProductConnection {
            id: id.into(),
            field: field.into(),
            after: Some("cursor".into()),
        });
    }
    for field in [
        "components",
        "groupedBy",
        "quantityPriceBreaks",
        "sellingPlanAllocations",
        "storeAvailability",
    ] {
        operations.push(Operation::VariantConnection {
            id: "gid://shopify/ProductVariant/1".into(),
            field: field.into(),
            after: Some("cursor".into()),
        });
    }
    for op in operations {
        let query = request(&op).unwrap();
        assert_eq!(query.variables["country"], CONFIG.country);
        assert_eq!(query.variables["language"], CONFIG.language);
        assert!(query.query.contains("language: $language"));
        for size in [1, 5, 10, 20, 25] {
            let name = format!("pageSize{size}");
            if !query.variables[&name].is_null() {
                assert_eq!(query.variables[&name], size);
            }
        }
        assert!(
            validate_response(&op, &json!({"data":null,"errors":[{"message":"denied"}]})).is_err()
        );
        assert!(validate_response(&op, &json!({"data":{"product":{},"node":{}}})).is_err());
    }
}
struct Reply {
    body: Value,
    calls: RefCell<Vec<GraphqlRequest>>,
}
impl Http for Reply {
    fn post(
        &self,
        url: &str,
        request: &GraphqlRequest,
        token: Option<&StorefrontToken>,
    ) -> Result<Value, String> {
        assert_eq!(url, CONFIG.graphql_url);
        assert_eq!(token.map(StorefrontToken::as_str), Some("synthetic-token"));
        self.calls.borrow_mut().push(request.clone());
        Ok(self.body.clone())
    }
}
#[test]
fn local_snapshot_keeps_artifact_versions_and_exact_evidence() {
    let mut product: Value =
        serde_json::from_str(include_str!("../../offer_details/v2/fixtures/product.json")).unwrap();
    // Make a complete one-page reply from this retailer's captured fields.
    for field in [
        "collections",
        "images",
        "media",
        "variants",
        "sellingPlanGroups",
    ] {
        product[field] = json!({"edges":[],"pageInfo":{"hasNextPage":false,"endCursor":null}});
    }
    // The selected fixture may be a different advertised market than the default.
    fn currency(value: &mut Value) {
        match value {
            Value::Object(m) => {
                if m.contains_key("currencyCode") {
                    m.insert("currencyCode".into(), json!(CONFIG.currency));
                }
                for value in m.values_mut() {
                    currency(value);
                }
            }
            Value::Array(a) => a.iter_mut().for_each(currency),
            _ => {}
        }
    }
    currency(&mut product);
    let id = product["id"].as_str().unwrap().to_owned();
    let context = MarketContext {
        country: CONFIG.country.into(),
        language: CONFIG.language.into(),
        currency: CONFIG.currency.into(),
    };
    let token = StorefrontToken::new("synthetic-token".into()).unwrap();
    for requested in [None, Some(&context)] {
        let reply = Reply {
            body: json!({"data":{"product":product},"extensions":{"context":{"country":CONFIG.country,"language":CONFIG.language}}}),
            calls: RefCell::new(vec![]),
        };
        let body = Integration
            .details(&reply, Some(&token), requested, &id)
            .unwrap();
        let artifact: Value = serde_json::from_str(&body).unwrap();
        assert_eq!(
            artifact["schema_version"],
            if requested.is_some() { 2 } else { 1 }
        );
        assert_eq!(artifact["product"], product);
        assert_eq!(artifact["pages"][0]["response"], reply.body);
        assert_eq!(
            artifact["pages"][0]["request"],
            serde_json::to_value(&reply.calls.borrow()[0]).unwrap()
        );
        assert!(!body.contains("synthetic-token"));
    }
    let wrong = Reply {
        body: json!({"data":{"product":product},"extensions":{"context":{"country":"XX","language":"ZZ"}}}),
        calls: RefCell::new(vec![]),
    };
    assert!(
        Integration
            .details(&wrong, Some(&token), Some(&context), &id)
            .is_err()
    );
}
#[test]
fn captured_markets_are_parsed_locally() {
    let body: Value = serde_json::from_str(include_str!("markets.json")).unwrap();
    assert!(!markets::contexts(&body).unwrap().is_empty());
}
