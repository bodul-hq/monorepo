use super::{
    acquire_storefront_token as token, offer_details as details, product_catalog_ids as catalog,
    transport::Http,
};
use crate::platform::{bus::Handle, object_storage::OfferStorageMetadata};
use retailer_sauce::shopify::initial_fill::{GraphqlRequest, StorefrontToken};
use serde_json::{Value, json};
use shared::retailer::RetailerCode;
use std::{collections::VecDeque, sync::Mutex};
use uuid::Uuid;

pub(super) fn token_value() -> StorefrontToken {
    StorefrontToken::new("synthetic-au-token".into()).unwrap()
}

#[test]
fn retailer_payloads_read_legacy_names_write_slugs_and_keep_command_ids() {
    let run_id = Uuid::from_u128(1);
    let old_command =
        json!({"run_id":run_id,"retailer_code":"AnkerAu","url":"https://www.anker.com/au"});
    let command: token::command::AcquireStorefrontToken =
        serde_json::from_value(old_command).unwrap();
    let encoded = serde_json::to_value(&command).unwrap();
    assert_eq!(encoded["retailer_code"], "ankerau");
    assert_eq!(
        serde_json::from_value::<token::command::AcquireStorefrontToken>(encoded)
            .unwrap()
            .retailer_code,
        RetailerCode::AnkerAu
    );

    let old_event = json!({
        "run_id":run_id,"retailer_code":"AnkerAu","storefront_token":"synthetic-au-token",
        "after":null,"product_ids":["gid://shopify/Product/1"],"next_cursor":null,
        "retrieved_at":"2026-11-09T00:00:00Z"
    });
    let old_event: catalog::event::ProductCatalogIdsRetrieved =
        serde_json::from_value(old_event).unwrap();
    let encoded = serde_json::to_value(&old_event).unwrap();
    assert_eq!(encoded["retailer_code"], "ankerau");
    let new_event: catalog::event::ProductCatalogIdsRetrieved =
        serde_json::from_value(encoded).unwrap();
    let old_key = br#"["FetchOfferDetails","AnkerAu","gid://shopify/Product/1"]"#;
    let expected_id = crate::platform::bus::stable_command_id(&run_id, old_key);
    for event in [old_event, new_event] {
        let queued = details::reacts_to::on(&event).remove(0);
        assert_eq!(queued.id, expected_id);
        assert_eq!(
            serde_json::to_value(&queued.command).unwrap()["retailer_code"],
            "ankerau"
        );
    }
}
pub(super) fn connection(edges: Vec<Value>, more: bool, cursor: Option<&str>) -> Value {
    json!({"edges":edges,"pageInfo":{"hasNextPage":more,"endCursor":cursor}})
}
pub(super) fn product(id: &str) -> Value {
    let mut value: Value = serde_json::from_str(include_str!(
        "../../../../lib/retailers_anker/src/ankerau/offer_details/v2/fixtures/product.json"
    ))
    .unwrap();
    value["id"] = json!(id);
    for field in [
        "collections",
        "images",
        "media",
        "variants",
        "sellingPlanGroups",
    ] {
        value[field] = connection(vec![], false, None);
    }
    value
}
fn sample_node(field: &str) -> Value {
    let value: Value = serde_json::from_str(include_str!(
        "../../../../lib/retailers_anker/src/ankerau/offer_details/v2/fixtures/product.json"
    ))
    .unwrap();
    value[field]["edges"][0]["node"].clone()
}
pub(super) struct FakeHttp {
    pub expected_token: Option<StorefrontToken>,
    pub responses: Mutex<VecDeque<Result<Value, String>>>,
    pub calls: Mutex<Vec<GraphqlRequest>>,
}
impl Default for FakeHttp {
    fn default() -> Self {
        Self {
            expected_token: Some(token_value()),
            responses: Mutex::new(VecDeque::new()),
            calls: Mutex::new(Vec::new()),
        }
    }
}
impl FakeHttp {
    pub fn from(responses: Vec<Value>) -> Self {
        Self {
            responses: Mutex::new(responses.into_iter().map(Ok).collect()),
            ..Default::default()
        }
    }
}
impl Http for FakeHttp {
    fn post(
        &self,
        _: &str,
        operation: &GraphqlRequest,
        token: Option<&StorefrontToken>,
    ) -> Result<Value, String> {
        assert_eq!(token, self.expected_token.as_ref());
        self.calls.lock().unwrap().push(operation.clone());
        self.responses
            .lock()
            .unwrap()
            .pop_front()
            .expect("unexpected extra request")
    }
}
fn catalog_page(start: usize, count: usize, more: bool, cursor: &str) -> Value {
    let edges = (start..start+count).map(|i| json!({"cursor":format!("p{i}"),"node":{"id":format!("gid://shopify/Product/{i}")}})).collect();
    json!({"data":{"products":connection(edges,more,Some(cursor))}})
}
pub(super) fn catalog_command(
    run_id: Uuid,
    after: Option<String>,
) -> catalog::command::FetchProductCatalogIdentifiers {
    catalog::command::FetchProductCatalogIdentifiers {
        context: None,
        run_id,
        retailer_code: RetailerCode::AnkerAu,
        storefront_token: Some(token_value()),
        after,
    }
}
#[test]
fn catalog_batches_500_and_reactions_continue_then_stop() {
    let handler = catalog::handler::Handler(FakeHttp::from(vec![
        catalog_page(1, 250, true, "a"),
        catalog_page(251, 250, true, "b"),
        catalog_page(501, 2, false, "c"),
    ]));
    let run_id = Uuid::now_v7();
    let event = handler
        .handle(catalog_command(run_id, None))
        .unwrap()
        .remove(0);
    assert_eq!(event.product_ids.len(), 500);
    assert_eq!(event.next_cursor.as_deref(), Some("b"));
    let commands = details::reacts_to::on(&event);
    assert_eq!(commands.len(), 500);
    let again = details::reacts_to::on(&event);
    assert_eq!(commands[0].id, again[0].id);
    assert_eq!(commands[0].run_id, run_id);
    let next = catalog::reacts_to::next_batch(&event).remove(0);
    let event = handler.handle(next.command).unwrap().remove(0);
    assert_eq!(event.product_ids.len(), 2);
    assert!(event.next_cursor.is_none());
    assert!(catalog::reacts_to::next_batch(&event).is_empty());
    assert!(handler.0.calls.lock().unwrap()[2].variables["after"] == "b");
}
#[test]
fn catalog_empty_short_duplicate_and_exact_batches() {
    let empty = catalog::handler::Handler(FakeHttp::from(vec![catalog_page(1, 0, false, "end")]));
    let event = empty
        .handle(catalog_command(Uuid::now_v7(), None))
        .unwrap()
        .remove(0);
    assert!(event.product_ids.is_empty());
    assert!(details::reacts_to::on(&event).is_empty());
    let short = catalog::handler::Handler(FakeHttp::from(vec![
        catalog_page(1, 1, true, "a"),
        catalog_page(1, 2, false, "b"),
    ]));
    assert_eq!(
        short.handle(catalog_command(Uuid::now_v7(), None)).unwrap()[0]
            .product_ids
            .len(),
        2
    );
    let exact = catalog::handler::Handler(FakeHttp::from(vec![
        catalog_page(1, 250, true, "a"),
        catalog_page(251, 250, false, "b"),
    ]));
    let event = exact
        .handle(catalog_command(Uuid::now_v7(), None))
        .unwrap()
        .remove(0);
    assert_eq!(event.product_ids.len(), 500);
    assert!(event.next_cursor.is_none());
}
#[test]
fn catalog_rejects_stalled_missing_cursors_invalid_ids_and_transport_errors() {
    for pages in [
        vec![catalog_page(1, 1, true, "a"), catalog_page(2, 1, true, "a")],
        vec![json!({"data":{"products":connection(vec![],true,None)}})],
        vec![
            json!({"data":{"products":connection(vec![json!({"cursor":"a","node":{"id":"bad"}})],false,None)}}),
        ],
    ] {
        assert!(
            catalog::handler::Handler(FakeHttp::from(pages))
                .handle(catalog_command(Uuid::now_v7(), None))
                .is_err()
        );
    }
    let fake = FakeHttp::default();
    fake.responses
        .lock()
        .unwrap()
        .push_back(Err("network failed".into()));
    assert!(
        catalog::handler::Handler(fake)
            .handle(catalog_command(Uuid::now_v7(), None))
            .is_err()
    );
}

