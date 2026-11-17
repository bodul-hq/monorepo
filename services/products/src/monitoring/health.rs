use poem_openapi::{OpenApi, payload::PlainText};
use shared::health as shared_health;

pub struct HealthApi;

#[OpenApi]
impl HealthApi {
    #[oai(path = "/health", method = "get")]
    async fn health(&self) -> PlainText<&'static str> {
        PlainText(shared_health())
    }
}
