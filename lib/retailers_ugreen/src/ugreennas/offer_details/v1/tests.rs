use super::{architecture, normalize, structuralize};
use html_extractor::{
    Backend,
    serde_json::{self, Value, json},
};
use std::path::{Path, PathBuf};

fn fixture() -> Value {
    Backend::Scraper
        .extract(include_str!("fixture.html"), &architecture())
        .unwrap()
        .json
}

fn normalized(input: &Value) -> Value {
    serde_json::from_str(
        &normalize(&input.to_string())
            .unwrap()
            .document_json
            .unwrap(),
    )
    .unwrap()
}

fn structured(input: &Value) -> Value {
    serde_json::from_str(
        &structuralize(&input.to_string())
            .unwrap()
            .document_json
            .unwrap(),
    )
    .unwrap()
}

#[test]
fn a_real_variant_preserves_cents_identifiers_and_images() {
    let input = fixture();
    assert_eq!(input["images"].as_array().unwrap().len(), 1);
    let n = normalized(&input);
    assert_eq!(
        n["product"]["variants"][0]["price"],
        json!({"amount":234999,"currency":"USD"})
    );
    let s = structured(&n);
    assert_eq!(s["variations"].as_array().unwrap().len(), 1);
    let v = &s["variations"][0];
    assert_eq!(
        v["identifiers"],
        json!({
            "gtin":"6941876248353", "sku":"45835",
            "shopify_product_gid":"gid://shopify/Product/8801294188738",
            "shopify_product_variation_gid":"gid://shopify/ProductVariant/48727400317122",
            "shopify_handle":"ugreen-idx6011-32gb-ai-nas"
        })
    );
    assert_eq!(
        v["url"],
        "https://ai.ugreen.com/products/ugreen-idx6011-32gb-ai-nas"
    );
    assert_eq!(v["available"], true);
    assert!(v["images"][0].as_str().unwrap().starts_with("https://"));
}

#[test]
fn raw_schema_rejects_unknown_fields_and_missing_islands() {
    for field in [
        "analytics",
        "shop",
        "variants",
        "selected_variant",
        "title",
        "canonical",
        "images",
    ] {
        let mut input = fixture();
        input.as_object_mut().unwrap().remove(field);
        assert!(normalize(&input.to_string()).is_err(), "{field}");
    }
    let mut input = fixture();
    input["variants"]["data"][0]["unexpected"] = json!(true);
    assert!(normalize(&input.to_string()).is_err());
    let mut input = fixture();
    input["variants"]["extra-attribute"] = json!(true);
    assert!(normalize(&input.to_string()).is_err());
}

#[test]
fn normalization_rejects_disagreements_and_invalid_values() {
    for (pointer, value) in [
        ("/analytics/meta/product/variants/0/price", json!(1)),
        ("/analytics/meta/page/resourceId", json!(1)),
        (
            "/analytics/meta/product/gid",
            json!("gid://shopify/Product/1"),
        ),
        ("/selected_variant/data/available", json!(false)),
        ("/shop/data/currencyCode", json!("INVALID")),
        (
            "/canonical/href",
            json!("https://ai.ugreen.com/products/another-product"),
        ),
        ("/title/data", json!("another title")),
    ] {
        let mut input = fixture();
        *input.pointer_mut(pointer).unwrap() = value;
        assert!(normalize(&input.to_string()).is_err(), "{pointer}");
    }
    let mut input = fixture();
    let duplicate = input["variants"]["data"][0].clone();
    input["variants"]["data"]
        .as_array_mut()
        .unwrap()
        .push(duplicate);
    assert!(normalize(&input.to_string()).is_err());
}

#[test]
fn variants_keep_their_prices_urls_and_missing_identifiers() {
    let mut input = normalized(&fixture());
    let mut second = input["product"]["variants"][0].clone();
    second["id"] = json!("gid://shopify/ProductVariant/123");
    second["price"]["amount"] = json!(3000);
    second["sku"] = Value::Null;
    second["barcode"] = Value::Null;
    second["available"] = json!(false);
    input["product"]["variants"]
        .as_array_mut()
        .unwrap()
        .push(second);
    let s = structured(&input);
    assert_eq!(s["variations"][1]["price"]["amount"], 3000);
    assert_eq!(s["variations"][1]["available"], false);
    assert!(s["variations"][1]["identifiers"]["sku"].is_null());
    assert!(s["variations"][1]["identifiers"]["gtin"].is_null());
    assert!(
        s["variations"][0]["url"]
            .as_str()
            .unwrap()
            .ends_with("?variant=48727400317122")
    );
    assert!(
        s["variations"][1]["url"]
            .as_str()
            .unwrap()
            .ends_with("?variant=123")
    );
}

