//! Retailer-owned typed operations; raw JSON is retained after validation.
use graphql_client::{GraphQLQuery, Response};

use super::Operation;
use retailer_sauce::shopify::initial_fill::{GraphqlRequest, StorefrontConfig};
use serde::de::DeserializeOwned;
use serde_json::{Value, json};

#[allow(clippy::upper_case_acronyms)]
type HTML = String;
#[allow(clippy::upper_case_acronyms)]
type URL = String;
type DateTime = String;
type Decimal = String;
type Color = String;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/ugreennasca/initial_fill/v1/operations/schema.graphql",
    query_path = "src/ugreennasca/initial_fill/v1/operations/catalog.graphql",
    variables_derives = "Deserialize"
)]
struct CatalogIdentifiers;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/ugreennasca/initial_fill/v1/operations/schema.graphql",
    query_path = "src/ugreennasca/initial_fill/v1/operations/offer_details.graphql",
    variables_derives = "Deserialize"
)]
struct OfferDetails;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/ugreennasca/initial_fill/v1/operations/schema.graphql",
    query_path = "src/ugreennasca/initial_fill/v1/operations/ProductCollections.graphql",
    variables_derives = "Deserialize"
)]
struct ProductCollections;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/ugreennasca/initial_fill/v1/operations/schema.graphql",
    query_path = "src/ugreennasca/initial_fill/v1/operations/ProductImages.graphql",
    variables_derives = "Deserialize"
)]
struct ProductImages;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/ugreennasca/initial_fill/v1/operations/schema.graphql",
    query_path = "src/ugreennasca/initial_fill/v1/operations/ProductMedia.graphql",
    variables_derives = "Deserialize"
)]
struct ProductMedia;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/ugreennasca/initial_fill/v1/operations/schema.graphql",
    query_path = "src/ugreennasca/initial_fill/v1/operations/ProductVariants.graphql",
    variables_derives = "Deserialize"
)]
struct ProductVariants;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/ugreennasca/initial_fill/v1/operations/schema.graphql",
    query_path = "src/ugreennasca/initial_fill/v1/operations/ProductSellingPlanGroups.graphql",
    variables_derives = "Deserialize"
)]
struct ProductSellingPlanGroups;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/ugreennasca/initial_fill/v1/operations/schema.graphql",
    query_path = "src/ugreennasca/initial_fill/v1/operations/VariantComponents.graphql",
    variables_derives = "Deserialize"
)]
struct VariantComponents;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/ugreennasca/initial_fill/v1/operations/schema.graphql",
    query_path = "src/ugreennasca/initial_fill/v1/operations/VariantGroupedBy.graphql",
    variables_derives = "Deserialize"
)]
struct VariantGroupedBy;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/ugreennasca/initial_fill/v1/operations/schema.graphql",
    query_path = "src/ugreennasca/initial_fill/v1/operations/VariantQuantityPriceBreaks.graphql",
    variables_derives = "Deserialize"
)]
struct VariantQuantityPriceBreaks;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/ugreennasca/initial_fill/v1/operations/schema.graphql",
    query_path = "src/ugreennasca/initial_fill/v1/operations/VariantSellingPlanAllocations.graphql",
    variables_derives = "Deserialize"
)]
struct VariantSellingPlanAllocations;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/ugreennasca/initial_fill/v1/operations/schema.graphql",
    query_path = "src/ugreennasca/initial_fill/v1/operations/VariantStoreAvailability.graphql",
    variables_derives = "Deserialize"
)]
struct VariantStoreAvailability;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/ugreennasca/initial_fill/v1/operations/schema.graphql",
    query_path = "src/ugreennasca/initial_fill/v1/operations/SellingPlans.graphql",
    variables_derives = "Deserialize"
)]
struct SellingPlans;

