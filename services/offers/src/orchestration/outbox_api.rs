//! Outbox batch endpoint; Mulac's Poem adapter validates and executes it, and
//! answers `503` while no publisher is configured. The publisher is
//! `CustomerFacingPublisher`, built from `CUSTOMER_FACING_URL` in `main`.

use mulac_poem::io::{OutboxAdapter, RunRequest, RunResponse};
use poem::Result;
use poem_openapi::{OpenApi, payload::Json};

pub struct OutboxApi {
    adapter: OutboxAdapter,
}

impl OutboxApi {
    pub fn new(adapter: OutboxAdapter) -> Self {
        Self { adapter }
    }
}

#[OpenApi]
impl OutboxApi {
    /// Publish one batch to customer-facing's inbox; returns 503 when delivery is not configured.
    #[oai(path = "/outbox/run", method = "post")]
    async fn run_outbox(&self, request: Json<RunRequest>) -> Result<Json<RunResponse>> {
        self.adapter.run(request).await
    }
}

#[cfg(test)]
mod tests {
    use mulac_poem::io::OutboxAdapter;
    use outbox::io::{OutboundMessageEnvelope, OutboxError, OutboxPublisherPort};
    use outbox_api::io::{OutboxService, testing};
    use poem::{http::StatusCode, test::TestClient};
    use serde_json::{Value, json};
    use std::sync::Arc;

    struct UnusedPublisher;
    impl OutboxPublisherPort for UnusedPublisher {
        fn publish(&self, _: OutboundMessageEnvelope) -> Result<(), OutboxError> {
            panic!("storage failure must prevent publication")
        }
    }

    fn client_with(outbox: OutboxAdapter) -> TestClient<poem::Route> {
        let (_, mut state) = crate::start_mulac(crate::test_pool()).unwrap();
        state.outbox = outbox;
        TestClient::new(crate::app_with_reporting_token(state, "test".into()))
    }

    #[tokio::test]
    async fn configured_outbox_reports_storage_failure() {
        let client = client_with(OutboxAdapter::new(OutboxService::from_postgres(
            crate::test_pool(),
            Arc::new(UnusedPublisher),
        )));
        for limit in [1, 1000] {
            let response = client
                .post("/api/outbox/run")
                .body_json(&json!({"limit":limit}))
                .send()
                .await;
            response.assert_status_is_ok();
            let body: Value = response.json().await.value().deserialize();
            assert!(body["processed"].is_null());
            let errors = body["errors"].as_array().unwrap();
            assert_eq!(errors.len(), 1);
            assert!(
                errors[0].as_str().unwrap().starts_with("storage error: "),
                "{errors:?}"
            );
        }
    }

    #[tokio::test]
    async fn outbox_run_validates_the_shared_run_request_table() {
        let client = TestClient::new(crate::app(crate::test_pool()));
        for body in testing::valid_run_requests() {
            // Validation passed when the unconfigured publisher is reached.
            let response = client.post("/api/outbox/run").body_json(&body).send().await;
            response.assert_status(StatusCode::SERVICE_UNAVAILABLE);
            response
                .assert_text("outbox publisher is not configured")
                .await;
        }
        for body in testing::invalid_run_requests() {
            let response = client.post("/api/outbox/run").body_json(&body).send().await;
            assert_eq!(response.0.status(), StatusCode::BAD_REQUEST, "{body}");
        }
        // Poem's RunRequest wrapper validates while parsing, so it prefixes the message.
        let response = client
            .post("/api/outbox/run")
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
    async fn outbox_run_requires_json_content_type() {
        let client = TestClient::new(crate::app(crate::test_pool()));
        for content_type in [None, Some("text/plain")] {
            let mut request = client.post("/api/outbox/run").body(r#"{"limit":1}"#);
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
    async fn outbox_run_publishes_the_shared_run_schemas() {
        let client = TestClient::new(crate::app(crate::test_pool()));
        let spec: Value = client
            .get("/api/openapi.json")
            .send()
            .await
            .json()
            .await
            .value()
            .deserialize();
        let run = &spec["paths"]["/outbox/run"]["post"];
        assert_eq!(
            run["summary"],
            "Publish one batch to customer-facing's inbox; returns 503 when delivery is not configured."
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
    }

    #[tokio::test]
    async fn outbox_run_keeps_published_items_of_a_partial_batch() {
        let (service, outcomes) = testing::partially_failing_service();
        let response = client_with(OutboxAdapter::new(service))
            .post("/api/outbox/run")
            .body_json(&json!({"limit":5}))
            .send()
            .await;
        response.assert_status_is_ok();
        response
            .assert_json(json!({"processed":null,"errors":["transport error: broker unavailable"]}))
            .await;
        assert_eq!(
            *outcomes.lock().unwrap(),
            vec!["completed", "failed", "completed"]
        );
    }

    #[tokio::test]
    async fn outbox_run_sanitizes_blocking_task_failures() {
        let response = client_with(OutboxAdapter::new(testing::panicking_service()))
            .post("/api/outbox/run")
            .body_json(&json!({"limit":1}))
            .send()
            .await;
        response.assert_status(StatusCode::INTERNAL_SERVER_ERROR);
        response.assert_text("queue processing task failed").await;
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn outbox_run_reports_customer_facing_publisher_errors_verbatim() {
        // Mulac's test rows are not routed to customer_facing, so each fails
        // before any request is sent.
        let publisher =
            crate::orchestration::customer_facing_publisher::CustomerFacingPublisher::new(
                "http://127.0.0.1:9",
                tokio::runtime::Handle::current(),
            )
            .unwrap();
        let service = testing::publisher_service(2, Arc::new(publisher));
        let response = client_with(OutboxAdapter::new(service))
            .post("/api/outbox/run")
            .body_json(&json!({"limit":5}))
            .send()
            .await;
        response.assert_status_is_ok();
        response
            .assert_json(json!({"processed":null,"errors":[
                "routing error: outbox message destination must be customer_facing",
                "routing error: outbox message destination must be customer_facing",
            ]}))
            .await;
    }
}
