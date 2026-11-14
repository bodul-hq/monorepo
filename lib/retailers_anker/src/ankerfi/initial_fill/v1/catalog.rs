use super::{Operation, next_cursor, product_gid};
use retailer_sauce::initial_fill::{CatalogBatch, Http};
use retailer_sauce::shopify::{initial_fill::StorefrontToken, markets::MarketContext};
use serde_json::Value;
use std::collections::HashSet;
const BATCH_SIZE: usize = 500;
pub fn fetch(
    http: &dyn Http,
    token: Option<&StorefrontToken>,
    context: Option<&MarketContext>,
    after: Option<String>,
) -> Result<CatalogBatch, String> {
    let mut after = after;
    let mut cursors = HashSet::new();
    if let Some(cursor) = &after {
        cursors.insert(cursor.clone());
    }
    let mut product_ids = Vec::new();
    let mut seen = HashSet::new();
    loop {
        let first = (BATCH_SIZE - product_ids.len()).min(250) as u16;
        let response = execute(
            http,
            token,
            &Operation::Catalog {
                first,
                after: after.clone(),
            },
            context,
        )?;
        let products = response
            .pointer("/data/products")
            .ok_or("catalog response is missing products")?;
        let edges = products["edges"]
            .as_array()
            .ok_or("catalog response is missing edges")?;
        if edges.len() > first as usize {
            return Err("catalog exceeded requested page size".into());
        }
        for edge in edges {
            let id = edge
                .pointer("/node/id")
                .and_then(Value::as_str)
                .ok_or("catalog product is missing an ID")?;
            product_gid(id)?;
            if seen.insert(id.to_owned()) {
                product_ids.push(id.to_owned());
            }
        }
        after = next_cursor(products)?;
        if let Some(cursor) = &after
            && !cursors.insert(cursor.clone())
        {
            return Err("catalog cursor did not advance".into());
        }
        if after.is_none() || product_ids.len() == BATCH_SIZE {
            break;
        }
    }
    Ok(CatalogBatch {
        product_ids,
        next_cursor: after,
    })
}
fn execute(
    http: &dyn Http,
    token: Option<&StorefrontToken>,
    operation: &Operation,
    context: Option<&MarketContext>,
) -> Result<Value, String> {
    if let Some(context) = context {
        super::execute(
            http,
            token,
            &Operation::InContext {
                context: context.clone(),
                operation: Box::new(operation.clone()),
            },
        )
    } else {
        super::execute(http, token, operation)
    }
}
