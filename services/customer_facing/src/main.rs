use async_graphql::{EmptyMutation, EmptySubscription, Object, Schema};
use async_graphql_poem::GraphQL;
use poem::{Route, Server, get, handler, listener::TcpListener, web::Html};
use poem_openapi::{OpenApi, OpenApiService, payload::PlainText};

struct Api;

#[OpenApi]
impl Api {
    #[oai(path = "/health", method = "get")]
    async fn health(&self) -> PlainText<&'static str> {
        PlainText(shared::health())
    }
}

struct Query;

#[Object]
impl Query {
    async fn health(&self) -> &'static str {
        shared::health()
    }
}

#[handler]
async fn graphiql() -> Html<String> {
    Html(
        async_graphql::http::GraphiQLSource::build()
            .endpoint("/graphql")
            .finish(),
    )
}

fn app() -> Route {
    let api = OpenApiService::new(Api, "Customer Facing", env!("CARGO_PKG_VERSION")).server("/api");
    let schema = Schema::new(Query, EmptyMutation, EmptySubscription);

    Route::new()
        .nest("/api/docs", api.swagger_ui())
        .at("/api/openapi.json", api.spec_endpoint())
        .nest("/api", api)
        .at("/graphql", get(graphiql).post(GraphQL::new(schema)))
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let port = match std::env::var("PORT") {
        Ok(value) => value.parse::<u16>().map_err(|_| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "PORT must be an integer between 0 and 65535",
            )
        })?,
        Err(std::env::VarError::NotPresent) => 3000,
        Err(error) => return Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, error)),
    };
    Server::new(TcpListener::bind(format!("0.0.0.0:{port}")))
        .run(app())
        .await
}

#[cfg(test)]
mod tests {
    use poem::{http::StatusCode, test::TestClient};
    use serde_json::{Value, json};

    use super::app;

    #[tokio::test]
    async fn health_returns_ok() {
        let response = TestClient::new(app()).get("/api/health").send().await;
        response.assert_status_is_ok();
        response.assert_text("ok").await;
    }

    #[tokio::test]
    async fn graphql_health_returns_ok() {
        let response = TestClient::new(app())
            .post("/graphql")
            .body_json(&json!({ "query": "{ health }" }))
            .send()
            .await;
        response.assert_status_is_ok();
        response
            .assert_json(json!({ "data": { "health": "ok" } }))
            .await;
    }

    #[tokio::test]
    async fn openapi_spec_matches_routes() {
        let client = TestClient::new(app());
        let response = client.get("/api/openapi.json").send().await;
        response.assert_status_is_ok();
        let spec: Value = response.json().await.value().deserialize();

        assert!(
            spec["paths"]["/health"]["get"].is_object(),
            "OpenAPI must document GET /health",
        );
        assert_eq!(spec["servers"][0]["url"], "/api");

        let server = spec["servers"][0]["url"].as_str().unwrap_or("");
        for path in spec["paths"].as_object().unwrap().keys() {
            let response = client.get(format!("{server}{path}")).send().await;
            assert_ne!(response.0.status(), StatusCode::NOT_FOUND, "{server}{path}");
        }
    }

    #[tokio::test]
    async fn swagger_ui_is_served() {
        let response = TestClient::new(app()).get("/api/docs/").send().await;
        response.assert_status_is_ok();
    }
}
