//! Bearer token check for routes that must not be open to everyone.

use axum::Json;
use axum::extract::Request;
use axum::http::{StatusCode, header};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use serde_json::json;

/// The token callers send as `Authorization: Bearer <token>`. Hardcoded, so
/// anyone who can read this repository can call the routes it guards.
pub const BEARER_TOKEN: &str = "8bcb74f24405708e90b0aa1be5679f422ea2ba199ae8ab0d0f1e80acd8c9c97d";

/// The OpenAPI security scheme name for [`BEARER_TOKEN`].
pub const SECURITY_SCHEME: &str = "bearer";

/// Lets a request through only when it carries [`BEARER_TOKEN`]; answers
/// `401 Unauthorized` otherwise.
pub async fn require_bearer_token(request: Request, next: Next) -> Response {
    let token = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "));

    match token {
        Some(token) if constant_time_eq(token.as_bytes(), BEARER_TOKEN.as_bytes()) => {
            next.run(request).await
        }
        _ => (
            StatusCode::UNAUTHORIZED,
            [(header::WWW_AUTHENTICATE, "Bearer")],
            Json(json!({ "error": "a valid bearer token is required" })),
        )
            .into_response(),
    }
}

/// Compares in time that depends only on the lengths, so response timing
/// doesn't reveal how much of a guessed token is right.
fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    a.len() == b.len() && a.iter().zip(b).fold(0, |diff, (x, y)| diff | (x ^ y)) == 0
}

#[cfg(test)]
mod tests {
    use super::constant_time_eq;

    #[test]
    fn compares_bytes() {
        assert!(constant_time_eq(b"token", b"token"));
        assert!(!constant_time_eq(b"token", b"tokem"));
        assert!(!constant_time_eq(b"token", b"tokens"));
        assert!(!constant_time_eq(b"", b"token"));
    }
}
