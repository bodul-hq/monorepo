use poem_openapi::{OpenApi, payload::PlainText};

pub struct HealthApi;

#[OpenApi]
impl HealthApi {
    #[oai(path = "/health", method = "get")]
    async fn health(&self) -> PlainText<&'static str> {
        PlainText(shared::health())
    }
}
