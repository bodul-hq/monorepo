mod common;
use axum::{
    Router,
    http::{StatusCode, header},
};
use common::*;
use diesel::{
    PgConnection,
    r2d2::{ConnectionManager, Pool},
};
use object_storage::{BEARER_TOKEN, app, store::Store};
use serde_json::{Value, json};

fn offline() -> Router {
    let pool = Pool::builder()
        .max_size(1)
        .min_idle(Some(0))
        .connection_timeout(std::time::Duration::from_millis(30))
        .build_unchecked(ConnectionManager::<PgConnection>::new(
            "postgres://127.0.0.1:1/unavailable?connect_timeout=1",
        ));
    app(Store(pool))
}

#[tokio::test]
async fn all_routes_authenticate_before_processing() {
    let app = offline();
    for path in [
        "/api/health",
        "/api/reporting/database-tables",
        "/api/openapi.json",
        "/api/docs",
        "/api/docs/",
        "/api/docs/swagger-ui.css",
        "/graphql",
        "/graphql/ide",
        "/api/buckets/sitemap_raw/records",
        "/api/buckets/sitemap_raw/records/invalid",
        "/unknown",
    ] {
        for method in ["GET", "POST"] {
            for token in [None, Some("wrong")] {
                let (status, headers, body) =
                    raw(&app, method, path, token, None, "not JSON").await;
                assert_eq!(status, StatusCode::UNAUTHORIZED, "{method} {path}");
                assert_eq!(headers[header::WWW_AUTHENTICATE], "Bearer");
                problem(status, &headers, &serde_json::from_slice(&body).unwrap());
            }
        }
    }
}

#[tokio::test]
async fn health_docs_and_post_only_graphql() {
    let app = offline();
    for (path, expected) in [
        ("/api/health", StatusCode::OK),
        ("/api/docs", StatusCode::SEE_OTHER),
        ("/api/docs/", StatusCode::OK),
        ("/api/docs/swagger-ui.css", StatusCode::OK),
        ("/graphql", StatusCode::METHOD_NOT_ALLOWED),
        (
            "/graphql/ide",
            if cfg!(debug_assertions) {
                StatusCode::OK
            } else {
                StatusCode::NOT_FOUND
            },
        ),
    ] {
        let (status, _, body) = raw(&app, "GET", path, Some(BEARER_TOKEN), None, "").await;
        assert_eq!(status, expected, "{path}");
        if path == "/api/health" {
            assert_eq!(body, b"ok");
        }
    }
    let (status, headers, response) = request(&app, "GET", "/graphql", Value::Null).await;
    assert_eq!(status, StatusCode::METHOD_NOT_ALLOWED);
    problem(status, &headers, &response);
    let (status, _, spec) = request(&app, "GET", "/api/openapi.json", Value::Null).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(spec["security"], json!([{"bearer":[]}]));
    assert_eq!(
        spec["components"]["securitySchemes"]["bearer"]["scheme"],
        "bearer"
    );
    assert!(spec["paths"]["/api/buckets/{bucket}/records"]["post"]["responses"]["409"].is_object());
    assert!(
        spec["components"]["schemas"]["CreateRecord"]["properties"]["metadata"]["not"].is_object()
    );
}

#[tokio::test]
async fn transport_errors_use_problem_details() {
    let app = offline();
    for path in ["/graphql", "/api/buckets/sitemap_raw/records"] {
        for content_type in [None, Some("text/plain"), Some("application/other+json")] {
            let (status, headers, body) =
                raw(&app, "POST", path, Some(BEARER_TOKEN), content_type, "{").await;
            assert_eq!(status, StatusCode::UNSUPPORTED_MEDIA_TYPE);
            problem(status, &headers, &serde_json::from_slice(&body).unwrap());
        }
        for body in ["{", "[]", "null", "{}"] {
            let (status, headers, body) = raw(
                &app,
                "POST",
                path,
                Some(BEARER_TOKEN),
                Some("application/json"),
                body,
            )
            .await;
            assert_eq!(status, StatusCode::BAD_REQUEST);
            problem(status, &headers, &serde_json::from_slice(&body).unwrap());
        }
    }
    for body in [
        json!({"query":1}),
        json!({"query":"{__typename}","variables":[]}),
        json!({"query":"{__typename}","operationName":4}),
    ] {
        let (status, headers, body) = request(&app, "POST", "/graphql", body).await;
        assert_eq!(status, StatusCode::BAD_REQUEST);
        problem(status, &headers, &body);
    }
}

#[tokio::test]
async fn rest_validates_input_before_storage() {
    let app = offline();
    let original = input();
    let mut invalid = vec![
        json!({}),
        json!({"idempotency_key":original["idempotency_key"],"payload":"x"}),
    ];
    for (field, value) in [
        ("id", json!(uuid::Uuid::now_v7())),
        ("data_id", json!("no")),
        ("created_at", json!("now")),
        ("payload", json!(1)),
        ("payload", json!("x\u{0}y")),
        ("idempotency_key", json!(uuid::Uuid::new_v4())),
        ("metadata", Value::Null),
        ("metadata", json!({"\u{0}":1})),
        ("metadata", json!(["\u{0}"])),
    ] {
        let mut body = original.clone();
        body[field] = value;
        invalid.push(body);
    }
    for body in invalid {
        let (status, headers, body) =
            request(&app, "POST", "/api/buckets/sitemap_raw/records", body).await;
        assert_eq!(status, StatusCode::BAD_REQUEST);
        problem(status, &headers, &body);
    }
    for path in [
        "/api/buckets/SITEMAP_RAW/records",
        "/api/buckets/nope/records",
        "/api/buckets/sitemap_raw/records/invalid",
    ] {
        let method = if path.ends_with("invalid") {
            "GET"
        } else {
            "POST"
        };
        assert_eq!(
            request(&app, method, path, original.clone()).await.0,
            StatusCode::BAD_REQUEST
        );
    }
}

