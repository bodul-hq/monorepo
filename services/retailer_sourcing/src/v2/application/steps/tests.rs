use super::{offer_details as details, product_catalog_ids as catalog};
use crate::app;
use crate::platform::bus::stable_command_id;
use crate::platform::{bus::Handle, object_storage::OfferStorageMetadata};
use crate::v2::incoming::InitialFillContexts;
use crate::v2::prelude as token;
use catalog::FetchProductCatalogIdentifiers;
use catalog::Handler as CatalogHandler;
use catalog::ProductCatalogIdsRetrieved;
use catalog::start_catalog_for_locales;
use chrono::DateTime;
use chrono::Utc;
use details::FetchOfferDetails;
use details::Handler as OfferDetailsHandler;
use details::SourceType;
use details::Store;
use details::fetch_details_for_catalog_products;
use details::fetch_in_context;
use retailer_data_extractor::Step;
use retailer_data_extractor::get_storefront_config;
use retailer_data_extractor::initial_fill_process;
use retailer_sauce::prelude::GraphqlRequest;
use retailer_sauce::prelude::Http;
use retailer_sauce::prelude::MarketContext;
use retailer_sauce::prelude::StorefrontToken;
use serde_json::from_slice;
use serde_json::from_str;
use serde_json::from_value;
use serde_json::to_string;
use serde_json::to_value;
use serde_json::{Value, json};
use shared::retailer::RetailerCode;
use std::collections::HashMap;
use std::fs::read_to_string;
use std::path::Path;
use std::{collections::VecDeque, sync::Mutex};
use token::AcquireStorefrontToken;
use token::AcquireStorefrontTokenHandler;
use token::StorefrontTokenFetch;
use uuid::Uuid;

pub(super) fn token_value() -> StorefrontToken {
    StorefrontToken::new("synthetic-au-token".into()).unwrap()
}

#[test]
fn tokenless_retries_skip_discovery_and_catalog_ignores_historical_tokens() {
    struct NoDiscovery;
    impl Http for NoDiscovery {
        fn get(&self, _: &str) -> Result<String, String> {
            panic!("tokenless retries must not download storefronts or scripts");
        }
        fn configured_token(&self, _: &str) -> Result<Option<StorefrontToken>, String> {
            panic!("tokenless retries must not read token overrides");
        }
    }
    for code in [
        "ugreenkr",
        "minisforumhk",
        "ugreenfr",
        "minisforumjp",
        "minisforumkr",
        "ugreeneu",
        "ugreenin",
        "ugreennas",
        "ugreennasca",
        "ugreennaseu",
        "ugreennl",
        "ugreenuk",
        "ugreenus",
    ] {
        let retailer = RetailerCode::from_str(code).unwrap();
        let config = get_storefront_config(retailer).unwrap();
        let event = AcquireStorefrontTokenHandler(NoDiscovery)
            .handle(AcquireStorefrontToken {
                run_id: Uuid::now_v7(),
                retailer_code: retailer,
                url: config.storefront_url().into(),
            })
            .unwrap()
            .remove(0);
        assert!(event.storefront_token.is_none());
        let commands = start_catalog_for_locales(&event);
        assert!(!commands.is_empty());
        for queued in commands {
            assert!(queued.command.storefront_token.is_none());
            let mut command = queued.command;
            let context = command.context.as_ref().unwrap();
            let mut response = catalog_page(1, 1, false, "last");
            response["extensions"] = json!({"context":{
                // A countryless context accepts whichever default the shop reports.
                "country":context.country.as_deref().unwrap_or("US"),
                "language":context.language
            }});
            // Old catalog commands can still carry credentials acquired earlier.
            command.storefront_token = Some(token_value());
            let handler = CatalogHandler(FakeHttp {
                expected_token: None,
                ..FakeHttp::from(vec![response])
            });
            let fetched = handler.handle(command).unwrap().remove(0);
            assert_eq!(fetched.product_ids, ["gid://shopify/Product/1"]);
        }
    }
}

