//! HTTP request validation is deliberately separate from execution. async-graphql
//! normally coerces some variables lazily, which could let an earlier mutation
//! run before a later field discovers invalid input. The preflight pass executes
//! no resolvers and validates every variable against the registered input types.
use crate::{error::Error, model, store::Store};
use async_graphql::{
    Context, EmptySubscription, ErrorExtensions, ID, InputObject, InputValueResult, Object,
    Request, Response, Scalar, ScalarType, Schema, ServerError, ServerResult, SimpleObject,
    ValidationResult, Value, Variables,
    extensions::{
        Extension, ExtensionContext, ExtensionFactory, NextExecute, NextParseQuery, NextValidation,
    },
    parser::types::{DocumentOperations, ExecutableDocument},
    registry::{MetaType, Registry},
};
use axum::{
    Json,
    extract::State,
    http::{StatusCode, header},
    response::IntoResponse,
};
use serde::Deserialize;
use std::sync::{Arc, Mutex};

// Explicit conversion avoids serde's private arbitrary-precision number map
// being mistaken for a JSON object by async-graphql's Value deserializer.
pub fn from_json(value: serde_json::Value) -> Value {
    match value {
        serde_json::Value::Null => Value::Null,
        serde_json::Value::Bool(v) => Value::Boolean(v),
        serde_json::Value::Number(v) => Value::Number(v),
        serde_json::Value::String(v) => Value::String(v),
        serde_json::Value::Array(v) => Value::List(v.into_iter().map(from_json).collect()),
        serde_json::Value::Object(v) => Value::Object(
            v.into_iter()
                .map(|(k, v)| (async_graphql::Name::new(k), from_json(v)))
                .collect(),
        ),
    }
}

fn to_json(value: Value) -> serde_json::Value {
    match value {
        Value::Null => serde_json::Value::Null,
        Value::Boolean(v) => v.into(),
        Value::Number(v) => serde_json::Value::Number(v),
        Value::String(v) => v.into(),
        Value::Enum(v) => v.to_string().into(),
        Value::List(v) => v.into_iter().map(to_json).collect(),
        Value::Object(v) => serde_json::Value::Object(
            v.into_iter()
                .map(|(k, v)| (k.to_string(), to_json(v)))
                .collect(),
        ),
        Value::Binary(_) => unreachable!("HTTP JSON has no binary values"),
    }
}

pub struct JsonValue(serde_json::Value);
#[Scalar(name = "JSON")]
impl ScalarType for JsonValue {
    fn parse(value: Value) -> InputValueResult<Self> {
        Ok(Self(to_json(value)))
    }
    fn to_value(&self) -> Value {
        from_json(self.0.clone())
    }
}

pub struct DateTime(chrono::DateTime<chrono::Utc>);
#[Scalar(name = "DateTime")]
impl ScalarType for DateTime {
    fn parse(value: Value) -> InputValueResult<Self> {
        if let Value::String(value) = &value {
            return chrono::DateTime::parse_from_rfc3339(value)
                .map(|v| Self(v.with_timezone(&chrono::Utc)))
                .map_err(|_| {
                    async_graphql::InputValueError::custom("Expected an RFC 3339 timestamp.")
                });
        }
        Err(async_graphql::InputValueError::expected_type(value))
    }
    fn to_value(&self) -> Value {
        Value::String(self.0.to_rfc3339_opts(chrono::SecondsFormat::AutoSi, true))
    }
}

#[derive(SimpleObject)]
pub struct StoredRecord {
    id: ID,
    bucket: String,
    idempotency_key: String,
    payload: String,
    metadata: JsonValue,
    created_at: DateTime,
}
impl From<model::Record> for StoredRecord {
    fn from(record: model::Record) -> Self {
        Self {
            id: record.id.to_string().into(),
            bucket: record.bucket,
            idempotency_key: record.idempotency_key.to_string(),
            payload: record.payload,
            metadata: JsonValue(record.metadata),
            created_at: DateTime(record.created_at),
        }
    }
}

#[derive(InputObject)]
pub struct CreateRecordInput {
    bucket: String,
    idempotency_key: String,
    payload: String,
    metadata: JsonValue,
}
#[derive(SimpleObject)]
pub struct CreateRecordPayload {
    id: ID,
}