#[tokio::test]
async fn graphql_request_errors_are_classified_without_execution() {
    let app = offline();
    let cases = [
        (json!({"query":"{"}), "GRAPHQL_PARSE_FAILED"),
        (json!({"query":"{unknown}"}), "GRAPHQL_VALIDATION_FAILED"),
        (
            json!({"query":"{record {id}}"}),
            "GRAPHQL_VALIDATION_FAILED",
        ),
        (
            json!({"query":"{record(bucket: 1, id: \"x\") {id}}"}),
            "GRAPHQL_VALIDATION_FAILED",
        ),
        (
            json!({"query":"query A {__typename} query B {__typename}"}),
            "OPERATION_RESOLUTION_FAILURE",
        ),
        (
            json!({"query":"query A {__typename}", "operationName":"B"}),
            "OPERATION_RESOLUTION_FAILURE",
        ),
        (
            json!({"query":"{__typename}", "operationName":"B"}),
            "OPERATION_RESOLUTION_FAILURE",
        ),
        (json!({"query":CREATE_QUERY}), "BAD_USER_INPUT"),
        (
            json!({"query":CREATE_QUERY,"variables":{"input":null}}),
            "BAD_USER_INPUT",
        ),
        (
            json!({"query":CREATE_QUERY,"variables":{"input":{"bucket":"sitemap_raw"}}}),
            "BAD_USER_INPUT",
        ),
        (
            json!({"query":"query($b: Boolean!) { __typename @include(if: $b) }", "variables":{"b":"yes"}}),
            "BAD_USER_INPUT",
        ),
    ];
    for (body, code) in cases {
        let (status, headers, response) = request(&app, "POST", "/graphql", body.clone()).await;
        assert_eq!(status, StatusCode::BAD_REQUEST, "{body}: {response}");
        assert_eq!(
            headers[header::CONTENT_TYPE],
            "application/graphql-response+json"
        );
        assert!(response.get("data").is_none(), "{response}");
        assert_eq!(
            response["errors"][0]["extensions"]["code"], code,
            "{body}: {response}"
        );
        assert!(response["errors"][0].get("path").is_none());
        if code != "OPERATION_RESOLUTION_FAILURE" {
            assert!(response["errors"][0]["locations"].is_array());
        }
    }
    let mut body = gql_input("sitemap_raw", &input());
    body["variables"]["input"]["unexpected"] = true.into();
    assert_eq!(
        request(&app, "POST", "/graphql", body).await.2["errors"][0]["extensions"]["code"],
        "BAD_USER_INPUT"
    );
}

#[tokio::test]
async fn execution_errors_preserve_data_paths_and_locations() {
    let app = offline();
    let body = json!({"query":"{ name: __typename bad: record(bucket: \"invalid\", id: \"invalid\") { id } }"});
    let (status, _, response) = request(&app, "POST", "/graphql", body).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(response["data"], json!({"name":"Query","bad":null}));
    assert_eq!(
        response["errors"][0]["extensions"]["code"],
        "BAD_USER_INPUT"
    );
    assert_eq!(response["errors"][0]["path"], json!(["bad"]));
    assert!(response["errors"][0]["locations"].is_array());
    let (status, headers, body) =
        request(&app, "POST", "/api/buckets/sitemap_raw/records", input()).await;
    assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
    problem(status, &headers, &body);
    let (status, _, response) =
        request(&app, "POST", "/graphql", gql_input("sitemap_raw", &input())).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        response["errors"][0]["extensions"]["code"],
        "SERVICE_UNAVAILABLE"
    );
}

#[tokio::test]
async fn graphql_defaults_and_definition_errors() {
    let app = offline();
    for query in [
        "query($include:Boolean! = true) { __typename @include(if:$include) }",
        "query($skip:Boolean! = false) { __typename @skip(if:$skip) }",
    ] {
        let (status, _, response) = request(&app, "POST", "/graphql", json!({"query":query})).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(response, json!({"data":{"__typename":"Query"}}));
    }
    for (query, code) in [
        (
            "query A {__typename} query A {__typename}",
            "GRAPHQL_VALIDATION_FAILED",
        ),
        (
            "{__typename} query A {__typename}",
            "GRAPHQL_VALIDATION_FAILED",
        ),
        ("{__typename(arg:1)}", "GRAPHQL_VALIDATION_FAILED"),
        ("{__typename { id }}", "GRAPHQL_VALIDATION_FAILED"),
        (
            "fragment Unused on Query {__typename}",
            "OPERATION_RESOLUTION_FAILURE",
        ),
    ] {
        let (status, _, response) = request(&app, "POST", "/graphql", json!({"query":query})).await;
        assert_eq!(status, StatusCode::BAD_REQUEST, "{response}");
        assert_eq!(response["errors"][0]["extensions"]["code"], code);
        assert!(response.get("data").is_none());
    }
}

#[tokio::test]
async fn reporting_unavailable_and_documented() {
    let app = offline();
    let (status, headers, body) =
        request(&app, "GET", "/api/reporting/database-tables", Value::Null).await;
    assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
    problem(status, &headers, &body);
    let (_, _, spec) = request(&app, "GET", "/api/openapi.json", Value::Null).await;
    assert!(spec["paths"]["/api/reporting/database-tables"]["get"]["responses"]["200"].is_object());
    assert!(spec["components"]["schemas"]["DatabaseTables"].is_object());
}