#[test]
fn structuralization_validates_identifiers() {
    for (field, value) in [
        ("sku", ""),
        ("barcode", ""),
        ("id", "gid://shopify/ProductVariant/bad"),
    ] {
        let mut input = normalized(&fixture());
        input["product"]["variants"][0][field] = json!(value);
        assert!(structuralize(&input.to_string()).is_err(), "{field}");
    }
}

const DATA_DIR: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../../data/ugreennas/offer_details/sources"
);
fn pages() -> Option<Vec<PathBuf>> {
    if !Path::new(DATA_DIR).is_dir() {
        eprintln!("skipping: {DATA_DIR} not found");
        return None;
    }
    let mut pages: Vec<_> = std::fs::read_dir(DATA_DIR)
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().is_some_and(|e| e == "html"))
        .collect();
    pages.sort();
    assert!(!pages.is_empty());
    Some(pages)
}

#[test]
fn extracts_required_islands_from_every_page() {
    let Some(pages) = pages() else { return };
    for page in pages {
        let html = std::fs::read_to_string(&page).unwrap();
        let data = Backend::Scraper
            .extract(&html, &architecture())
            .unwrap()
            .json;
        assert_eq!(
            data["shop"]["data"]["currencyCode"],
            "USD",
            "{}",
            page.display()
        );
        assert!(!data["variants"]["data"].as_array().unwrap().is_empty());
        assert!(!data["images"].as_array().unwrap().is_empty());
        assert!(data["title"]["data"].is_string());
    }
}

#[test]
fn backends_agree_on_every_page() {
    let Some(pages) = pages() else { return };
    for page in pages {
        let html = std::fs::read_to_string(&page).unwrap();
        let expected = Backend::Scraper
            .extract(&html, &architecture())
            .unwrap()
            .json;
        for backend in Backend::ALL {
            let actual = backend.extract(&html, &architecture()).unwrap().json;
            assert_eq!(actual, expected, "{} {backend:?}", page.display());
        }
    }
}

#[test]
fn normalizes_and_structuralizes_every_page() {
    let Some(pages) = pages() else { return };
    let mut failures = Vec::new();
    for page in pages {
        let html = std::fs::read_to_string(&page).unwrap();
        let extracted = Backend::Scraper
            .extract(&html, &architecture())
            .unwrap()
            .json;
        let result = normalize(&extracted.to_string())
            .and_then(|n| structuralize(&n.document_json.unwrap()));
        match result {
            Err(e) => failures.push(format!("{}: {e}", page.display())),
            Ok(s) => {
                let s: Value = serde_json::from_str(&s.document_json.unwrap()).unwrap();
                assert_eq!(
                    s["variations"].as_array().unwrap().len(),
                    extracted["variants"]["data"].as_array().unwrap().len()
                );
            }
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn refurbished_offers_keep_their_own_handle_despite_the_canonical() {
    let mut input = fixture();
    input["analytics"]["meta"]["product"]["handle"] = json!("ugreen-nasync-dxp2800-refurbished");
    input["canonical"]["href"] =
        json!("https://ai.ugreen.com/products/ugreen-nasync-dxp2800-nas-storage");
    let n = normalized(&input);
    assert_eq!(n["product"]["canonical_url"], input["canonical"]["href"]);
    assert_eq!(
        structured(&n)["variations"][0]["url"],
        "https://ai.ugreen.com/products/ugreen-nasync-dxp2800-refurbished"
    );
}

#[test]
fn zero_prices_and_deposits_are_not_replaced_with_compare_at_prices() {
    for price in [0, 3000] {
        let mut input = fixture();
        input["analytics"]["meta"]["product"]["variants"][0]["price"] = json!(price);
        input["variants"]["data"][0]["price"] = json!(price);
        input["variants"]["data"][0]["compare_at_price"] = json!(4000);
        input["selected_variant"]["data"] = input["variants"]["data"][0].clone();
        let s = structured(&normalized(&input));
        assert_eq!(s["variations"][0]["price"]["amount"], price);
        assert_eq!(s["variations"][0]["compare_at_price"]["amount"], 4000);
    }
}
