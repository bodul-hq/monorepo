//! Event batch endpoint; Mulac's Poem adapter validates and executes it.
//! Subscribers stay registered with the kernel in `start_mulac`.

use mulac_poem::io::{EventingAdapter, RunRequest, RunResponse};
use poem::Result;
use poem_openapi::{OpenApi, payload::Json};

pub struct EventingApi {
    adapter: EventingAdapter,
}

impl EventingApi {
    pub fn new(adapter: EventingAdapter) -> Self {
        Self { adapter }
    }
}

#[OpenApi]
impl EventingApi {
    /// Dispatch one batch of due events to the registered projectors.
    #[oai(path = "/events/run", method = "post")]
    async fn run_events(&self, request: Json<RunRequest>) -> Result<Json<RunResponse>> {
        self.adapter.run(request).await
    }
}

#[cfg(test)]
mod tests {
    use eventing_api::io::testing;
    use mulac_poem::io::EventingAdapter;
    use poem::{http::StatusCode, test::TestClient};
    use serde_json::{Value, json};

    #[tokio::test]
    async fn events_run_validates_the_shared_run_request_table() {
        let client = TestClient::new(crate::app(crate::test_pool()));
        for body in testing::valid_run_requests() {
            // Validation passed when the unavailable test database is reached.
            let response = client.post("/api/events/run").body_json(&body).send().await;
            response.assert_status_is_ok();
            let body: Value = response.json().await.value().deserialize();
            assert!(body["processed"].is_null());
            let errors = body["errors"].as_array().unwrap();
            assert_eq!(errors.len(), 1);
            assert!(
                errors[0]
                    .as_str()
                    .unwrap()
                    .starts_with("reservation error: ")
            );
        }
        for body in testing::invalid_run_requests() {
            let response = client.post("/api/events/run").body_json(&body).send().await;
            assert_eq!(response.0.status(), StatusCode::BAD_REQUEST, "{body}");
        }
        // Poem's RunRequest wrapper validates while parsing, so it prefixes the message.
        let response = client
            .post("/api/events/run")
            .body_json(&json!({"limit":0}))
            .send()
            .await;
        response.assert_status(StatusCode::BAD_REQUEST);
        response
            .assert_text(
                "parse request payload error: failed to parse \"RunRequest\": limit must be between 1 and 1000",
            )
            .await;
    }

    #[tokio::test]
    async fn events_run_requires_json_content_type() {
        let client = TestClient::new(crate::app(crate::test_pool()));
        for content_type in [None, Some("text/plain")] {
            let mut request = client.post("/api/events/run").body(r#"{"limit":1}"#);
            if let Some(content_type) = content_type {
                request = request.content_type(content_type);
            }
            request
                .send()
                .await
                .assert_status(StatusCode::UNSUPPORTED_MEDIA_TYPE);
        }
    }

    #[tokio::test]
    async fn events_run_publishes_the_shared_run_schemas() {
        let client = TestClient::new(crate::app(crate::test_pool()));
        let spec: Value = client
            .get("/api/openapi.json")
            .send()
            .await
            .json()
            .await
            .value()
            .deserialize();
        let run = &spec["paths"]["/events/run"]["post"];
        assert_eq!(
            run["summary"],
            "Dispatch one batch of due events to the registered projectors."
        );
        assert_eq!(
            run["requestBody"]["content"]["application/json; charset=utf-8"]["schema"]["$ref"],
            "#/components/schemas/RunRequest"
        );
        assert_eq!(
            run["responses"]["200"]["content"]["application/json; charset=utf-8"]["schema"]["$ref"],
            "#/components/schemas/RunResponse"
        );
        assert_eq!(
            run["responses"]
                .as_object()
                .unwrap()
                .keys()
                .collect::<Vec<_>>(),
            vec!["200"]
        );
        let run_schemas: Vec<_> = spec["components"]["schemas"]
            .as_object()
            .unwrap()
            .keys()
            .filter(|name| name.starts_with("Run"))
            .collect();
        assert_eq!(run_schemas, vec!["RunRequest", "RunResponse"]);
    }

    #[tokio::test]
    async fn events_run_sanitizes_blocking_task_failures() {
        let (kernel, mut state) = crate::start_mulac(crate::test_pool()).unwrap();
        state.events = EventingAdapter::new(testing::panicking_service());
        let client = TestClient::new(crate::app_with_reporting_token(state, "test".into()));
        let response = client
            .post("/api/events/run")
            .body_json(&json!({"limit":1}))
            .send()
            .await;
        response.assert_status(StatusCode::INTERNAL_SERVER_ERROR);
        response.assert_text("queue processing task failed").await;
        kernel.shutdown();
    }
}
