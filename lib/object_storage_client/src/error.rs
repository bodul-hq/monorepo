use serde::Deserialize;
use serde_json::{Map, Value};

/// A transport-independent classification; GraphQL uses its first error code.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorKind {
    Configuration,
    Transport,
    BadUserInput,
    Unauthorized,
    NotFound,
    Conflict,
    ServiceUnavailable,
    InternalServerError,
    GraphqlParseFailed,
    GraphqlValidationFailed,
    OperationResolutionFailure,
    UnexpectedResponse,
}

/// RFC 9457 details returned by REST and by GraphQL's HTTP/authentication layer.
#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct ProblemDetails {
    #[serde(rename = "type")]
    pub problem_type: String,
    pub title: String,
    pub status: u16,
    pub detail: String,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct ErrorLocation {
    pub line: usize,
    pub column: usize,
}

/// Preserves GraphQL locations, response paths, and extension fields.
#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct GraphqlError {
    pub message: String,
    #[serde(default)]
    pub locations: Vec<ErrorLocation>,
    #[serde(default)]
    pub path: Vec<Value>,
    #[serde(default)]
    pub extensions: Map<String, Value>,
}
impl GraphqlError {
    pub fn code(&self) -> Option<&str> {
        self.extensions.get("code").and_then(Value::as_str)
    }
}

/// Display messages omit credentials and response contents. Structured server
/// details remain available to callers; avoid logging artifact-bearing data.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("invalid object storage configuration: {0}")]
    Configuration(&'static str),
    #[error("object storage HTTP request failed")]
    Transport(#[source] reqwest::Error),
    #[error("object storage returned HTTP {status}")]
    Http {
        status: u16,
        problem: Option<ProblemDetails>,
    },
    #[error("object storage returned GraphQL errors (HTTP {status})")]
    Graphql {
        status: u16,
        errors: Vec<GraphqlError>,
        data: Option<Value>,
    },
    #[error("unexpected object storage response (HTTP {status}): {detail}")]
    Protocol { status: u16, detail: &'static str },
}

impl From<reqwest::Error> for Error {
    fn from(error: reqwest::Error) -> Self {
        Self::Transport(error.without_url())
    }
}

impl Error {
    pub fn kind(&self) -> ErrorKind {
        use ErrorKind::*;
        match self {
            Self::Configuration(_) => Configuration,
            Self::Transport(_) => Transport,
            Self::Protocol { .. } => UnexpectedResponse,
            Self::Http { status, .. } => match status {
                400 | 415 => BadUserInput,
                401 => Unauthorized,
                404 => NotFound,
                409 => Conflict,
                503 => ServiceUnavailable,
                500 => InternalServerError,
                _ => UnexpectedResponse,
            },
            Self::Graphql { errors, .. } => match errors.first().and_then(GraphqlError::code) {
                Some("BAD_USER_INPUT") => BadUserInput,
                Some("NOT_FOUND") => NotFound,
                Some("CONFLICT") => Conflict,
                Some("SERVICE_UNAVAILABLE") => ServiceUnavailable,
                Some("INTERNAL_SERVER_ERROR") => InternalServerError,
                Some("GRAPHQL_PARSE_FAILED") => GraphqlParseFailed,
                Some("GRAPHQL_VALIDATION_FAILED") => GraphqlValidationFailed,
                Some("OPERATION_RESOLUTION_FAILURE") => OperationResolutionFailure,
                _ => UnexpectedResponse,
            },
        }
    }
}