pub struct Query;
#[Object]
impl Query {
    async fn record(
        &self,
        ctx: &Context<'_>,
        bucket: String,
        id: ID,
    ) -> async_graphql::Result<Option<StoredRecord>> {
        match ctx
            .data_unchecked::<Store>()
            .get(bucket, id.to_string())
            .await
        {
            Ok(record) => Ok(Some(record.into())),
            Err(error) => {
                ctx.add_error(ctx.set_error_path(error.graphql().into_server_error(ctx.item.pos)));
                Ok(None)
            }
        }
    }
}
pub struct Mutation;
#[Object]
impl Mutation {
    async fn create_record(
        &self,
        ctx: &Context<'_>,
        input: CreateRecordInput,
    ) -> async_graphql::Result<CreateRecordPayload> {
        ctx.data_unchecked::<Store>()
            .create(
                input.bucket,
                model::Create {
                    idempotency_key: input.idempotency_key,
                    payload: input.payload,
                    metadata: input.metadata.0,
                },
            )
            .await
            .map(|created| CreateRecordPayload {
                id: created.id.to_string().into(),
            })
            .map_err(Error::graphql)
    }
}

pub type ApiSchema = Schema<Query, Mutation, EmptySubscription>;
pub fn schema(store: Store) -> ApiSchema {
    Schema::build(Query, Mutation, EmptySubscription)
        .data(store)
        .extension(PreflightFactory)
        .finish()
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HttpRequest {
    query: String,
    #[serde(default)]
    variables: Option<serde_json::Map<String, serde_json::Value>>,
    operation_name: Option<String>,
}

pub async fn handle(
    State(schema): State<ApiSchema>,
    input: Result<Json<HttpRequest>, axum::extract::rejection::JsonRejection>,
) -> axum::response::Response {
    let input = match input { Ok(Json(input)) => input, Err(_) => return Error::Input("Expected a JSON object with a query string, optional variables object, and optional operationName string.").into_response() };
    // Keep parser error variants intact: duplicate definitions are schema
    // violations, while an absent operation is an operation-selection failure.
    let document = match async_graphql::parser::parse_query(&input.query) {
        Ok(document) => document,
        Err(error) => {
            use async_graphql::parser::Error as ParseError;
            let (code, message) = match &error {
                ParseError::OperationDuplicated { .. }
                | ParseError::FragmentDuplicated { .. }
                | ParseError::MultipleOperations { .. } => (
                    "GRAPHQL_VALIDATION_FAILED",
                    "Operation and fragment definitions must be unique, and anonymous operations must stand alone.",
                ),
                ParseError::MissingOperation => (
                    "OPERATION_RESOLUTION_FAILURE",
                    "The document contains no executable operation.",
                ),
                _ => ("GRAPHQL_PARSE_FAILED", "Invalid GraphQL document syntax."),
            };
            let mut result = tagged(ServerError::new(message, None), code);
            result.locations = error.positions().collect();
            return request_errors(vec![result]);
        }
    };
    let mut request = Request::new(input.query);
    request.set_parsed_query(document.clone());
    request.operation_name = input.operation_name;
    request.variables = Variables::from_value(from_json(serde_json::Value::Object(
        input.variables.unwrap_or_default(),
    )));
    let preflight = Preflight {
        variables: request.variables.clone(),
        operation_name: request.operation_name.clone(),
        document: Mutex::new(None),
    };
    let mut validation = Request::new(request.query.clone()).data(preflight);
    // Validate the whole document without runtime variable values. This keeps
    // schema errors distinct from variable coercion errors.
    validation.operation_name = request.operation_name.clone();
    validation.set_parsed_query(document);
    let checked = schema.execute(validation).await;
    if !checked.errors.is_empty() {
        return request_errors(checked.errors);
    }
    // async-graphql applies @skip/@include before resolving argument defaults.
    // Materialize selected-operation defaults so directives see coerced values.
    let operation_name = request.operation_name.clone();
    let defaults: Vec<_> = request
        .parsed_query()
        .expect("already parsed")
        .operations
        .iter()
        .filter(|(name, _)| {
            operation_name
                .as_deref()
                .is_none_or(|selected| name.is_some_and(|name| name.as_str() == selected))
        })
        .flat_map(|(_, operation)| operation.node.variable_definitions.iter())
        .filter_map(|definition| {
            definition
                .node
                .default_value
                .as_ref()
                .map(|value| (definition.node.name.node.clone(), value.node.clone()))
        })
        .collect();
    for (name, value) in defaults {
        request.variables.entry(name).or_insert(value);
    }
    let mut result = schema.execute(request).await;
    for error in &mut result.errors {
        if error
            .extensions
            .as_ref()
            .and_then(|ext| ext.get("code"))
            .is_none()
        {
            error.message = Error::Internal.detail().into();
            error.extensions = Some(Error::Internal.graphql().extend().extensions.unwrap());
        }
    }
    (
        [(header::CONTENT_TYPE, "application/graphql-response+json")],
        Json(result),
    )
        .into_response()
}

struct Preflight {
    variables: Variables,
    operation_name: Option<String>,
    document: Mutex<Option<ExecutableDocument>>,
}
struct PreflightFactory;
impl ExtensionFactory for PreflightFactory {
    fn create(&self) -> Arc<dyn Extension> {
        Arc::new(PreflightExtension)
    }
}
struct PreflightExtension;
fn tagged(mut error: ServerError, code: &str) -> ServerError {
    error
        .extensions
        .get_or_insert_with(Default::default)
        .set("code", code);
    error
}

#[async_graphql::async_trait::async_trait]
impl Extension for PreflightExtension {
    async fn parse_query(
        &self,
        ctx: &ExtensionContext<'_>,
        query: &str,
        variables: &Variables,
        next: NextParseQuery<'_>,
    ) -> ServerResult<ExecutableDocument> {
        let mut document = next.run(ctx, query, variables).await.map_err(|mut error| {
            error.message = "GraphQL document exceeds the supported selection depth.".into();
            tagged(error, "GRAPHQL_VALIDATION_FAILED")
        })?;
        normalize_typename(&mut document)?;
        if let Some(preflight) = ctx.data_opt::<Preflight>() {
            *preflight.document.lock().unwrap() = Some(document.clone());
        }
        Ok(document)
    }

    async fn validation(
        &self,
        ctx: &ExtensionContext<'_>,
        next: NextValidation<'_>,
    ) -> Result<ValidationResult, Vec<ServerError>> {
        let result = next.run(ctx).await.map_err(|errors| {
            errors
                .into_iter()
                .map(|mut error| {
                    // Library validation messages can echo literal payload contents.
                    error.message = "Document does not conform to the GraphQL schema.".into();
                    tagged(error, "GRAPHQL_VALIDATION_FAILED")
                })
                .collect::<Vec<_>>()
        })?;
        if let Some(preflight) = ctx.data_opt::<Preflight>() {
            let document = preflight.document.lock().unwrap();
            let document = document.as_ref().unwrap();
            let operation = match (&document.operations, &preflight.operation_name) {
                (DocumentOperations::Single(op), None) => Some(op),
                (DocumentOperations::Multiple(ops), Some(name)) => ops.get(name.as_str()),
                (DocumentOperations::Multiple(ops), None) if ops.len() == 1 => ops.values().next(),
                _ => None,
            }.ok_or_else(|| vec![tagged(ServerError::new("Select an existing operation with operationName when the document contains multiple operations.", None), "OPERATION_RESOLUTION_FAILURE")])?;
            let mut errors = Vec::new();
            for definition in &operation.node.variable_definitions {
                let definition_value = &definition.node;
                let value = preflight
                    .variables
                    .get(&definition_value.name.node)
                    .or_else(|| definition_value.default_value.as_ref().map(|v| &v.node));
                if !valid_variable(
                    &ctx.schema_env.registry,
                    &definition_value.var_type.node.to_string(),
                    value,
                ) {
                    errors.push(tagged(
                        ServerError::new(
                            format!(
                                "Variable ${} cannot be coerced to {}.",
                                definition_value.name.node, definition_value.var_type.node
                            ),
                            Some(definition.pos),
                        ),
                        "BAD_USER_INPUT",
                    ));
                }
            }
            if !errors.is_empty() {
                return Err(errors);
            }
        }
        Ok(result)
    }

    async fn execute(
        &self,
        ctx: &ExtensionContext<'_>,
        operation_name: Option<&str>,
        next: NextExecute<'_>,
    ) -> Response {
        if ctx.data_opt::<Preflight>().is_some() {
            Response::new(Value::Null)
        } else {
            next.run(ctx, operation_name).await
        }
    }
}

fn valid_variable(registry: &Registry, ty: &str, value: Option<&Value>) -> bool {
    if let Some(inner) = ty.strip_suffix('!') {
        return value
            .is_some_and(|v| *v != Value::Null && valid_variable(registry, inner, Some(v)));
    }
    let Some(value) = value.filter(|v| **v != Value::Null) else {
        return true;
    };
    if let Some(inner) = ty.strip_prefix('[').and_then(|v| v.strip_suffix(']')) {
        return match value {
            Value::List(values) => values
                .iter()
                .all(|v| valid_variable(registry, inner, Some(v))),
            value => valid_variable(registry, inner, Some(value)),
        };
    }
    match registry.types.get(ty) {
        Some(MetaType::Scalar { is_valid, .. }) => {
            if ty == "Int" {
                return matches!(value, Value::Number(n) if n.as_i64().is_some_and(|n| i32::try_from(n).is_ok()));
            }
            is_valid.as_ref().is_none_or(|check| check(value))
        }
        Some(MetaType::Enum { enum_values, .. }) => match value {
            Value::String(name) => enum_values.contains_key(name.as_str()),
            Value::Enum(name) => enum_values.contains_key(name.as_str()),
            _ => false,
        },
        Some(MetaType::InputObject { input_fields, .. }) => match value {
            Value::Object(values) => {
                values
                    .keys()
                    .all(|key| input_fields.contains_key(key.as_str()))
                    && input_fields.values().all(|field| {
                        let value = values.get(field.name.as_str());
                        (value.is_none() && field.default_value.is_some())
                            || valid_variable(registry, &field.ty, value)
                    })
            }
            _ => false,
        },
        _ => false,
    }
}

// async-graphql 7.2's validation visitor skips __typename fields entirely,
// including their directives. Put those directives on an equivalent inline
// fragment so variable usage and directive arguments are still validated.
fn normalize_typename(document: &mut ExecutableDocument) -> ServerResult<()> {
    use async_graphql::{
        Positioned,
        parser::types::{InlineFragment, Selection, SelectionSet},
    };
    fn visit(set: &mut SelectionSet) -> ServerResult<()> {
        for selection in &mut set.items {
            match &mut selection.node {
                Selection::Field(field) => {
                    if field.node.name.node == "__typename" {
                        if !field.node.arguments.is_empty()
                            || !field.node.selection_set.node.items.is_empty()
                        {
                            return Err(tagged(
                                ServerError::new(
                                    "__typename accepts no arguments or subfields.",
                                    Some(field.pos),
                                ),
                                "GRAPHQL_VALIDATION_FAILED",
                            ));
                        }
                        if !field.node.directives.is_empty() {
                            let directives = std::mem::take(&mut field.node.directives);
                            let child = selection.clone();
                            selection.node = Selection::InlineFragment(Positioned::new(
                                InlineFragment {
                                    type_condition: None,
                                    directives,
                                    selection_set: Positioned::new(
                                        SelectionSet { items: vec![child] },
                                        selection.pos,
                                    ),
                                },
                                selection.pos,
                            ));
                        }
                    } else {
                        visit(&mut field.node.selection_set.node)?;
                    }
                }
                Selection::InlineFragment(fragment) => {
                    visit(&mut fragment.node.selection_set.node)?
                }
                Selection::FragmentSpread(_) => {}
            }
        }
        Ok(())
    }
    match &mut document.operations {
        DocumentOperations::Single(op) => visit(&mut op.node.selection_set.node)?,
        DocumentOperations::Multiple(ops) => {
            for op in ops.values_mut() {
                visit(&mut op.node.selection_set.node)?;
            }
        }
    }
    for fragment in document.fragments.values_mut() {
        visit(&mut fragment.node.selection_set.node)?;
    }
    Ok(())
}

fn request_errors(errors: Vec<ServerError>) -> axum::response::Response {
    (
        StatusCode::BAD_REQUEST,
        [(header::CONTENT_TYPE, "application/graphql-response+json")],
        Json(serde_json::json!({"errors":errors})),
    )
        .into_response()
}
