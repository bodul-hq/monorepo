use super::*;
use poem::{http::StatusCode, test::TestClient};
use serde_json::{Value, json};

struct Memory(Vec<Offer>);

impl Repository for Memory {
    fn load(&self, limit: i64, offset: i64) -> Result<Vec<Offer>, ()> {
        Ok(self
            .0
            .iter()
            .skip(offset as usize)
            .take(limit as usize)
            .cloned()
            .collect())
    }
}

pub(crate) fn offers(items: Vec<Offer>) -> Offers {
    Offers::new(Arc::new(Memory(items)))
}

fn fixture(index: u128) -> Offer {
    Offer {
        id: Uuid::from_u128(index).to_string(),
        retailer_code: "ankerau".into(),
        title: " \tČaj 🔌\n".into(),
        price: OfferPrice {
            amount: i64::MAX.to_string(),
            currency: "AUD".into(),
        },
        url: "https://shop.example/Charger?a=1&variant=2#details".into(),
        created_at: "2026-10-24T12:00:00.123456Z".into(),
    }
}

fn client(offers: Offers) -> TestClient<poem::Route> {
    let mut state = crate::queue_tests::state();
    state.offers = offers;
    TestClient::new(crate::app(state))
}

#[tokio::test]
async fn pagination_defaults_boundaries_and_transport_parity() {
    let client = client(offers((0..31).map(fixture).collect()));
    for (suffix, limit, offset, count, more) in [
        ("", 30, 0, 30, true),
        ("?offset=30", 30, 30, 1, false),
        ("?limit=1&offset=29", 1, 29, 1, true),
        ("?limit=1&offset=30", 1, 30, 1, false),
        ("?limit=100&offset=31", 100, 31, 0, false),
        ("?offset=2147483647", 30, i32::MAX, 0, false),
    ] {
        let response = client.get(format!("/api/offers{suffix}")).send().await;
        response.assert_status_is_ok();
        let page: Value = response.json().await.value().deserialize();
        assert_eq!(page["limit"], limit);
        assert_eq!(page["offset"], offset);
        assert_eq!(page["items"].as_array().unwrap().len(), count);
        assert_eq!(page["has_more"], more);
        let query = format!(
            "{{ offers(limit:{limit}, offset:{offset}) {{ limit offset hasMore items {{ id retailerCode title price {{ amount currency }} url createdAt }} }} }}"
        );
        for get in [false, true] {
            let response = if get {
                client.get("/graphql").query("query", &query).send().await
            } else {
                client
                    .post("/graphql")
                    .body_json(&json!({"query":query}))
                    .send()
                    .await
            };
            response.assert_status_is_ok();
            let result: Value = response.json().await.value().deserialize();
            let mut expected = page.clone();
            let obj = expected.as_object_mut().unwrap();
            let more = obj.remove("has_more").unwrap();
            obj.insert("hasMore".into(), more);
            for item in obj["items"].as_array_mut().unwrap() {
                let item = item.as_object_mut().unwrap();
                for (from, to) in [
                    ("retailer_code", "retailerCode"),
                    ("created_at", "createdAt"),
                ] {
                    let value = item.remove(from).unwrap();
                    item.insert(to.into(), value);
                }
            }
            assert_eq!(result, json!({"data":{"offers":expected}}));
        }
    }
    let response = client
        .post("/graphql")
        .body_json(&json!({"query":"{ offers { limit offset hasMore } }"}))
        .send()
        .await;
    response
        .assert_json(json!({"data":{"offers":{"limit":30,"offset":0,"hasMore":true}}}))
        .await;
}

#[tokio::test]
async fn invalid_pagination_is_rejected_by_both_transports() {
    let client = client(offers(Vec::new()));
    for params in [
        "limit=0",
        "limit=-1",
        "limit=101",
        "limit=2147483648",
        "limit=1.5",
        "limit=no",
        "limit=",
        "offset=-1",
        "offset=2147483648",
        "offset=1.5",
        "offset=no",
        "offset=",
    ] {
        client
            .get(format!("/api/offers?{params}"))
            .send()
            .await
            .assert_status(StatusCode::BAD_REQUEST);
    }
    for args in [
        "limit:0",
        "limit:-1",
        "limit:101",
        "limit:2147483648",
        "limit:1.5",
        "limit:null",
        "offset:-1",
        "offset:2147483648",
        "offset:1.5",
        "offset:null",
    ] {
        let response = client
            .post("/graphql")
            .body_json(&json!({"query":format!("{{ offers({args}) {{ limit }} }}")}))
            .send()
            .await;
        let result: Value = response.json().await.value().deserialize();
        assert!(result["errors"].is_array(), "{args}: {result}");
    }
}

struct Broken(bool);
impl Repository for Broken {
    fn load(&self, _: i64, _: i64) -> Result<Vec<Offer>, ()> {
        assert!(!self.0, "private database diagnostic");
        Err(())
    }
}

#[tokio::test]
async fn empty_pages_and_sanitized_read_failures() {
    client(offers(Vec::new()))
        .get("/api/offers")
        .send()
        .await
        .assert_json(json!({"items":[],"limit":30,"offset":0,"has_more":false}))
        .await;
    for panic in [false, true] {
        let client = client(Offers::new(Arc::new(Broken(panic))));
        let response = client.get("/api/offers").send().await;
        response.assert_status(StatusCode::INTERNAL_SERVER_ERROR);
        response.assert_text("offers could not be retrieved").await;
        let response = client
            .post("/graphql")
            .body_json(&json!({"query":"{ offers { items { id } } }"}))
            .send()
            .await;
        let result: Value = response.json().await.value().deserialize();
        assert_eq!(
            result["errors"][0]["message"],
            "offers could not be retrieved"
        );
        assert!(!result.to_string().contains("private"));
    }
}

#[tokio::test]
async fn schema_exposes_bounds_and_string_money() {
    let client = client(offers(Vec::new()));
    let response = client.get("/api/openapi.json").send().await;
    let spec: Value = response.json().await.value().deserialize();
    let op = &spec["paths"]["/offers"]["get"];
    assert_eq!(op["parameters"][0]["schema"]["default"], 30);
    assert_eq!(
        op["parameters"][0]["schema"]["maximum"].as_f64(),
        Some(100.0)
    );
    assert_eq!(op["parameters"][1]["schema"]["default"], 0);
    assert_eq!(
        spec["components"]["schemas"]["OfferPrice"]["properties"]["amount"]["type"],
        "string"
    );
    for status in ["200", "400", "500"] {
        assert!(op["responses"][status].is_object());
    }
}
