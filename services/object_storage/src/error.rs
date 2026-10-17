use async_graphql::ErrorExtensions;
use axum::{
    Json,
    http::{StatusCode, header},
    response::{IntoResponse, Response},
};
use serde_json::json;

#[derive(Debug, Clone, Copy)]
pub enum Error {
    Input(&'static str),
    NotFound,
    Conflict,
    Unavailable,
    Internal,
    Unauthorized,
    ContentType,
    MethodNotAllowed,
}

impl Error {
    pub fn status(self) -> StatusCode {
        match self {
            Self::Input(_) => StatusCode::BAD_REQUEST,
            Self::NotFound => StatusCode::NOT_FOUND,
            Self::Conflict => StatusCode::CONFLICT,
            Self::Unavailable => StatusCode::SERVICE_UNAVAILABLE,
            Self::Internal => StatusCode::INTERNAL_SERVER_ERROR,
            Self::Unauthorized => StatusCode::UNAUTHORIZED,
            Self::ContentType => StatusCode::UNSUPPORTED_MEDIA_TYPE,
            Self::MethodNotAllowed => StatusCode::METHOD_NOT_ALLOWED,
        }
    }
    pub fn detail(self) -> &'static str {
        match self {
            Self::Input(message) => message,
            Self::NotFound => "No record matches this bucket and ID.",
            Self::Conflict => {
                "This idempotency key already identifies different content in this bucket."
            }
            Self::Unavailable => "Storage is temporarily unavailable.",
            Self::Internal => "An unexpected server failure occurred.",
            Self::Unauthorized => "A valid bearer token is required.",
            Self::ContentType => "Content-Type must be application/json.",
            Self::MethodNotAllowed => "This method is not supported for this route.",
        }
    }
    pub fn graphql(self) -> async_graphql::Error {
        let code = match self {
            Self::Input(_) => "BAD_USER_INPUT",
            Self::NotFound => "NOT_FOUND",
            Self::Conflict => "CONFLICT",
            Self::Unavailable => "SERVICE_UNAVAILABLE",
            _ => "INTERNAL_SERVER_ERROR",
        };
        async_graphql::Error::new(self.detail())
            .extend_with(|_, extensions| extensions.set("code", code))
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        let status = self.status();
        let mut response = (
            status,
            [(header::CONTENT_TYPE, "application/problem+json")],
            Json(json!({
                "type": "about:blank", "title": status.canonical_reason().unwrap(),
                "status": status.as_u16(), "detail": self.detail()
            })),
        )
            .into_response();
        if matches!(self, Self::Unauthorized) {
            response
                .headers_mut()
                .insert(header::WWW_AUTHENTICATE, "Bearer".parse().unwrap());
        }
        response
    }
}
