//! OpenAPI uses JSON Schema directly so unrestricted JSON metadata has no
//! artificial object shape or numeric bounds imposed by a derive macro.
use serde_json::{Value, json};

pub fn document() -> Value {
    let mut document = json!({
        "openapi": "3.1.0",
        "info": {"title":"Object Storage", "version":env!("CARGO_PKG_VERSION")},
        "security": [{"bearer":[]}],
        "components": {
            "securitySchemes": {"bearer":{"type":"http","scheme":"bearer"}},
            "schemas": {
                "CreateRecord": {"type":"object", "additionalProperties":false,
                    "required":["idempotency_key","payload","metadata"],
                    "properties":{"idempotency_key":{"type":"string","format":"uuid","description":"UUIDv7"},"payload":{"type":"string"},"metadata":{"not":{"type":"null"}}}},
                "Created": {"type":"object","additionalProperties":false,"required":["id"],"properties":{"id":{"type":"string","format":"uuid"}}},
                "Record": {"type":"object","additionalProperties":false,"required":["id","bucket","idempotency_key","payload","metadata","created_at"],"properties":{
                    "id":{"type":"string","format":"uuid"},"bucket":{"type":"string","enum":crate::model::BUCKETS},
                    "idempotency_key":{"type":"string","format":"uuid"},"payload":{"type":"string"},"metadata":{"not":{"type":"null"}},"created_at":{"type":"string","format":"date-time","description":"UTC with Z suffix"}}},
                "Problem": {"type":"object","required":["type","title","status","detail"],"properties":{"type":{"type":"string","const":"about:blank"},"title":{"type":"string"},"status":{"type":"integer"},"detail":{"type":"string"}}}
            }
        },
        "paths": {
            "/api/health":{"get":{"summary":"Liveness", "responses":{"200":{"description":"Alive","content":{"text/plain":{"schema":{"type":"string","const":"ok"}}}}}}},
            "/api/buckets/{bucket}/records":{"post":{
                "summary":"Create an immutable record, or retry by idempotency key",
                "parameters":[{"name":"bucket","in":"path","required":true,"schema":{"type":"string","enum":crate::model::BUCKETS}}],
                "requestBody":{"required":true,"content":{"application/json":{"schema":{"$ref":"#/components/schemas/CreateRecord"}}}},
                "responses":{
                    "201":created("New record"),"200":created("Matching retry"),
                    "400":problem("Malformed or unsupported input"),"409":problem("Different content for the same key"),"415":problem("Unsupported content type"),"500":problem("Unexpected failure"),"503":problem("Storage temporarily unavailable")
                }
            }},
            "/api/buckets/{bucket}/records/{id}":{"get":{
                "summary":"Retrieve a record within its bucket",
                "parameters":[{"name":"bucket","in":"path","required":true,"schema":{"type":"string","enum":crate::model::BUCKETS}},{"name":"id","in":"path","required":true,"schema":{"type":"string","format":"uuid","description":"UUIDv7"}}],
                "responses":{"200":{"description":"Record","content":{"application/json":{"schema":{"$ref":"#/components/schemas/Record"}}}},"400":problem("Invalid bucket or UUIDv7"),"404":problem("No matching record"),"500":problem("Unexpected failure"),"503":problem("Storage temporarily unavailable")}
            }},
            "/api/openapi.json":{"get":{"summary":"OpenAPI document","responses":{"200":{"description":"OpenAPI 3.1 document","content":{"application/json":{"schema":{"type":"object"}}}}}}},
            "/api/docs":{"get":{"summary":"Swagger UI","responses":{"303":{"description":"Redirect to /api/docs/","headers":{"Location":{"schema":{"type":"string"}}}}}}},
            "/api/docs/":{"get":{"summary":"Swagger UI page","responses":{"200":{"description":"Swagger UI","content":{"text/html":{"schema":{"type":"string"}}}}}}}
        }
    });
    document["components"]["schemas"]["TableStats"] = json!({
        "type":"object", "required":["table_name","row_count","table_bytes","index_bytes","total_bytes"],
        "properties":{
            "table_name":{"type":"string"},
            "row_count":{"type":"integer","minimum":0,"description":"Estimated live rows from PostgreSQL statistics; may lag behind writes."},
            "table_bytes":{"type":"integer","minimum":0,"description":"Table storage including TOAST and auxiliary storage."},
            "index_bytes":{"type":"integer","minimum":0},
            "total_bytes":{"type":"integer","minimum":0}
        }
    });
    document["components"]["schemas"]["DatabaseTables"] = json!({
        "type":"object", "required":["schema_name","generated_at","row_count_kind","tables"],
        "properties":{
            "schema_name":{"type":"string"}, "generated_at":{"type":"string","format":"date-time"},
            "row_count_kind":{"type":"string","enum":["estimated"]},
            "tables":{"type":"array","items":{"$ref":"#/components/schemas/TableStats"}}
        }
    });
    document["paths"]["/api/reporting/database-tables"] = json!({"get":{
        "summary":"Active schema table storage and estimated live rows", "tags":["reporting"],
        "responses":{
            "200":{"description":"Database table report","content":{"application/json":{"schema":{"$ref":"#/components/schemas/DatabaseTables"}}}},
            "500":problem("Unexpected reporting failure"), "503":problem("Database unavailable")
        }
    }});
    for path in document["paths"].as_object_mut().unwrap().values_mut() {
        for operation in path.as_object_mut().unwrap().values_mut() {
            let mut auth = problem("Missing or invalid bearer token");
            auth["headers"] =
                json!({"WWW-Authenticate":{"schema":{"type":"string","const":"Bearer"}}});
            operation["responses"]["401"] = auth;
        }
    }
    document
}
fn problem(description: &str) -> Value {
    json!({"description":description,"content":{"application/problem+json":{"schema":{"$ref":"#/components/schemas/Problem"}}}})
}
fn created(description: &str) -> Value {
    json!({"description":description,"headers":{"Location":{"description":"Relative retrieval URL","schema":{"type":"string"}}},"content":{"application/json":{"schema":{"$ref":"#/components/schemas/Created"}}}})
}