#[test]
fn retailer_payloads_read_legacy_names_write_slugs_and_keep_command_ids() {
    let run_id = Uuid::from_u128(1);
    let old_command =
        json!({"run_id":run_id,"retailer_code":"AnkerAu","url":"https://www.anker.com/au"});
    let command: AcquireStorefrontToken = from_value(old_command).unwrap();
    let encoded = to_value(&command).unwrap();
    assert_eq!(encoded["retailer_code"], "ankerau");
    assert_eq!(
        from_value::<AcquireStorefrontToken>(encoded)
            .unwrap()
            .retailer_code,
        RetailerCode::AnkerAu
    );

    let old_event = json!({
        "run_id":run_id,"retailer_code":"AnkerAu","storefront_token":"synthetic-au-token",
        "after":null,"product_ids":["gid://shopify/Product/1"],"next_cursor":null,
        "retrieved_at":"2026-11-09T00:00:00Z"
    });
    let old_event: ProductCatalogIdsRetrieved = from_value(old_event).unwrap();
    assert_eq!((old_event.product_count, old_event.loop_count), (0, 0));
    let encoded = to_value(&old_event).unwrap();
    assert_eq!(encoded["retailer_code"], "ankerau");
    let new_event: ProductCatalogIdsRetrieved = from_value(encoded).unwrap();
    let old_key = br#"["FetchOfferDetails","AnkerAu","gid://shopify/Product/1"]"#;
    let expected_id = stable_command_id(&run_id, old_key);
    for event in [old_event, new_event] {
        let queued = fetch_details_for_catalog_products(&event).remove(0);
        assert_eq!(queued.id, expected_id);
        assert_eq!(
            to_value(&queued.command).unwrap()["retailer_code"],
            "ankerau"
        );
    }
}
pub(super) fn connection(edges: Vec<Value>, more: bool, cursor: Option<&str>) -> Value {
    json!({"edges":edges,"pageInfo":{"hasNextPage":more,"endCursor":cursor}})
}
pub(super) fn product(id: &str) -> Value {
    let mut value: Value = from_str(include_str!(
        "../../../../tests/fixtures/anker/ankerau_product.json"
    ))
    .unwrap();
    value["id"] = json!(id);
    for field in ["collections", "images", "media", "variants"] {
        value[field] = connection(vec![], false, None);
    }
    value
}
fn sample_node(field: &str) -> Value {
    let value: Value = from_str(include_str!(
        "../../../../tests/fixtures/anker/ankerau_product.json"
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
pub(super) fn catalog_command(run_id: Uuid) -> FetchProductCatalogIdentifiers {
    FetchProductCatalogIdentifiers {
        context: None,
        run_id,
        retailer_code: RetailerCode::AnkerAu,
        storefront_token: Some(token_value()),
    }
}
#[test]
fn catalog_handler_collects_every_batch_into_one_event() {
    let handler = CatalogHandler(FakeHttp::from(vec![
        catalog_page(1, 250, true, "a"),
        catalog_page(251, 250, true, "b"),
        catalog_page(501, 2, false, "c"),
    ]));
    let run_id = Uuid::now_v7();
    let events = handler.handle(catalog_command(run_id)).unwrap();
    assert_eq!(events.len(), 1);
    let event = &events[0];
    assert_eq!(event.product_ids.len(), 502);
    assert_eq!(event.product_count, 502);
    assert_eq!(event.loop_count, 2);
    assert_eq!(event.product_ids[0], "gid://shopify/Product/1");
    assert_eq!(event.product_ids[501], "gid://shopify/Product/502");
    let commands = fetch_details_for_catalog_products(event);
    assert_eq!(commands.len(), 502);
    let again = fetch_details_for_catalog_products(event);
    assert_eq!(commands[0].id, again[0].id);
    assert_eq!(commands[0].run_id, run_id);
    let calls = handler.0.calls.lock().unwrap();
    assert_eq!(calls.len(), 3);
    assert!(calls[2].variables["after"] == "b");
}
#[test]
fn catalog_handler_restarts_legacy_continuation_commands_from_the_beginning() {
    let mut legacy = to_value(catalog_command(Uuid::now_v7())).unwrap();
    legacy["after"] = json!("b");
    let command: FetchProductCatalogIdentifiers = from_value(legacy).unwrap();
    let handler = CatalogHandler(FakeHttp::from(vec![catalog_page(1, 2, false, "c")]));
    let event = handler.handle(command).unwrap().remove(0);
    assert_eq!(event.product_ids.len(), 2);
    let calls = handler.0.calls.lock().unwrap();
    assert!(calls[0].variables.get("after").is_none_or(Value::is_null));
}
#[test]
fn catalog_handler_drops_ids_repeated_across_batches() {
    let handler = CatalogHandler(FakeHttp::from(vec![
        catalog_page(1, 250, true, "a"),
        catalog_page(251, 250, true, "b"),
        catalog_page(500, 2, false, "c"),
    ]));
    let event = handler
        .handle(catalog_command(Uuid::now_v7()))
        .unwrap()
        .remove(0);
    assert_eq!(event.product_ids.len(), 501);
    assert_eq!(event.product_count, 501);
    assert_eq!(event.loop_count, 2);
    assert_eq!(event.product_ids[499], "gid://shopify/Product/500");
    assert_eq!(event.product_ids[500], "gid://shopify/Product/501");
}
#[test]
fn catalog_handler_rejects_cursor_cycles_across_batches() {
    let handler = CatalogHandler(FakeHttp::from(vec![
        catalog_page(1, 250, true, "a"),
        catalog_page(251, 250, true, "b"),
        catalog_page(501, 250, true, "c"),
        catalog_page(751, 250, true, "d"),
        catalog_page(1001, 250, true, "e"),
        catalog_page(1251, 250, true, "b"),
    ]));
    let error = handler.handle(catalog_command(Uuid::now_v7())).unwrap_err();
    assert_eq!(error, "catalog cursor b repeated");
}
#[test]
fn catalog_empty_short_duplicate_and_exact_batches() {
    let empty = CatalogHandler(FakeHttp::from(vec![catalog_page(1, 0, false, "end")]));
    let event = empty
        .handle(catalog_command(Uuid::now_v7()))
        .unwrap()
        .remove(0);
    assert!(event.product_ids.is_empty());
    assert_eq!((event.product_count, event.loop_count), (0, 1));
    assert!(fetch_details_for_catalog_products(&event).is_empty());
    let short = CatalogHandler(FakeHttp::from(vec![
        catalog_page(1, 1, true, "a"),
        catalog_page(1, 2, false, "b"),
    ]));
    assert_eq!(
        short.handle(catalog_command(Uuid::now_v7())).unwrap()[0]
            .product_ids
            .len(),
        2
    );
    let exact = CatalogHandler(FakeHttp::from(vec![
        catalog_page(1, 250, true, "a"),
        catalog_page(251, 250, false, "b"),
    ]));
    let event = exact
        .handle(catalog_command(Uuid::now_v7()))
        .unwrap()
        .remove(0);
    assert_eq!(event.product_ids.len(), 500);
    assert_eq!(exact.0.calls.lock().unwrap().len(), 2);
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
            CatalogHandler(FakeHttp::from(pages))
                .handle(catalog_command(Uuid::now_v7()))
                .is_err()
        );
    }
    let fake = FakeHttp::default();
    fake.responses
        .lock()
        .unwrap()
        .push_back(Err("network failed".into()));
    assert!(
        CatalogHandler(fake)
            .handle(catalog_command(Uuid::now_v7()))
            .is_err()
    );
}

struct Documents(HashMap<String, String>);
impl StorefrontTokenFetch for Documents {
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
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/storefront_token");
    let assets: Vec<Value> = from_str(&read_to_string(root.join("assets.json")).unwrap()).unwrap();
    let mut documents = HashMap::from([(
        "https://www.anker.com/au".to_owned(),
        read_to_string(root.join("ankerau.html")).unwrap(),
    )]);
    for asset in assets {
        documents.insert(
            asset["url"].as_str().unwrap().to_owned(),
            read_to_string(root.join(asset["file"].as_str().unwrap())).unwrap(),
        );
    }
    // Independently read from assets/16.js: storeTokens.au, used by that
    // bundle's X-Http-Storefront-Access-Token header implementation.
    let expected = StorefrontToken::new("ca89549ed256ec3f2c3a12d113b9f29e".into()).unwrap();
    let event = AcquireStorefrontTokenHandler(Documents(documents))
        .handle(AcquireStorefrontToken {
            run_id: Uuid::now_v7(),
            retailer_code: RetailerCode::AnkerAu,
            url: "https://www.anker.com/au".into(),
        })
        .unwrap()
        .remove(0);
    assert_eq!(event.storefront_token, Some(expected.clone()));
    assert_eq!(event.retailer_code, RetailerCode::AnkerAu);
    let commands = start_catalog_for_locales(&event);
    assert_eq!(commands.len(), 1);
    assert_eq!(commands[0].command.storefront_token, event.storefront_token);
}

#[test]
#[ignore = "retailer integration disconnected"]
fn regional_bundle_resolves_imported_tokens_for_fr_pl_nz_vn_eu_and_us() {
    let bundle = include_str!("../../../../tests/fixtures/storefront_token/regional/app.js");
    let expected: Value = from_str(include_str!(
        "../../../../tests/fixtures/storefront_token/regional/expected.json"
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
        let config = get_storefront_config(retailer_code).unwrap();
        // Minimal HTML referencing the exact shared bundle observed on all six homepages.
        let handler = AcquireStorefrontTokenHandler(Documents(
            [
                (
                    config.storefront_url().to_owned(),
                    format!(r#"<script src="{script_url}"></script>"#),
                ),
                (script_url.to_owned(), bundle.to_owned()),
            ]
            .into(),
        ));
        let run_id = Uuid::now_v7();
        let event = handler
            .handle(AcquireStorefrontToken {
                run_id,
                retailer_code,
                url: config.storefront_url().into(),
            })
            .unwrap()
            .remove(0);
        let expected = StorefrontToken::new(
            expected[config.token_market().unwrap()]
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
            start_catalog_for_locales(&event)[0]
                .command
                .storefront_token,
            Some(expected)
        );
    }
}

#[test]
fn token_handler_fetches_assets_and_emits_market_token_and_timestamp() {
    let handler=AcquireStorefrontTokenHandler(Documents([
        ("https://www.anker.com/au".into(),"<script src='/app.js'></script><script src='https://third.party/ignored.js'></script>".into()),
        ("https://www.anker.com/app.js".into(),"window.config={storeTokens:{au:'synthetic-au-token',ca:'other'}}".into()),
    ].into()));
    let start = Utc::now();
    let run_id = Uuid::now_v7();
    let command = AcquireStorefrontToken {
        run_id,
        retailer_code: RetailerCode::AnkerAu,
        url: "https://www.anker.com/au".into(),
    };
    let event = handler.handle(command).unwrap().remove(0);
    assert_eq!(event.storefront_token, Some(token_value()));
    assert!(event.acquired_at >= start);
    assert_eq!(event.run_id, run_id);
    let commands = start_catalog_for_locales(&event);
    assert_eq!(commands.len(), 1);
    assert_eq!(commands[0].command.storefront_token, event.storefront_token);
}
#[test]
fn token_handler_rejects_missing_token_failed_assets_and_arbitrary_urls() {
    for html in [
        "<html></html>",
        "<script src='/missing.js'></script>",
        "<script>storeTokens:{au:''}</script>",
    ] {
        let handler = AcquireStorefrontTokenHandler(Documents(
            [("https://www.anker.com/au".into(), html.into())].into(),
        ));
        let command = AcquireStorefrontToken {
            run_id: Uuid::now_v7(),
            retailer_code: RetailerCode::AnkerAu,
            url: "https://www.anker.com/au".into(),
        };
        assert!(handler.handle(command.clone()).is_err());
        assert!(
            handler
                .handle(AcquireStorefrontToken {
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
impl Store for FakeStore {
    fn store(&self, body: &str, at: DateTime<Utc>) -> Result<OfferStorageMetadata, String> {
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
pub(super) fn detail_command(run_id: Uuid, id: &str) -> FetchOfferDetails {
    FetchOfferDetails {
        context: None,
        run_id,
        retailer_code: RetailerCode::AnkerAu,
        storefront_token: Some(token_value()),
        source: SourceType::GraphQl {
            product_id: id.into(),
        },
    }
}
#[test]
fn detail_handler_preserves_raw_evidence_and_emits_only_after_storage() {
    let id = "gid://shopify/Product/1";
    let raw = json!({"data":{"product":product(id)},"extensions":{"unexpected":"preserved"}});
    let handler = OfferDetailsHandler {
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
    let artifact: Value = from_str(&bodies[0]).unwrap();
    assert_eq!(artifact["pages"][0]["response"], raw);
    assert_eq!(artifact["product"]["id"], id);
    assert_eq!(event.storage.content_size, bodies[0].len() as u64);
    assert_eq!(event.storage.scraped_at, Some(event.fetched_at));
    assert!(!bodies[0].contains("synthetic-au-token"));
    let failing = OfferDetailsHandler {
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
        let handler = OfferDetailsHandler {
            fetcher: FakeHttp::from(vec![raw]),
            store: FakeStore {
                bodies: Mutex::new(vec![]),
                fail: false,
            },
        };
        assert!(handler.handle(detail_command(Uuid::now_v7(), id)).is_err());
        assert!(handler.store.bodies.lock().unwrap().is_empty());
    }
    let handler = OfferDetailsHandler {
        fetcher: FakeHttp::default(),
        store: FakeStore {
            bodies: Mutex::new(vec![]),
            fail: false,
        },
    };
    let mut command = detail_command(Uuid::now_v7(), id);
    command.source = SourceType::Html {
        url: "https://example.test".into(),
    };
    assert!(handler.handle(command).is_err());
    assert!(handler.fetcher.calls.lock().unwrap().is_empty());
}

#[test]
fn every_product_and_variant_connection_is_exhausted() {
    let id = "gid://shopify/Product/1";
    let variant_id = "gid://shopify/ProductVariant/10";
    let mut variant = sample_node("variants");
    variant["id"] = json!(variant_id);
    let fields = [
        "components",
        "groupedBy",
        "quantityPriceBreaks",
        "storeAvailability",
    ];
    for field in fields {
        variant[field] = connection(vec![], true, Some(&format!("{field}-1")));
    }
    let mut initial = product(id);
    let product_fields = ["collections", "images", "media", "variants"];
    for field in product_fields {
        let node = match field {
            "variants" => variant.clone(),
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
    for retailer in [RetailerCode::AnkerAu] {
        let config = get_storefront_config(retailer).unwrap();
        let access = config.requires_token().then(token_value);
        let fake = FakeHttp {
            expected_token: access.clone(),
            ..FakeHttp::from(from_str(&to_string(&responses).unwrap()).unwrap())
        };
        let result = fetch_details(&fake, retailer, access.as_ref(), id).unwrap();
        assert_eq!(result.pages.len(), 9);
        let config = get_storefront_config(retailer).unwrap();
        for page in &result.pages {
            assert!(page.request.variables.get("country").is_none());
            assert_eq!(
                page.request.variables["language"],
                config.default_language()
            );
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
        // Selling plans need a scope acquired storefront tokens usually lack.
        let calls = fake.calls.lock().unwrap();
        assert!(calls.iter().all(|call| !call.query.contains("sellingPlan")));
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
#[ignore = "retailer integration disconnected"]
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
        .filter(|code| get_storefront_config(*code).is_some())
        .map(|code| code.as_str())
        .collect();
    cases.extend(
        supported
            .iter()
            .map(|code| (code.as_str(), StatusCode::SERVICE_UNAVAILABLE)),
    );
    for (retailer, status) in cases {
        let response = app(Default::default())
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
        let response = app(Default::default())
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
    let response = app(Default::default())
        .oneshot(
            Request::get("/api/openapi.json")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let body: Value =
        from_slice(&to_bytes(response.into_body(), usize::MAX).await.unwrap()).unwrap();
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
#[ignore = "retailer integration disconnected"]
fn canada_acquires_its_token_batches_500_and_stores_english_cad_details() {
    let config = get_storefront_config(RetailerCode::AnkerCa).unwrap();
    let script_url = "https://one.anker.com/_next/static/chunks/1b2878kmn7hql.js";
    // Synthetic HTML referencing the actual captured multi-market Anker bundle.
    // This is not a recorded Canadian homepage.
    let bundle = include_str!("../../../../tests/fixtures/storefront_token/assets/16.js");
    assert!(bundle.contains(r#""ca":"anker-canada.myshopify.com""#));
    assert_eq!(
        config.graphql_url(),
        "https://anker-canada.myshopify.com/api/2026-10/graphql.json"
    );
    let handler = AcquireStorefrontTokenHandler(Documents(HashMap::from([
        (
            config.storefront_url().into(),
            format!(r#"<script src="{script_url}"></script>"#),
        ),
        (script_url.into(), bundle.into()),
    ])));
    let run_id = Uuid::now_v7();
    let acquired = handler
        .handle(AcquireStorefrontToken {
            run_id,
            retailer_code: RetailerCode::AnkerCa,
            url: config.storefront_url().into(),
        })
        .unwrap()
        .remove(0);
    // Independently read from storeTokens.ca in the captured bundle.
    assert_eq!(
        acquired.storefront_token.as_ref().unwrap().as_str(),
        "0a32b45f34ceb286e6757f7b8e71b4b5"
    );
    let expected_token = acquired.storefront_token.clone();
    let catalog = CatalogHandler(FakeHttp {
        expected_token: expected_token.clone(),
        ..FakeHttp::from(vec![
            catalog_page(1, 250, true, "a"),
            catalog_page(251, 250, true, "b"),
            catalog_page(501, 1, false, "c"),
        ])
    });
    let batch = catalog
        .handle(start_catalog_for_locales(&acquired).remove(0).command)
        .unwrap()
        .remove(0);
    assert_eq!(batch.retailer_code, RetailerCode::AnkerCa);
    assert_eq!(batch.run_id, run_id);
    assert_eq!(batch.product_ids.len(), 501);
    let commands = fetch_details_for_catalog_products(&batch);
    assert_eq!(commands.len(), 501);
    assert!(
        commands
            .iter()
            .all(|queued| queued.command.retailer_code == RetailerCode::AnkerCa)
    );

    let mut cad_product = product("gid://shopify/Product/1");
    cad_product = from_str(&cad_product.to_string().replace("AUD", "CAD")).unwrap();
    let handler = OfferDetailsHandler {
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
    let artifact: Value = from_str(&bodies[0]).unwrap();
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
#[ignore = "retailer integration disconnected"]
fn canada_rejects_french_token_urls_and_wrong_currency_before_storage() {
    let command = AcquireStorefrontToken {
        run_id: Uuid::now_v7(),
        retailer_code: RetailerCode::AnkerCa,
        url: "https://www.anker.com/ca-fr".into(),
    };
    assert!(
        AcquireStorefrontTokenHandler(Documents(Default::default()))
            .handle(command)
            .unwrap_err()
            .contains("URL does not match")
    );
    let only_french = Documents(HashMap::from([(
        "https://www.anker.com/ca".into(),
        r#"<script>storeTokens:{"ca-fr":"french-only-token"}</script>"#.into(),
    )]));
    let error = AcquireStorefrontTokenHandler(only_french)
        .handle(AcquireStorefrontToken {
            run_id: Uuid::now_v7(),
            retailer_code: RetailerCode::AnkerCa,
            url: "https://www.anker.com/ca".into(),
        })
        .unwrap_err();
    assert!(error.contains("token not found"));
    for currency in ["AUD", "USD", "EUR"] {
        let mut value = product("gid://shopify/Product/1");
        value["priceRange"]["minVariantPrice"]["currencyCode"] = json!(currency);
        let handler = OfferDetailsHandler {
            fetcher: FakeHttp::from(vec![json!({"data":{"product":value}})]),
            store: FakeStore {
                bodies: Mutex::new(vec![]),
                fail: false,
            },
        };
        let error = handler
            .handle(FetchOfferDetails {
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
#[ignore = "retailer integration disconnected"]
fn every_market_preserves_its_token_policy_through_catalog_fanout_and_storage() {
    let mut count = 0;
    for retailer in RetailerCode::ALL {
        let Some(config) = get_storefront_config(retailer) else {
            continue;
        };
        count += 1;
        let run_id = Uuid::now_v7();
        let command = if config.requires_token() {
            let market = config.token_market().unwrap_or("unused");
            let html = format!(
                r#"<script>{{"storeTokens":{{"{market}":"synthetic-au-token","unrelated":"wrong-token"}}}}</script>"#
            );
            let acquired = AcquireStorefrontTokenHandler(Documents(HashMap::from([(
                config.storefront_url().into(),
                html,
            )])))
            .handle(AcquireStorefrontToken {
                run_id,
                retailer_code: retailer,
                url: config.storefront_url().into(),
            })
            .unwrap()
            .remove(0);
            start_catalog_for_locales(&acquired).remove(0).command
        } else {
            let process = initial_fill_process(retailer);
            let mut expected = vec![Step::ProductCatalogIdentifiers, Step::OfferDetails];
            expected.insert(0, Step::AcquireStorefrontToken);
            assert_eq!(process.steps, expected);
            assert!(
                AcquireStorefrontTokenHandler(Documents(Default::default()))
                    .handle(AcquireStorefrontToken {
                        run_id,
                        retailer_code: retailer,
                        url: config.storefront_url().into()
                    })
                    .is_err()
            );
            FetchProductCatalogIdentifiers {
                context: None,
                run_id,
                retailer_code: retailer,
                storefront_token: None,
            }
        };
        let expected_token = command.storefront_token.clone();
        let batch = CatalogHandler(FakeHttp {
            expected_token: expected_token.clone(),
            ..FakeHttp::from(vec![
                catalog_page(1, 250, true, "a"),
                catalog_page(251, 250, false, "b"),
            ])
        })
        .handle(command)
        .unwrap()
        .remove(0);
        let commands = fetch_details_for_catalog_products(&batch);
        assert_eq!(commands.len(), 500);
        assert!(
            commands
                .iter()
                .all(|command| command.command.retailer_code == retailer
                    && command.command.storefront_token == expected_token)
        );
        assert_eq!((batch.product_count, batch.loop_count), (500, 1));
        let value = product("gid://shopify/Product/1");
        let handler = OfferDetailsHandler {
            fetcher: FakeHttp {
                expected_token,
                ..FakeHttp::from(vec![json!({"data":{"product":value}})])
            },
            store: FakeStore {
                bodies: Mutex::new(vec![]),
                fail: false,
            },
        };
        let command = commands.into_iter().next().unwrap().command;
        let expected_country = command.context.as_ref().and_then(|c| c.country.clone());
        let event = handler.handle(command).unwrap().remove(0);
        assert_eq!(event.retailer_code, retailer);
        assert_eq!(event.run_id, run_id);
        let bodies = handler.store.bodies.lock().unwrap();
        let artifact: Value = from_str(&bodies[0]).unwrap();
        assert_eq!(
            artifact["pages"][0]["request"]["variables"]["country"],
            json!(expected_country)
        );
        assert_eq!(
            artifact["pages"][0]["request"]["variables"]["language"],
            config.default_language()
        );
        assert!(!bodies[0].contains("synthetic-au-token"));
    }
    assert_eq!(count, 48);
}

#[test]
#[ignore = "retailer integration disconnected"]
fn nordic_acquisition_discovers_browser_tokens_and_preserves_environment_override() {
    struct NordicDocuments {
        configured: Result<Option<StorefrontToken>, String>,
        missing: bool,
    }
    impl StorefrontTokenFetch for NordicDocuments {
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
                "https://www.ankernordics.com/_next/static/chunks/pages/_app-1022739065c72234.js" => Ok(include_str!("../../../../tests/fixtures/storefront_token/regional/nordics-app.js").into()),
                _ => panic!("unexpected fetch: {url}"),
            }
        }
    }
    let expected: Value = from_str(include_str!(
        "../../../../tests/fixtures/storefront_token/regional/nordics-expected.json"
    ))
    .unwrap();
    for retailer in [
        RetailerCode::AnkerSe,
        RetailerCode::AnkerNo,
        RetailerCode::AnkerFi,
        RetailerCode::AnkerIs,
        RetailerCode::AnkerDk,
    ] {
        let command = AcquireStorefrontToken {
            run_id: Uuid::now_v7(),
            retailer_code: retailer,
            url: "https://www.ankernordics.com".into(),
        };
        let acquired = AcquireStorefrontTokenHandler(NordicDocuments {
            configured: Ok(None),
            missing: false,
        })
        .handle(command.clone())
        .unwrap()
        .remove(0);
        let config = get_storefront_config(retailer).unwrap();
        let expected = StorefrontToken::new(
            expected[config.token_market().unwrap()]
                .as_str()
                .unwrap()
                .into(),
        )
        .unwrap();
        assert_eq!(acquired.storefront_token, Some(expected.clone()));
        assert_eq!(acquired.retailer_code, retailer);
        assert_eq!(
            start_catalog_for_locales(&acquired)[0]
                .command
                .storefront_token,
            Some(expected)
        );
        let overridden = AcquireStorefrontTokenHandler(NordicDocuments {
            configured: Ok(Some(token_value())),
            missing: false,
        })
        .handle(command.clone())
        .unwrap()
        .remove(0);
        assert_eq!(overridden.storefront_token, Some(token_value()));
        let missing = AcquireStorefrontTokenHandler(NordicDocuments {
            configured: Ok(None),
            missing: true,
        })
        .handle(command.clone())
        .unwrap_err();
        assert_eq!(missing, "public Storefront token not found for retailer");
        let invalid = AcquireStorefrontTokenHandler(NordicDocuments {
            configured: Err("invalid configured token".into()),
            missing: false,
        })
        .handle(command)
        .unwrap_err();
        assert_eq!(invalid, "invalid configured token");
    }
}

#[test]
#[ignore = "retailer integration disconnected"]
fn contextual_details_preserve_market_in_every_page_and_artifact() {
    let context = MarketContext {
        country: "CA".into(),
        language: "FR".into(),
        currency: "CAD".into(),
    };
    let id = "gid://shopify/Product/1";
    let mut p = product(id);
    p = from_str(&p.to_string().replace("AUD", "CAD")).unwrap();
    p["images"] = connection(vec![], true, Some("next"));
    let mut fetcher = FakeHttp::from(vec![
        json!({"data":{"product":p},"extensions":{"context":{"country":"CA","language":"FR"}}}),
        json!({"data":{"product":{"id":id,"images":connection(vec![],false,None)}},"extensions":{"context":{"country":"CA","language":"FR"}}}),
    ]);
    fetcher.expected_token = None;
    let artifact =
        fetch_in_context(&fetcher, RetailerCode::UgreenCa, None, id, Some(&context)).unwrap();
    let artifact: TestArtifact = from_str(&artifact).unwrap();
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
fn disabled_storefront_settings_are_ignored_and_optional_absence_propagates() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/storefront_token/public");
    for code in [
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
        let config = get_storefront_config(retailer).unwrap();
        let html = read_to_string(root.join(format!("{code}.html"))).unwrap();
        let event = AcquireStorefrontTokenHandler(Documents(HashMap::from([(
            config.storefront_url().into(),
            html,
        )])))
        .handle(AcquireStorefrontToken {
            run_id: Uuid::now_v7(),
            retailer_code: retailer,
            url: config.storefront_url().into(),
        })
        .unwrap()
        .remove(0);
        assert!(event.storefront_token.is_none());
        let commands = start_catalog_for_locales(&event);
        assert!(!commands.is_empty());
        assert!(
            commands
                .iter()
                .all(|c| c.command.storefront_token == event.storefront_token)
        );
    }
    for retailer in [
        RetailerCode::AnkerItalyCom,
        RetailerCode::AnkerJapanCom,
        RetailerCode::MinisForumAu,
        RetailerCode::UgreenCa,
    ] {
        let config = get_storefront_config(retailer).unwrap();
        let event = AcquireStorefrontTokenHandler(Documents(HashMap::from([(
            config.storefront_url().into(),
            "<script>var settings = {storefrontAccessToken: ''};</script>".into(),
        )])))
        .handle(AcquireStorefrontToken {
            run_id: Uuid::now_v7(),
            retailer_code: retailer,
            url: config.storefront_url().into(),
        })
        .unwrap()
        .remove(0);
        assert!(event.storefront_token.is_none());
        let commands = start_catalog_for_locales(&event);
        assert!(!commands.is_empty());
        assert!(
            commands
                .iter()
                .all(|c| c.command.storefront_token.is_none())
        );
    }
}

#[derive(serde::Deserialize)]
struct TestArtifact {
    schema_version: u32,
    context: Option<MarketContext>,
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
    from_str(&fetch_in_context(http, retailer, token, id, None)?).map_err(|e| e.to_string())
}

#[test]
fn countryless_inventory_context_survives_catalog_batches_and_detail_pagination() {
    let run_id = Uuid::now_v7();
    let retailer = get_storefront_config(RetailerCode::MinisForumEu).unwrap();
    let context = retailer.initial_fill_contexts().unwrap().remove(0);
    assert_eq!(context.language, "EN");
    assert_eq!(context.country, None);
    let with_context = |mut value: Value| {
        value["extensions"] = json!({"context":{"country":"DE","language":"EN"}});
        value
    };
    let handler = CatalogHandler(FakeHttp {
        expected_token: None,
        ..FakeHttp::from(vec![
            with_context(catalog_page(1, 250, true, "a")),
            with_context(catalog_page(251, 250, true, "b")),
            with_context(catalog_page(501, 1, false, "c")),
        ])
    });
    let event = handler
        .handle(FetchProductCatalogIdentifiers {
            context: Some(context),
            run_id,
            retailer_code: RetailerCode::MinisForumEu,
            storefront_token: None,
        })
        .unwrap()
        .remove(0);
    assert_eq!(
        event.context.as_ref().unwrap().observed_country.as_deref(),
        Some("DE")
    );
    assert_eq!((event.product_count, event.loop_count), (501, 2));
    let command = fetch_details_for_catalog_products(&event)
        .pop()
        .unwrap()
        .command;
    assert_eq!(command.context, event.context);
    let id = "gid://shopify/Product/501";
    let mut p = product(id);
    p["images"] = connection(vec![], true, Some("next"));
    let details = OfferDetailsHandler {
        fetcher: FakeHttp {
            expected_token: None,
            ..FakeHttp::from(vec![
                with_context(json!({"data":{"product":p}})),
                with_context(
                    json!({"data":{"product":{"id":id,"images":connection(vec![],false,None)}}}),
                ),
            ])
        },
        store: FakeStore {
            bodies: Mutex::new(vec![]),
            fail: false,
        },
    };
    let result = details.handle(command).unwrap().remove(0);
    let observed = result.context.unwrap();
    assert_eq!(
        (
            observed.country.as_str(),
            observed.language.as_str(),
            observed.currency.as_str()
        ),
        ("DE", "EN", "AUD")
    );
    let artifact: Value = from_str(&details.store.bodies.lock().unwrap()[0]).unwrap();
    assert_eq!(artifact["schema_version"], 2);
    assert_eq!(artifact["context"], to_value(observed).unwrap());
    assert_eq!(artifact["pages"].as_array().unwrap().len(), 2);
    for request in handler
        .0
        .calls
        .lock()
        .unwrap()
        .iter()
        .chain(details.fetcher.calls.lock().unwrap().iter())
    {
        assert!(request.variables.get("country").is_none());
        assert_eq!(request.variables["language"], "EN");
    }
}