struct Documents(std::collections::HashMap<String, String>);
impl token::handler::Fetch for Documents {
    fn configured_token(&self, name: &str) -> Result<Option<StorefrontToken>, String> {
        Ok((name == "ANKER_NORDICS_STOREFRONT_TOKEN").then(token_value))
    }
    fn get(&self, url: &str) -> Result<String, String> {
        self.0
            .get(url)
            .cloned()
            .ok_or_else(|| "document fetch failed".into())
    }
}
#[test]
fn downloaded_au_storefront_emits_the_expected_token() {
    let root =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/storefront_token");
    let assets: Vec<serde_json::Value> =
        serde_json::from_str(&std::fs::read_to_string(root.join("assets.json")).unwrap()).unwrap();
    let mut documents = std::collections::HashMap::from([(
        "https://www.anker.com/au".to_owned(),
        std::fs::read_to_string(root.join("ankerau.html")).unwrap(),
    )]);
    for asset in assets {
        documents.insert(
            asset["url"].as_str().unwrap().to_owned(),
            std::fs::read_to_string(root.join(asset["file"].as_str().unwrap())).unwrap(),
        );
    }
    // Independently read from assets/16.js: storeTokens.au, used by that
    // bundle's X-Http-Storefront-Access-Token header implementation.
    let expected = StorefrontToken::new("ca89549ed256ec3f2c3a12d113b9f29e".into()).unwrap();
    let event = token::handler::Handler(Documents(documents))
        .handle(token::command::AcquireStorefrontToken {
            run_id: Uuid::now_v7(),
            retailer_code: RetailerCode::AnkerAu,
            url: "https://www.anker.com/au".into(),
        })
        .unwrap()
        .remove(0);
    assert_eq!(event.storefront_token, Some(expected.clone()));
    assert_eq!(event.retailer_code, RetailerCode::AnkerAu);
    let queued = catalog::reacts_to::acquired(&event).remove(0);
    assert_eq!(queued.command.storefront_token, Some(expected));
}

