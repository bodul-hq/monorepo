use super::artifact::{OfferDetails, Page};
use super::{Operation, next_cursor, product_gid};
use retailer_sauce::initial_fill::Http;
use retailer_sauce::shopify::{initial_fill::StorefrontToken, markets::MarketContext};
use serde_json::Value;
use std::collections::HashSet;
pub fn fetch_in_context(
    fetcher: &dyn Http,
    token: Option<&StorefrontToken>,
    id: &str,
    context: Option<&MarketContext>,
) -> Result<OfferDetails, String> {
    product_gid(id)?;
    let config = super::CONFIG;
    let mut session = Session {
        fetcher,
        context,
        token,
        pages: Vec::new(),
    };
    let response = session.execute(Operation::Details { id: id.into() })?;
    let mut product = owner(&response, "/data/product", id)?.clone();
    for field in [
        "collections",
        "images",
        "media",
        "variants",
        "sellingPlanGroups",
    ] {
        let connection = product
            .get_mut(field)
            .ok_or_else(|| format!("missing product connection {field}"))?;
        session.complete(connection, |after| Operation::ProductConnection {
            id: id.into(),
            field: field.into(),
            after,
        })?;
    }
    for edge in product["variants"]["edges"]
        .as_array_mut()
        .ok_or("missing variant edges")?
    {
        let variant = edge.get_mut("node").ok_or("missing variant")?;
        let variant_id = variant["id"]
            .as_str()
            .ok_or("missing variant ID")?
            .to_owned();
        for field in [
            "components",
            "groupedBy",
            "quantityPriceBreaks",
            "sellingPlanAllocations",
            "storeAvailability",
        ] {
            let connection = variant
                .get_mut(field)
                .ok_or_else(|| format!("missing variant connection {field}"))?;
            session.complete(connection, |after| Operation::VariantConnection {
                id: variant_id.clone(),
                field: field.into(),
                after,
            })?;
        }
    }
    let mut group_after = None;
    for edge in product["sellingPlanGroups"]["edges"]
        .as_array_mut()
        .ok_or("missing selling-plan group edges")?
    {
        let cursor = edge["cursor"]
            .as_str()
            .ok_or("missing selling-plan group cursor")?
            .to_owned();
        let plans = edge
            .pointer_mut("/node/sellingPlans")
            .ok_or("missing selling plans")?;
        session.complete(plans, |after| Operation::SellingPlans {
            id: id.into(),
            group_after: group_after.clone(),
            group_cursor: cursor.clone(),
            after,
        })?;
        group_after = Some(cursor);
    }
    validate_currency(
        &product,
        context.map_or(config.currency, |c| c.currency.as_str()),
    )?;
    Ok(OfferDetails {
        schema_version: if context.is_some() { 2 } else { 1 },
        context: context.cloned(),
        product,
        pages: session.pages,
    })
}

fn validate_currency(value: &Value, expected: &str) -> Result<(), String> {
    match value {
        Value::Object(map) => {
            if let Some(currency) = map.get("currencyCode")
                && currency != expected
            {
                return Err(format!("offer price currency does not match {expected}"));
            }
            for value in map.values() {
                validate_currency(value, expected)?;
            }
        }
        Value::Array(values) => {
            for value in values {
                validate_currency(value, expected)?;
            }
        }
        _ => {}
    }
    Ok(())
}

fn owner<'a>(response: &'a Value, pointer: &str, id: &str) -> Result<&'a Value, String> {
    let value = response
        .pointer(pointer)
        .filter(|v| v.is_object())
        .ok_or("requested product or variant is unavailable")?;
    if value["id"].as_str() != Some(id) {
        return Err("response owner does not match requested ID".into());
    }
    Ok(value)
}

struct Session<'a> {
    fetcher: &'a dyn Http,
    context: Option<&'a MarketContext>,
    token: Option<&'a StorefrontToken>,
    pages: Vec<Page>,
}
impl Session<'_> {
    fn execute(&mut self, operation: Operation) -> Result<Value, String> {
        let operation = if let Some(context) = self.context {
            Operation::InContext {
                context: context.clone(),
                operation: Box::new(operation),
            }
        } else {
            operation
        };
        let request = super::request(&operation)?;
        let response = super::execute(self.fetcher, self.token, &operation)?;
        self.pages.push(Page {
            request,
            response: response.clone(),
        });
        Ok(response)
    }
    fn complete(
        &mut self,
        connection: &mut Value,
        operation: impl Fn(Option<String>) -> Operation,
    ) -> Result<(), String> {
        let mut cursors = HashSet::new();
        deduplicate_edges(connection)?;
        while let Some(after) = next_cursor(connection)? {
            if !cursors.insert(after.clone()) {
                return Err("detail connection cursor did not advance".into());
            }
            let op = operation(Some(after));
            let response = self.execute(op.clone())?;
            let page = match &op {
                Operation::ProductConnection { id, field, .. } => {
                    owner(&response, "/data/product", id)?
                        .get(field)
                        .ok_or("missing product continuation")?
                }
                Operation::VariantConnection { id, field, .. } => {
                    owner(&response, "/data/node", id)?
                        .get(field)
                        .ok_or("missing variant continuation")?
                }
                Operation::SellingPlans {
                    id, group_cursor, ..
                } => {
                    let product = owner(&response, "/data/product", id)?;
                    let groups = product
                        .pointer("/sellingPlanGroups/edges")
                        .and_then(Value::as_array)
                        .ok_or("missing selling-plan group")?;
                    if groups.len() != 1 || groups[0]["cursor"].as_str() != Some(group_cursor) {
                        return Err("selling-plan group changed during pagination".into());
                    }
                    groups[0]
                        .pointer("/node/sellingPlans")
                        .ok_or("missing selling-plan continuation")?
                }
                _ => return Err("invalid continuation operation".into()),
            };
            // Validate continuation metadata before accepting any of its edges.
            next_cursor(page)?;
            let edges = page["edges"]
                .as_array()
                .ok_or("missing continuation edges")?;
            connection["edges"]
                .as_array_mut()
                .ok_or("missing connection edges")?
                .extend(edges.iter().cloned());
            connection["pageInfo"] = page["pageInfo"].clone();
            deduplicate_edges(connection)?;
        }
        Ok(())
    }
}

fn deduplicate_edges(connection: &mut Value) -> Result<(), String> {
    let edges = connection["edges"]
        .as_array_mut()
        .ok_or("missing connection edges")?;
    let mut ids = HashSet::new();
    let mut cursors = HashSet::new();
    for edge in edges.iter() {
        if edge["cursor"].as_str().is_none_or(str::is_empty) {
            return Err("missing edge cursor".into());
        }
        if !edge["node"].is_object() {
            return Err("missing edge node".into());
        }
    }
    edges.retain(|edge| {
        let cursor = edge["cursor"].as_str().unwrap().to_owned();
        let id = edge.pointer("/node/id").and_then(Value::as_str);
        let fresh = cursors.insert(cursor);
        match id {
            Some(id) => ids.insert(id.to_owned()) && fresh,
            None => fresh,
        }
    });
    Ok(())
}
