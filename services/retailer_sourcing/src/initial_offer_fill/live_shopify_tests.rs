//! Explicit live smoke test: acquire a public token, read catalog batches, store one complete product.
use super::{
    acquire_storefront_token as token, offer_details as details, product_catalog_ids as catalog,
};
use crate::platform::{bus::Handle, http, object_storage::ObjectStorage};
use shared::retailer::RetailerCode;
use uuid::Uuid;

#[test]
#[ignore = "requires live network and OBJECT_STORAGE_URL/TOKEN; run make test-initial-fill-live"]
fn anker_au_token_catalog_and_one_complete_offer() {
    run(RetailerCode::AnkerAu);
}

#[test]
#[ignore = "requires live network and OBJECT_STORAGE_URL/TOKEN; run make test-initial-fill-live"]
fn anker_ca_english_token_catalog_and_one_complete_offer() {
    run(RetailerCode::AnkerCa);
}

#[test]
#[ignore = "requires live network and OBJECT_STORAGE_URL/TOKEN; run make test-initial-fill-live"]
fn remaining_anker_markets_catalog_and_one_complete_offer() {
    for retailer in RetailerCode::ALL {
        if !matches!(retailer, RetailerCode::AnkerAu | RetailerCode::AnkerCa)
            && retailer_data_extractor::initial_fill_config_for(retailer).is_some()
        {
            run(retailer);
        }
    }
}

fn run(retailer: RetailerCode) {
    assert_eq!(
        std::env::var("INITIAL_FILL_LIVE").as_deref(),
        Ok("1"),
        "set INITIAL_FILL_LIVE=1 explicitly"
    );
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let objects = ObjectStorage::from_env(runtime.handle().clone()).unwrap();
    let config = retailer_data_extractor::initial_fill_config_for(retailer).unwrap();
    let command = if config.requires_token() {
        let event = token::handler::Handler(http::Client)
            .handle(token::command::AcquireStorefrontToken {
                run_id: Uuid::now_v7(),
                retailer_code: retailer,
                url: config.storefront_url.into(),
            })
            .unwrap()
            .remove(0);
        catalog::reacts_to::acquired(&event).remove(0).command
    } else {
        catalog::command::FetchProductCatalogIdentifiers {
            context: None,
            run_id: Uuid::now_v7(),
            retailer_code: retailer,
            storefront_token: None,
            after: None,
        }
    };
    let http = http::OfferClient::configured().unwrap();
    let handler = catalog::handler::Handler(http.clone());
    let batch = handler.handle(command).unwrap().remove(0);
    assert!(!batch.product_ids.is_empty());
    assert!(batch.product_ids.len() <= 500);
    if let Some(next) = catalog::reacts_to::next_batch(&batch).pop() {
        let next_batch = handler.handle(next.command).unwrap().remove(0);
        assert_ne!(next_batch.after, next_batch.next_cursor);
    }
    let command = details::reacts_to::on(&batch).remove(0).command;
    let event = details::handler::Handler {
        fetcher: http,
        store: objects.clone(),
    }
    .handle(command)
    .unwrap()
    .remove(0);
    let artifact = objects
        .get_offer(details::handler::BUCKET, &event.storage)
        .unwrap();
    let artifact: serde_json::Value = serde_json::from_str(&artifact).unwrap();
    assert_eq!(artifact["schema_version"], 1);
    assert!(!artifact["pages"].as_array().unwrap().is_empty());
}