#[test]
fn regional_bundle_resolves_imported_tokens_for_fr_pl_nz_vn_eu_and_us() {
    let bundle = include_str!("../../tests/fixtures/storefront_token/regional/app.js");
    let expected: Value = serde_json::from_str(include_str!(
        "../../tests/fixtures/storefront_token/regional/expected.json"
    ))
    .unwrap();
    let script_url = "https://www.anker.com/_next/static/chunks/pages/_app-5b4c0e10ae40069f.js";
    for retailer_code in [
        RetailerCode::AnkerFr,
        RetailerCode::AnkerPl,
        RetailerCode::AnkerNz,
        RetailerCode::AnkerVn,
        RetailerCode::AnkerEu,
        RetailerCode::AnkerCom,
    ] {
        let config = retailer_data_extractor::initial_fill_config_for(retailer_code).unwrap();
        // Minimal HTML referencing the exact shared bundle observed on all six homepages.
        let handler = token::handler::Handler(Documents(
            [
                (
                    config.storefront_url.to_owned(),
                    format!(r#"<script src="{script_url}"></script>"#),
                ),
                (script_url.to_owned(), bundle.to_owned()),
            ]
            .into(),
        ));
        let run_id = Uuid::now_v7();
        let event = handler
            .handle(token::command::AcquireStorefrontToken {
                run_id,
                retailer_code,
                url: config.storefront_url.into(),
            })
            .unwrap()
            .remove(0);
        let expected = StorefrontToken::new(
            expected[config.token_market.unwrap()]
                .as_str()
                .unwrap()
                .into(),
        )
        .unwrap();
        assert_eq!(
            event.storefront_token,
            Some(expected.clone()),
            "{retailer_code:?}"
        );
        assert_eq!(event.run_id, run_id);
        assert_eq!(event.retailer_code, retailer_code);
        assert_eq!(
            catalog::reacts_to::acquired(&event)[0]
                .command
                .storefront_token,
            Some(expected)
        );
    }
}

#[test]
fn token_handler_fetches_assets_and_emits_market_token_and_timestamp() {
    let handler=token::handler::Handler(Documents([
        ("https://www.anker.com/au".into(),"<script src='/app.js'></script><script src='https://third.party/ignored.js'></script>".into()),
        ("https://www.anker.com/app.js".into(),"window.config={storeTokens:{au:'synthetic-au-token',ca:'other'}}".into()),
    ].into()));
    let start = chrono::Utc::now();
    let run_id = Uuid::now_v7();
    let command = token::command::AcquireStorefrontToken {
        run_id,
        retailer_code: RetailerCode::AnkerAu,
        url: "https://www.anker.com/au".into(),
    };
    let event = handler.handle(command).unwrap().remove(0);
    assert_eq!(event.storefront_token, Some(token_value()));
    assert!(event.acquired_at >= start);
    let queued = catalog::reacts_to::acquired(&event).remove(0);
    assert_eq!(queued.command.run_id, run_id);
    assert!(queued.command.after.is_none());
}
#[test]
fn token_handler_rejects_missing_token_failed_assets_and_arbitrary_urls() {
    for html in [
        "<html></html>",
        "<script src='/missing.js'></script>",
        "<script>storeTokens:{au:''}</script>",
    ] {
        let handler = token::handler::Handler(Documents(
            [("https://www.anker.com/au".into(), html.into())].into(),
        ));
        let command = token::command::AcquireStorefrontToken {
            run_id: Uuid::now_v7(),
            retailer_code: RetailerCode::AnkerAu,
            url: "https://www.anker.com/au".into(),
        };
        assert!(handler.handle(command.clone()).is_err());
        assert!(
            handler
                .handle(token::command::AcquireStorefrontToken {
                    url: "https://other.test".into(),
                    ..command
                })
                .is_err()
        );
    }
}

pub(super) struct FakeStore {
    pub bodies: Mutex<Vec<String>>,
    pub fail: bool,
}
impl details::handler::Store for FakeStore {
    fn store(
        &self,
        body: &str,
        at: chrono::DateTime<chrono::Utc>,
    ) -> Result<OfferStorageMetadata, String> {
        if self.fail {
            return Err("storage failed".into());
        }
        self.bodies.lock().unwrap().push(body.into());
        Ok(OfferStorageMetadata {
            scraped_at: Some(at),
            content_size: body.len() as u64,
            object_storage_id: Uuid::now_v7(),
            external_id: Uuid::now_v7(),
        })
    }
}
pub(super) fn detail_command(run_id: Uuid, id: &str) -> details::command::FetchOfferDetails {
    details::command::FetchOfferDetails {
        context: None,
        run_id,
        retailer_code: RetailerCode::AnkerAu,
        storefront_token: Some(token_value()),
        source: details::SourceType::GraphQl {
            product_id: id.into(),
        },
    }
}
#[test]
fn detail_handler_preserves_raw_evidence_and_emits_only_after_storage() {
    let id = "gid://shopify/Product/1";
    let raw = json!({"data":{"product":product(id)},"extensions":{"unexpected":"preserved"}});
    let handler = details::handler::Handler {
        fetcher: FakeHttp::from(vec![raw.clone()]),
        store: FakeStore {
            bodies: Mutex::new(vec![]),
            fail: false,
        },
    };
    let event = handler
        .handle(detail_command(Uuid::now_v7(), id))
        .unwrap()
        .remove(0);
    let bodies = handler.store.bodies.lock().unwrap();
    let artifact: Value = serde_json::from_str(&bodies[0]).unwrap();
    assert_eq!(artifact["pages"][0]["response"], raw);
    assert_eq!(artifact["product"]["id"], id);
    assert_eq!(event.storage.content_size, bodies[0].len() as u64);
    assert_eq!(event.storage.scraped_at, Some(event.fetched_at));
    assert!(!bodies[0].contains("synthetic-au-token"));
    let failing = details::handler::Handler {
        fetcher: FakeHttp::from(vec![raw]),
        store: FakeStore {
            bodies: Mutex::new(vec![]),
            fail: true,
        },
    };
    assert!(failing.handle(detail_command(Uuid::now_v7(), id)).is_err());
}
#[test]
fn details_fail_for_missing_wrong_product_currency_or_unsupported_html() {
    let id = "gid://shopify/Product/1";
    let mut wrong_currency = product(id);
    wrong_currency["priceRange"]["minVariantPrice"]["currencyCode"] = json!("USD");
    for raw in [
        json!({"data":{"product":null}}),
        json!({"data":{"product":product("gid://shopify/Product/2")}}),
        json!({"data":{"product":wrong_currency}}),
    ] {
        let handler = details::handler::Handler {
            fetcher: FakeHttp::from(vec![raw]),
            store: FakeStore {
                bodies: Mutex::new(vec![]),
                fail: false,
            },
        };
        assert!(handler.handle(detail_command(Uuid::now_v7(), id)).is_err());
        assert!(handler.store.bodies.lock().unwrap().is_empty());
    }
    let handler = details::handler::Handler {
        fetcher: FakeHttp::default(),
        store: FakeStore {
            bodies: Mutex::new(vec![]),
            fail: false,
        },
    };
    let mut command = detail_command(Uuid::now_v7(), id);
    command.source = details::SourceType::Html {
        url: "https://example.test".into(),
    };
    assert!(handler.handle(command).is_err());
    assert!(handler.fetcher.calls.lock().unwrap().is_empty());
}

#[test]
fn every_product_variant_and_selling_plan_connection_is_exhausted() {
    let id = "gid://shopify/Product/1";
    let variant_id = "gid://shopify/ProductVariant/10";
    let mut variant = sample_node("variants");
    variant["id"] = json!(variant_id);
    let fields = [
        "components",
        "groupedBy",
        "quantityPriceBreaks",
        "sellingPlanAllocations",
        "storeAvailability",
    ];
    for field in fields {
        variant[field] = connection(vec![], true, Some(&format!("{field}-1")));
    }
    let group1 = json!({"__typename":"SellingPlanGroup", "appName":null,"options":[],"name":"first","sellingPlans":connection(vec![],true,Some("plans-1"))});
    let group2 = json!({"__typename":"SellingPlanGroup", "appName":null,"options":[],"name":"second","sellingPlans":connection(vec![],true,Some("plans-2"))});
    let mut initial = product(id);
    let product_fields = [
        "collections",
        "images",
        "media",
        "variants",
        "sellingPlanGroups",
    ];
    for field in product_fields {
        let node = match field {
            "variants" => variant.clone(),
            "sellingPlanGroups" => group1.clone(),
            _ => {
                let mut node = sample_node(field);
                node["id"] = json!(format!("{field}-one"));
                node
            }
        };
        initial[field] = connection(
            vec![json!({"cursor":format!("{field}-1"),"node":node})],
            true,
            Some(&format!("{field}-1")),
        );
    }
    let mut responses = vec![json!({"data":{"product":initial}})];
    for field in product_fields {
        let edges = match field {
            "variants" => vec![],
            "sellingPlanGroups" => vec![json!({"cursor":"sellingPlanGroups-2","node":group2})],
            _ => {
                let mut node = sample_node(field);
                node["id"] = json!(format!("{field}-two"));
                vec![json!({"cursor":format!("{field}-2"),"node":node})]
            }
        };
        let mut parent = json!({"id":id});
        parent[field] = connection(edges, false, Some(&format!("{field}-2")));
        responses.push(json!({"data":{"product":parent}}));
    }
    for field in fields {
        let mut parent = json!({"__typename":"ProductVariant", "id":variant_id});
        parent[field] = connection(vec![], false, Some(&format!("{field}-2")));
        responses.push(json!({"data":{"node":parent}}));
    }
    for n in [1, 2] {
        responses.push(json!({"data":{"product":{"id":id,"sellingPlanGroups":{"edges":[{"cursor":format!("sellingPlanGroups-{n}"),"node":{"sellingPlans":connection(vec![],false,Some("end"))}}]}}}}));
    }
    for retailer in [
        RetailerCode::AnkerAu,
        RetailerCode::AnkerCa,
        RetailerCode::AnkerJapanCom,
    ] {
        let config = retailer_data_extractor::initial_fill_config_for(retailer).unwrap();
        let access = config.requires_token().then(token_value);
        let fake = FakeHttp {
            expected_token: access.clone(),
            ..FakeHttp::from(
                serde_json::from_str(
                    &serde_json::to_string(&responses)
                        .unwrap()
                        .replace("AUD", config.currency),
                )
                .unwrap(),
            )
        };
        let result = fetch_details(&fake, retailer, access.as_ref(), id).unwrap();
        assert_eq!(result.pages.len(), 13);
        let config = retailer_data_extractor::initial_fill_config_for(retailer).unwrap();
        for page in &result.pages {
            assert_eq!(page.request.variables["country"], config.country);
            assert_eq!(page.request.variables["language"], config.language);
            assert!(page.request.query.contains("language: $language"));
        }
        for field in product_fields {
            assert_eq!(result.product[field]["pageInfo"]["hasNextPage"], false);
        }
        for field in fields {
            assert_eq!(
                result.product["variants"]["edges"][0]["node"][field]["edges"]
                    .as_array()
                    .unwrap()
                    .len(),
                0
            );
        }
        let calls = fake.calls.lock().unwrap();
        assert!(
            calls[11].operation_name == "SellingPlans"
                && calls[11].variables["groupAfter"].is_null()
        );
        assert!(
            calls[12].operation_name == "SellingPlans"
                && calls[12].variables["groupAfter"] == "sellingPlanGroups-1"
        );
    }
}
#[test]
fn detail_pagination_rejects_stalls_and_wrong_parent_identity() {
    let id = "gid://shopify/Product/1";
    let mut p = product(id);
    p["collections"] = connection(vec![], true, Some("a"));
    for response in [
        json!({"data":{"product":{"id":id,"collections":connection(vec![],true,Some("a"))}}}),
        json!({"data":{"product":{"id":"gid://shopify/Product/2","collections":connection(vec![],false,Some("b"))}}}),
    ] {
        assert!(
            fetch_details(
                &FakeHttp::from(vec![json!({"data":{"product":p.clone()}}), response]),
                RetailerCode::AnkerAu,
                Some(&token_value()),
                id
            )
            .is_err()
        );
    }
}
#[tokio::test]
async fn api_validates_retailers_and_documents_the_new_route() {
    use axum::{
        body::{Body, to_bytes},
        http::{Request, StatusCode},
    };
    use tower::ServiceExt;
    let mut cases = vec![
        ("unknown", StatusCode::BAD_REQUEST),
        ("micom", StatusCode::BAD_REQUEST),
        ("ankerkr", StatusCode::BAD_REQUEST),
        ("ankerca", StatusCode::SERVICE_UNAVAILABLE),
        ("ankerau", StatusCode::SERVICE_UNAVAILABLE),
        ("ankernordicscom", StatusCode::BAD_REQUEST),
    ];
    let supported: Vec<String> = RetailerCode::ALL
        .into_iter()
        .filter(|code| retailer_data_extractor::initial_fill_config_for(*code).is_some())
        .map(|code| code.as_str())
        .collect();
    cases.extend(
        supported
            .iter()
            .map(|code| (code.as_str(), StatusCode::SERVICE_UNAVAILABLE)),
    );
    for (retailer, status) in cases {
        let response = crate::app(Default::default())
            .oneshot(
                Request::post("/api/initial-offer-fills")
                    .header("content-type", "application/json")
                    .body(Body::from(json!({"retailers":[retailer]}).to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), status, "{retailer}");
    }
    for (body, status) in [
        (
            json!({"retailers": supported}),
            StatusCode::SERVICE_UNAVAILABLE,
        ),
        (
            json!({"retailers": ["ankerau", "ankeritalycom", "ankerau"]}),
            StatusCode::SERVICE_UNAVAILABLE,
        ),
        (json!({"retailers": []}), StatusCode::BAD_REQUEST),
        (
            json!({"retailers": ["ankerau", "unknown"]}),
            StatusCode::BAD_REQUEST,
        ),
        (
            json!({"retailers": ["ankerau", "ankerkr"]}),
            StatusCode::BAD_REQUEST,
        ),
        (
            json!({"retailers": ["ankerau", "ankernordicscom"]}),
            StatusCode::BAD_REQUEST,
        ),
        (
            json!({"retailer": "ankerau"}),
            StatusCode::UNPROCESSABLE_ENTITY,
        ),
        (
            json!({"retailers": "ankerau"}),
            StatusCode::UNPROCESSABLE_ENTITY,
        ),
    ] {
        let response = crate::app(Default::default())
            .oneshot(
                Request::post("/api/initial-offer-fills")
                    .header("content-type", "application/json")
                    .body(Body::from(body.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), status, "{body}");
    }
    let response = crate::app(Default::default())
        .oneshot(
            Request::get("/api/openapi.json")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let body: Value =
        serde_json::from_slice(&to_bytes(response.into_body(), usize::MAX).await.unwrap()).unwrap();
    assert!(body["paths"]["/initial-offer-fills"]["post"]["responses"]["202"].is_object());
    let schemas = &body["components"]["schemas"];
    assert_eq!(
        schemas["InitialFillRequest"]["properties"]["retailers"]["type"],
        "array"
    );
    assert_eq!(
        schemas["InitialFillRequest"]["required"],
        json!(["retailers"])
    );
    assert_eq!(
        schemas["InitialFillResponse"]["properties"]["fills"]["type"],
        "array"
    );
    assert_eq!(
        schemas["QueuedInitialFill"]["properties"]["run_id"]["format"],
        "uuid"
    );
    assert!(schemas["InitialFillQueueFailure"]["properties"]["fills"].is_object());
    assert_eq!(
        body["paths"]["/initial-offer-fills"]["post"]["responses"]["500"]["content"]["application/json"]
            ["schema"]["$ref"],
        "#/components/schemas/InitialFillQueueFailure"
    );
}

#[test]
fn canada_acquires_its_token_batches_500_and_stores_english_cad_details() {
    let config = retailer_data_extractor::initial_fill_config_for(RetailerCode::AnkerCa).unwrap();
    let script_url = "https://one.anker.com/_next/static/chunks/1b2878kmn7hql.js";
    // Synthetic HTML referencing the actual captured multi-market Anker bundle.
    // This is not a recorded Canadian homepage.
    let bundle = include_str!("../../tests/fixtures/storefront_token/assets/16.js");
    assert!(bundle.contains(r#""ca":"anker-canada.myshopify.com""#));
    assert_eq!(
        config.graphql_url,
        "https://anker-canada.myshopify.com/api/2026-10/graphql.json"
    );
    let handler = token::handler::Handler(Documents(std::collections::HashMap::from([
        (
            config.storefront_url.into(),
            format!(r#"<script src="{script_url}"></script>"#),
        ),
        (script_url.into(), bundle.into()),
    ])));
    let run_id = Uuid::now_v7();
    let acquired = handler
        .handle(token::command::AcquireStorefrontToken {
            run_id,
            retailer_code: RetailerCode::AnkerCa,
            url: config.storefront_url.into(),
        })
        .unwrap()
        .remove(0);
    // Independently read from storeTokens.ca in the captured bundle.
    assert_eq!(
        acquired.storefront_token.as_ref().unwrap().as_str(),
        "0a32b45f34ceb286e6757f7b8e71b4b5"
    );
    let expected_token = acquired.storefront_token.clone();
    let catalog = catalog::handler::Handler(FakeHttp {
        expected_token: expected_token.clone(),
        ..FakeHttp::from(vec![
            catalog_page(1, 250, true, "a"),
            catalog_page(251, 250, true, "b"),
            catalog_page(501, 1, false, "c"),
        ])
    });
    let batch = catalog
        .handle(catalog::reacts_to::acquired(&acquired).remove(0).command)
        .unwrap()
        .remove(0);
    assert_eq!(batch.retailer_code, RetailerCode::AnkerCa);
    assert_eq!(batch.run_id, run_id);
    assert_eq!(batch.product_ids.len(), 500);
    let commands = details::reacts_to::on(&batch);
    assert_eq!(commands.len(), 500);
    assert!(
        commands
            .iter()
            .all(|queued| queued.command.retailer_code == RetailerCode::AnkerCa)
    );
    let remaining = catalog
        .handle(catalog::reacts_to::next_batch(&batch).remove(0).command)
        .unwrap()
        .remove(0);
    assert_eq!(remaining.product_ids.len(), 1);
    assert!(catalog::reacts_to::next_batch(&remaining).is_empty());

    let mut cad_product = product("gid://shopify/Product/1");
    cad_product = serde_json::from_str(&cad_product.to_string().replace("AUD", "CAD")).unwrap();
    let handler = details::handler::Handler {
        fetcher: FakeHttp {
            expected_token,
            ..FakeHttp::from(vec![json!({"data":{"product":cad_product}})])
        },
        store: FakeStore {
            bodies: Mutex::new(vec![]),
            fail: false,
        },
    };
    let event = handler
        .handle(commands.into_iter().next().unwrap().command)
        .unwrap()
        .remove(0);
    assert_eq!(event.retailer_code, RetailerCode::AnkerCa);
    assert_eq!(event.run_id, run_id);
    let bodies = handler.store.bodies.lock().unwrap();
    let artifact: Value = serde_json::from_str(&bodies[0]).unwrap();
    assert_eq!(
        artifact["product"]["priceRange"]["minVariantPrice"]["currencyCode"],
        "CAD"
    );
    assert_eq!(
        artifact["pages"][0]["request"]["variables"]["country"],
        "CA"
    );
    assert!(
        artifact["pages"][0]["request"]["query"]
            .as_str()
            .unwrap()
            .contains("language: $language")
    );
    assert!(!bodies[0].contains(acquired.storefront_token.as_ref().unwrap().as_str()));
}

#[test]
fn canada_rejects_french_token_urls_and_wrong_currency_before_storage() {
    let command = token::command::AcquireStorefrontToken {
        run_id: Uuid::now_v7(),
        retailer_code: RetailerCode::AnkerCa,
        url: "https://www.anker.com/ca-fr".into(),
    };
    assert!(
        token::handler::Handler(Documents(Default::default()))
            .handle(command)
            .unwrap_err()
            .contains("URL does not match")
    );
    let only_french = Documents(std::collections::HashMap::from([(
        "https://www.anker.com/ca".into(),
        r#"<script>storeTokens:{"ca-fr":"french-only-token"}</script>"#.into(),
    )]));
    let error = token::handler::Handler(only_french)
        .handle(token::command::AcquireStorefrontToken {
            run_id: Uuid::now_v7(),
            retailer_code: RetailerCode::AnkerCa,
            url: "https://www.anker.com/ca".into(),
        })
        .unwrap_err();
    assert!(error.contains("token not found"));
    for currency in ["AUD", "USD", "EUR"] {
        let mut value = product("gid://shopify/Product/1");
        value["priceRange"]["minVariantPrice"]["currencyCode"] = json!(currency);
        let handler = details::handler::Handler {
            fetcher: FakeHttp::from(vec![json!({"data":{"product":value}})]),
            store: FakeStore {
                bodies: Mutex::new(vec![]),
                fail: false,
            },
        };
        let error = handler
            .handle(details::command::FetchOfferDetails {
                context: None,
                retailer_code: RetailerCode::AnkerCa,
                ..detail_command(Uuid::now_v7(), "gid://shopify/Product/1")
            })
            .unwrap_err();
        assert!(error.contains("CAD"));
        assert!(handler.store.bodies.lock().unwrap().is_empty());
    }
}

#[test]
fn every_market_preserves_its_token_policy_through_catalog_fanout_and_storage() {
    let mut count = 0;
    for retailer in RetailerCode::ALL {
        let Some(config) = retailer_data_extractor::initial_fill_config_for(retailer) else {
            continue;
        };
        count += 1;
        let run_id = Uuid::now_v7();
        let command = if config.requires_token() {
            let market = config.token_market.unwrap_or("unused");
            let html = format!(
                r#"<script>{{"storeTokens":{{"{market}":"synthetic-au-token","unrelated":"wrong-token"}}}}</script>"#
            );
            let acquired = token::handler::Handler(Documents(std::collections::HashMap::from([(
                config.storefront_url.into(),
                html,
            )])))
            .handle(token::command::AcquireStorefrontToken {
                run_id,
                retailer_code: retailer,
                url: config.storefront_url.into(),
            })
            .unwrap()
            .remove(0);
            catalog::reacts_to::acquired(&acquired).remove(0).command
        } else {
            let process = retailer_data_extractor::initial_fill_process(retailer);
            let mut expected = vec![
                retailer_data_extractor::Step::ProductCatalogIdentifiers,
                retailer_data_extractor::Step::OfferDetails,
            ];
            if retailer_data_extractor::all_market_contexts(retailer) {
                expected.insert(0, retailer_data_extractor::Step::DiscoverMarkets);
            }
            expected.insert(0, retailer_data_extractor::Step::AcquireStorefrontToken);
            assert_eq!(process.steps, expected);
            assert!(
                token::handler::Handler(Documents(Default::default()))
                    .handle(token::command::AcquireStorefrontToken {
                        run_id,
                        retailer_code: retailer,
                        url: config.storefront_url.into()
                    })
                    .is_err()
            );
            catalog::command::FetchProductCatalogIdentifiers {
                context: None,
                run_id,
                retailer_code: retailer,
                storefront_token: None,
                after: None,
            }
        };
        let expected_token = command.storefront_token.clone();
        let batch = catalog::handler::Handler(FakeHttp {
            expected_token: expected_token.clone(),
            ..FakeHttp::from(vec![
                catalog_page(1, 250, true, "a"),
                catalog_page(251, 250, false, "b"),
            ])
        })
        .handle(command)
        .unwrap()
        .remove(0);
        let commands = details::reacts_to::on(&batch);
        assert_eq!(commands.len(), 500);
        assert!(
            commands
                .iter()
                .all(|command| command.command.retailer_code == retailer
                    && command.command.storefront_token == expected_token)
        );
        assert!(catalog::reacts_to::next_batch(&batch).is_empty());
        let mut value = product("gid://shopify/Product/1");
        value = serde_json::from_str(&value.to_string().replace("AUD", config.currency)).unwrap();
        let handler = details::handler::Handler {
            fetcher: FakeHttp {
                expected_token,
                ..FakeHttp::from(vec![json!({"data":{"product":value}})])
            },
            store: FakeStore {
                bodies: Mutex::new(vec![]),
                fail: false,
            },
        };
        let event = handler
            .handle(commands.into_iter().next().unwrap().command)
            .unwrap()
            .remove(0);
        assert_eq!(event.retailer_code, retailer);
        assert_eq!(event.run_id, run_id);
        let bodies = handler.store.bodies.lock().unwrap();
        let artifact: Value = serde_json::from_str(&bodies[0]).unwrap();
        assert_eq!(
            artifact["pages"][0]["request"]["variables"]["country"],
            config.country
        );
        assert_eq!(
            artifact["pages"][0]["request"]["variables"]["language"],
            config.language
        );
        assert!(!bodies[0].contains("synthetic-au-token"));
    }
    assert_eq!(count, 48);
}

#[test]
fn nordic_acquisition_discovers_browser_tokens_and_preserves_environment_override() {
    struct NordicDocuments {
        configured: Result<Option<StorefrontToken>, String>,
        missing: bool,
    }
    impl token::handler::Fetch for NordicDocuments {
        fn configured_token(&self, name: &str) -> Result<Option<StorefrontToken>, String> {
            assert_eq!(name, "ANKER_NORDICS_STOREFRONT_TOKEN");
            self.configured.clone()
        }
        fn get(&self, url: &str) -> Result<String, String> {
            assert!(
                matches!(self.configured, Ok(None)),
                "override must avoid discovery"
            );
            if self.missing {
                return Ok("<html></html>".into());
            }
            match url {
                "https://www.ankernordics.com" => Ok(r#"<script src="/_next/static/chunks/pages/_app-1022739065c72234.js"></script>"#.into()),
                "https://www.ankernordics.com/_next/static/chunks/pages/_app-1022739065c72234.js" => Ok(include_str!("../../tests/fixtures/storefront_token/regional/nordics-app.js").into()),
                _ => panic!("unexpected fetch: {url}"),
            }
        }
    }
    let expected: Value = serde_json::from_str(include_str!(
        "../../tests/fixtures/storefront_token/regional/nordics-expected.json"
    ))
    .unwrap();
    for retailer in [
        RetailerCode::AnkerSe,
        RetailerCode::AnkerNo,
        RetailerCode::AnkerFi,
        RetailerCode::AnkerIs,
        RetailerCode::AnkerDk,
    ] {
        let command = token::command::AcquireStorefrontToken {
            run_id: Uuid::now_v7(),
            retailer_code: retailer,
            url: "https://www.ankernordics.com".into(),
        };
        let acquired = token::handler::Handler(NordicDocuments {
            configured: Ok(None),
            missing: false,
        })
        .handle(command.clone())
        .unwrap()
        .remove(0);
        let config = retailer_data_extractor::initial_fill_config_for(retailer).unwrap();
        let expected = StorefrontToken::new(
            expected[config.token_market.unwrap()]
                .as_str()
                .unwrap()
                .into(),
        )
        .unwrap();
        assert_eq!(acquired.storefront_token, Some(expected.clone()));
        assert_eq!(acquired.retailer_code, retailer);
        assert_eq!(
            catalog::reacts_to::acquired(&acquired)[0]
                .command
                .storefront_token,
            Some(expected)
        );
        let overridden = token::handler::Handler(NordicDocuments {
            configured: Ok(Some(token_value())),
            missing: false,
        })
        .handle(command.clone())
        .unwrap()
        .remove(0);
        assert_eq!(overridden.storefront_token, Some(token_value()));
        let missing = token::handler::Handler(NordicDocuments {
            configured: Ok(None),
            missing: true,
        })
        .handle(command.clone())
        .unwrap_err();
        assert_eq!(missing, "public Storefront token not found for retailer");
        let invalid = token::handler::Handler(NordicDocuments {
            configured: Err("invalid configured token".into()),
            missing: false,
        })
        .handle(command)
        .unwrap_err();
        assert_eq!(invalid, "invalid configured token");
    }
}

#[test]
fn contextual_details_preserve_market_in_every_page_and_artifact() {
    let context = retailer_sauce::shopify::markets::MarketContext {
        country: "CA".into(),
        language: "FR".into(),
        currency: "CAD".into(),
    };
    let id = "gid://shopify/Product/1";
    let mut p = product(id);
    p = serde_json::from_str(&p.to_string().replace("AUD", "CAD")).unwrap();
    p["images"] = connection(vec![], true, Some("next"));
    let mut fetcher = FakeHttp::from(vec![
        json!({"data":{"product":p},"extensions":{"context":{"country":"CA","language":"FR"}}}),
        json!({"data":{"product":{"id":id,"images":connection(vec![],false,None)}},"extensions":{"context":{"country":"CA","language":"FR"}}}),
    ]);
    fetcher.expected_token = None;
    let artifact = details::fetch::fetch_in_context(
        &fetcher,
        RetailerCode::UgreenCa,
        None,
        id,
        Some(&context),
    )
    .unwrap();
    let artifact: TestArtifact = serde_json::from_str(&artifact).unwrap();
    assert_eq!(artifact.schema_version, 2);
    assert_eq!(artifact.context, Some(context.clone()));
    assert_eq!(artifact.pages.len(), 2);
    for page in artifact.pages {
        assert_eq!(page.request.variables["country"], "CA");
        assert_eq!(page.request.variables["language"], "FR");
    }
    for op in fetcher.calls.lock().unwrap().iter() {
        assert_eq!(op.variables["country"], context.country);
        assert_eq!(op.variables["language"], context.language);
    }
}

#[test]
fn public_storefront_settings_resolve_tokens_and_propagate_optional_absence() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/storefront_token/public");
    for code in [
        "minisforumjp",
        "minisforumkr",
        "ugreeneu",
        "ugreenin",
        "ugreennas",
        "ugreennasca",
        "ugreennaseu",
        "ugreennl",
        "ugreenus",
    ] {
        let retailer = RetailerCode::from_str(code).unwrap();
        let config = retailer_data_extractor::initial_fill_config_for(retailer).unwrap();
        let html = std::fs::read_to_string(root.join(format!("{code}.html"))).unwrap();
        let event = token::handler::Handler(Documents(std::collections::HashMap::from([(
            config.storefront_url.into(),
            html,
        )])))
        .handle(token::command::AcquireStorefrontToken {
            run_id: Uuid::now_v7(),
            retailer_code: retailer,
            url: config.storefront_url.into(),
        })
        .unwrap()
        .remove(0);
        assert_eq!(
            event.storefront_token.as_ref().unwrap().as_str(),
            format!("synthetic-{code}")
        );
        assert!(catalog::reacts_to::acquired(&event).is_empty());
    }
    for retailer in [
        RetailerCode::AnkerItalyCom,
        RetailerCode::AnkerJapanCom,
        RetailerCode::MinisForumAu,
        RetailerCode::UgreenCa,
    ] {
        let config = retailer_data_extractor::initial_fill_config_for(retailer).unwrap();
        let event = token::handler::Handler(Documents(std::collections::HashMap::from([(
            config.storefront_url.into(),
            "<script>var settings = {storefrontAccessToken: ''};</script>".into(),
        )])))
        .handle(token::command::AcquireStorefrontToken {
            run_id: Uuid::now_v7(),
            retailer_code: retailer,
            url: config.storefront_url.into(),
        })
        .unwrap()
        .remove(0);
        assert!(event.storefront_token.is_none());
        if !retailer_data_extractor::all_market_contexts(retailer) {
            assert!(
                catalog::reacts_to::acquired(&event)[0]
                    .command
                    .storefront_token
                    .is_none()
            );
        }
    }
}

#[derive(serde::Deserialize)]
struct TestArtifact {
    schema_version: u32,
    context: Option<retailer_sauce::shopify::markets::MarketContext>,
    product: Value,
    pages: Vec<TestPage>,
}
#[derive(serde::Deserialize)]
struct TestPage {
    request: GraphqlRequest,
}
fn fetch_details<F: Http>(
    http: &F,
    retailer: RetailerCode,
    token: Option<&StorefrontToken>,
    id: &str,
) -> Result<TestArtifact, String> {
    serde_json::from_str(&details::fetch::fetch_in_context(
        http, retailer, token, id, None,
    )?)
    .map_err(|e| e.to_string())
}