fn build<Q: GraphQLQuery>(
    mut variables: Value,
    config: &StorefrontConfig,
) -> Result<GraphqlRequest, String>
where
    Q::Variables: DeserializeOwned,
{
    variables["country"] = json!(config.country);
    variables["language"] = json!(config.language);
    for size in [1, 5, 10, 20, 25] {
        variables[format!("pageSize{size}")] = json!(size);
    }
    let request = Q::build_query(serde_json::from_value(variables).map_err(|e| e.to_string())?);
    Ok(GraphqlRequest {
        operation_name: request.operation_name.into(),
        query: request.query.into(),
        variables: serde_json::to_value(request.variables).map_err(|e| e.to_string())?,
    })
}
fn validate<Q: GraphQLQuery>(body: &Value) -> Result<(), String> {
    let response: Response<Q::ResponseData> =
        serde_json::from_value(body.clone()).map_err(|error| {
            format!("Storefront response does not match the selected GraphQL operation: {error}")
        })?;
    if response.errors.as_ref().is_some_and(|e| !e.is_empty()) || response.data.is_none() {
        return Err("Storefront returned GraphQL errors or missing data".into());
    }
    Ok(())
}

pub fn request(operation: &Operation, config: &StorefrontConfig) -> Result<GraphqlRequest, String> {
    match operation {
        Operation::Markets => Ok(super::markets::request()),
        Operation::InContext { context, operation } => {
            let mut request = request(operation, config)?;
            request.variables["country"] = json!(context.country);
            request.variables["language"] = json!(context.language);
            Ok(request)
        }
        Operation::Catalog { first, after } => {
            build::<CatalogIdentifiers>(json!({"first": first, "after": after}), config)
        }
        Operation::Details { id } => build::<OfferDetails>(json!({"id": id}), config),
        Operation::ProductConnection { id, field, after } => {
            let variables = json!({"id": id, "after": after});
            match field.as_str() {
                "collections" => build::<ProductCollections>(variables, config),
                "images" => build::<ProductImages>(variables, config),
                "media" => build::<ProductMedia>(variables, config),
                "variants" => build::<ProductVariants>(variables, config),
                "sellingPlanGroups" => build::<ProductSellingPlanGroups>(variables, config),
                _ => Err("unsupported product connection".into()),
            }
        }
        Operation::VariantConnection { id, field, after } => {
            let variables = json!({"id": id, "after": after});
            match field.as_str() {
                "components" => build::<VariantComponents>(variables, config),
                "groupedBy" => build::<VariantGroupedBy>(variables, config),
                "quantityPriceBreaks" => build::<VariantQuantityPriceBreaks>(variables, config),
                "sellingPlanAllocations" => {
                    build::<VariantSellingPlanAllocations>(variables, config)
                }
                "storeAvailability" => build::<VariantStoreAvailability>(variables, config),
                _ => Err("unsupported variant connection".into()),
            }
        }
        Operation::SellingPlans {
            id,
            group_after,
            after,
            ..
        } => build::<SellingPlans>(
            json!({"id": id, "groupAfter": group_after, "after": after}),
            config,
        ),
    }
}

pub fn validate_response(
    operation: &Operation,
    body: &Value,
    config: &StorefrontConfig,
) -> Result<(), String> {
    if let Operation::Markets = operation {
        return super::markets::contexts(body).map(|_| ());
    }
    if let Operation::InContext { operation, .. } = operation {
        return validate_response(operation, body, config);
    }
    let name = request(operation, config)?.operation_name;
    match name.as_str() {
        "CatalogIdentifiers" => validate::<CatalogIdentifiers>(body),
        "OfferDetails" => validate::<OfferDetails>(body),
        "ProductCollections" => validate::<ProductCollections>(body),
        "ProductImages" => validate::<ProductImages>(body),
        "ProductMedia" => validate::<ProductMedia>(body),
        "ProductVariants" => validate::<ProductVariants>(body),
        "ProductSellingPlanGroups" => validate::<ProductSellingPlanGroups>(body),
        "VariantComponents" => validate::<VariantComponents>(body),
        "VariantGroupedBy" => validate::<VariantGroupedBy>(body),
        "VariantQuantityPriceBreaks" => validate::<VariantQuantityPriceBreaks>(body),
        "VariantSellingPlanAllocations" => validate::<VariantSellingPlanAllocations>(body),
        "VariantStoreAvailability" => validate::<VariantStoreAvailability>(body),
        "SellingPlans" => validate::<SellingPlans>(body),
        _ => Err("unsupported GraphQL operation".into()),
    }
}
