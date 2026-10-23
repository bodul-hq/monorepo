//! Inbox endpoints; Mulac's Poem adapter parses, validates and executes them.

use mulac_poem::io::{InboxAdapter, InboxSubmission, RunRequest, RunResponse, SubmitInboxRequest};
use poem::Result;
use poem_openapi::{OpenApi, payload::Json};

pub struct InboxApi {
    adapter: InboxAdapter,
}

impl InboxApi {
    pub fn new(adapter: InboxAdapter) -> Self {
        Self { adapter }
    }
}

#[OpenApi]
impl InboxApi {
    /// Store one message without processing it. Unknown routing keys are accepted.
    #[oai(path = "/inbox", method = "post")]
    async fn submit_inbox(&self, request: Json<SubmitInboxRequest>) -> Result<InboxSubmission> {
        self.adapter.submit(request).await
    }

    /// Queue commands from stored inbox messages, without executing them.
    #[oai(path = "/inbox/run", method = "post")]
    async fn run_inbox(&self, request: Json<RunRequest>) -> Result<Json<RunResponse>> {
        self.adapter.run(request).await
    }
}

#[cfg(test)]
mod tests {
    use inbox_api::io::testing;
    use mulac_poem::io::InboxAdapter;
    use poem::{http::StatusCode, test::TestClient};
    use serde_json::{Value, json};

    #[tokio::test]
    async fn inbox_rejects_malformed_envelopes() {
        let client = TestClient::new(crate::app(crate::test_pool()));
        for body in testing::invalid_submissions() {
            let response = client.post("/api/inbox").body_json(&body).send().await;
            assert_eq!(response.0.status(), StatusCode::BAD_REQUEST, "{body}");
        }
        for (body, message) in testing::submission_error_messages() {
            let response = client.post("/api/inbox").body_json(&body).send().await;
            response.assert_status(StatusCode::BAD_REQUEST);
            response.assert_text(message).await;
        }
        client
            .post("/api/inbox")
            .content_type("application/json")
            .body("{")
            .send()
            .await
            .assert_status(StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn inbox_accepts_arbitrary_json_and_sanitizes_storage_errors() {
        let client = TestClient::new(crate::app(crate::test_pool()));
        for body in testing::valid_submissions() {
            let response = client.post("/api/inbox").body_json(&body).send().await;
            response.assert_status(StatusCode::INTERNAL_SERVER_ERROR);
            response
                .assert_text("inbox message could not be stored")
                .await;
        }
    }

    #[tokio::test]
    async fn inbox_accepts_unknown_fields_duplicate_keys_and_uuid_formats() {
        let client = TestClient::new(crate::app(crate::test_pool()));
        for body in [
            r#"{"payload":1,"routing_key":"x","source":"crawler","extra":1}"#,
            r#"{"payload":1,"payload":2,"routing_key":"x","source":"crawler"}"#,
            r#"{"payload":1,"routing_key":" x ","source":" crawler "}"#,
            r#"{"payload":1,"routing_key":"x","source":"crawler","message_id":null,"correlation_id":null}"#,
            r#"{"payload":1,"routing_key":"x","source":"crawler","message_id":"0191d6c4-8f5e-7a3b-9c1d-2e3f4a5b6c7d"}"#,
            r#"{"payload":1,"routing_key":"x","source":"crawler","message_id":"0191D6C4-8F5E-7A3B-9C1D-2E3F4A5B6C7D"}"#,
            r#"{"payload":1,"routing_key":"x","source":"crawler","correlation_id":"0191d6c48f5e7a3b9c1d2e3f4a5b6c7d"}"#,
        ] {
            // Validation passed when the unavailable test database is reached.
            let response = client
                .post("/api/inbox")
                .content_type("application/json")
                .body(body)
                .send()
                .await;
            response.assert_status(StatusCode::INTERNAL_SERVER_ERROR);
            response
                .assert_text("inbox message could not be stored")
                .await;
        }
        let response = client
            .post("/api/inbox")
            .body_json(&json!({"payload":1,"routing_key":"x","source":"crawler","message_id":""}))
            .send()
            .await;
        response.assert_status(StatusCode::BAD_REQUEST);
        response.assert_text("message_id must be a UUID").await;
    }

    #[tokio::test]
    async fn inbox_routes_require_json_content_type() {
        let client = TestClient::new(crate::app(crate::test_pool()));
        for (path, body) in [
            (
                "/api/inbox",
                r#"{"payload":1,"routing_key":"x","source":"crawler"}"#,
            ),
            ("/api/inbox/run", r#"{"limit":1}"#),
        ] {
            client
                .post(path)
                .body(body)
                .send()
                .await
                .assert_status(StatusCode::UNSUPPORTED_MEDIA_TYPE);
            client
                .post(path)
                .content_type("text/plain")
                .body(body)
                .send()
                .await
                .assert_status(StatusCode::UNSUPPORTED_MEDIA_TYPE);
        }
    }

    #[tokio::test]
    async fn inbox_run_accepts_limit_bounds_and_unknown_fields() {
        let client = TestClient::new(crate::app(crate::test_pool()));
        for body in [
            json!({"limit":1}),
            json!({"limit":1000}),
            json!({"limit":1,"extra":true}),
        ] {
            let response = client.post("/api/inbox/run").body_json(&body).send().await;
            response.assert_status_is_ok();
            let body: Value = response.json().await.value().deserialize();
            assert!(body["processed"].is_null());
            assert_eq!(body["errors"].as_array().unwrap().len(), 1);
        }
        for body in [json!({"limit":null}), json!({"limit":1.0})] {
            client
                .post("/api/inbox/run")
                .body_json(&body)
                .send()
                .await
                .assert_status(StatusCode::BAD_REQUEST);
        }
    }

    #[tokio::test]
    async fn inbox_sanitizes_blocking_task_failures() {
        let (kernel, mut state) = crate::start_mulac(crate::test_pool()).unwrap();
        state.inbox = InboxAdapter::new(testing::panicking_service());
        let client = TestClient::new(crate::app_with_reporting_token(state, "test".into()));
        let response = client
            .post("/api/inbox")
            .body_json(&json!({"payload":1,"routing_key":"unknown","source":"crawler"}))
            .send()
            .await;
        response.assert_status(StatusCode::INTERNAL_SERVER_ERROR);
        response.assert_text("inbox submission task failed").await;
        kernel.shutdown();
    }
}
