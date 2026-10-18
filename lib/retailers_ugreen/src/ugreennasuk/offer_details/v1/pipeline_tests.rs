use super::{architecture, normalize, structuralize};
use html_extractor::{
    Backend,
    serde_json::{self, Value, json},
};

const ORIGIN: &str = "https://ai-uk.ugreen.com";
const CURRENCY: &str = "GBP";
const PRICE: &str = "£10.99";
const COMPARE: &str = "£150.00";
const HAS_BIS: bool = true;

fn variant(id: u64, price: i64, available: bool) -> Value {
    json!({
        "id": id, "title": format!("Size {id}"), "option1": format!("Size {id}"), "option2": null, "option3": null,
        "sku": format!("sku-{id}"), "requires_shipping": true, "taxable": true,
        "featured_image": {"id": 71, "product_id": 7, "position": 1, "created_at": "2026-01-01T00:00:00Z", "updated_at": "2026-01-01T00:00:00Z", "alt": null, "width": 100, "height": 100, "src": "//cdn.example.com/variant.jpg", "variant_ids": [11, 12]},
        "available": available, "name": format!("NAS - Size {id}"), "public_title": format!("Size {id}"), "options": [format!("Size {id}")],
        "price": price, "weight": 1, "compare_at_price": 15000, "inventory_management": "shopify", "barcode": "6941876247899", "requires_selling_plan": false, "selling_plan_allocations": [], "quantity_rule": {"min": 1, "max": null, "increment": 1}
    })
}

fn glood_product() -> Value {
    json!({
        "id": 7, "title": "NAS", "handle": "test-nas", "vendor": "UGREEN", "type": "NAS",
        "variants": [variant(11, 1099, true), variant(12, 1199, false)], "available": true,
        "price": 1099, "price_min": 1099, "price_max": 1199, "price_varies": true,
        "compare_at_price": 15000, "compare_at_price_min": 15000, "compare_at_price_max": 15000, "compare_at_price_varies": false,
        "featured_image": "//cdn.example.com/gallery.jpg", "images": ["//cdn.example.com/gallery.jpg"], "options": ["Size"],
        "created_at": "2026-01-01T00:00:00Z", "published_at": "2026-01-01T00:00:00Z", "requires_selling_plan": false, "selling_plan_groups": [],
        "description": "", "content": "", "tags": [], "media": []
    })
}

fn put_glood(input: &mut Value, product: &Value) {
    input["glood"]["text"] = json!(format!("window.glood = {{\n  product: {product},\n}};"));
}

fn fixture() -> Value {
    let g = glood_product();
    let variants: Vec<Value> = g["variants"].as_array().unwrap().iter().map(|v| json!({"id": v["id"], "price": v["price"], "name": v["name"], "public_title": v["public_title"], "sku": v["sku"]})).collect();
    let mut input = json!({
        "analytics": {"meta": {"product": {"id": 7, "gid": "gid://shopify/Product/7", "handle": "test-nas", "vendor": "UGREEN", "type": "NAS", "variants": variants, "remote": false}, "page": {"pageType": "product", "resourceType": "product", "resourceId": 7, "requestId": "request"}}},
        "paypal": {"id": "in-context-paypal-metadata", "data-shop-id": "1", "data-venmo-supported": "false", "data-environment": "production", "data-locale": "en_US", "data-paypal-v4": "true", "data-currency": CURRENCY},
        "glood": {"id": "glood-loader", "page-enabled": "true", "data-cmp-ab": "1", "text": ""},
        "product_url": {"content": format!("{ORIGIN}/products/test-nas")},
        "title": {"text": "Displayed NAS"}, "selected_variant_id": {"value": "11"},
        "images": [{"src": "//cdn.example.com/gallery.jpg", "alt": "NAS"}, {"src": "//cdn.example.com/variant.jpg", "alt": null}],
        "variants": {"type": "application/json", "data": g["variants"]},
        "price": {"class": "price price--on-sale"}, "price_regular": {"text": PRICE}, "price_sale": {"text": PRICE}, "price_compare": {"text": COMPARE}
    });
    put_glood(&mut input, &g);
    if HAS_BIS {
        input["em_product_variants"] =
            json!({"id": "em_product_variants", "type": "application/json", "data": g["variants"]});
        input["em_selected_variant"] = json!({"id": "em_product_selected_or_first_available_variant", "type": "application/json", "data": g["variants"][0]});
    }
    input
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
fn rejects(input: &Value, field: &str) {
    let error = normalize(&input.to_string()).err().expect("must fail");
    assert!(error.contains(field), "expected {field}, got {error}");
}

#[test]
fn pipeline_retains_per_variant_facts_and_exact_output_shape() {
    let n = normalized(&fixture());
    assert_eq!(n["product"]["variants"][0]["options"], json!(["Size 11"]));
    let s = structured(&n);
    assert_eq!(
        s,
        json!({"variations": [
            {"identifiers": {"gtin": "6941876247899", "sku": "sku-11", "shopify_product_gid": "gid://shopify/Product/7", "shopify_product_variation_gid": "gid://shopify/ProductVariant/11", "shopify_handle": "test-nas"}, "title": "Displayed NAS", "price": {"amount": 1099, "currency": CURRENCY}, "compare_at_price": {"amount": 15000, "currency": CURRENCY}, "available": true, "url": format!("{ORIGIN}/products/test-nas?variant=11"), "images": ["https://cdn.example.com/variant.jpg", "https://cdn.example.com/gallery.jpg"]},
            {"identifiers": {"gtin": "6941876247899", "sku": "sku-12", "shopify_product_gid": "gid://shopify/Product/7", "shopify_product_variation_gid": "gid://shopify/ProductVariant/12", "shopify_handle": "test-nas"}, "title": "Displayed NAS", "price": {"amount": 1199, "currency": CURRENCY}, "compare_at_price": {"amount": 15000, "currency": CURRENCY}, "available": false, "url": format!("{ORIGIN}/products/test-nas?variant=12"), "images": ["https://cdn.example.com/variant.jpg", "https://cdn.example.com/gallery.jpg"]}
        ]})
    );
}

#[test]
fn raw_schema_rejects_missing_islands_and_unknown_nested_fields() {
    for key in [
        "analytics",
        "glood",
        "paypal",
        "product_url",
        "selected_variant_id",
        "title",
        "images",
    ] {
        let mut input = fixture();
        input.as_object_mut().unwrap().remove(key);
        rejects(&input, key);
    }
    for pointer in [
        "",
        "/paypal",
        "/glood",
        "/analytics/meta/product",
        "/variants/data/0",
        "/variants/data/0/quantity_rule",
    ] {
        let mut input = fixture();
        input.pointer_mut(pointer).unwrap()["unexpected"] = json!(true);
        rejects(&input, "unexpected");
    }
    let mut input = fixture();
    let mut g = glood_product();
    g["variants"][0]["unit_price"] = json!(1);
    put_glood(&mut input, &g);
    rejects(&input, "unit_price");
}

#[test]
fn malformed_missing_and_ambiguous_glood_products_fail() {
    for text in [
        "window.glood = {};",
        "product: {broken},",
        "product: {},\nproduct: {},",
        "product: null,",
    ] {
        let mut input = fixture();
        input["glood"]["text"] = json!(text);
        rejects(&input, "Glood");
    }
    let mut input = fixture();
    input["glood"]["text"] = json!(format!("product: {} garbage", glood_product()));
    rejects(&input, "after Glood");
}

#[test]
fn json_strings_with_braces_and_property_text_are_not_executed_or_split() {
    let mut input = fixture();
    let mut g = glood_product();
    g["description"] =
        json!("closing } and \"quotes\"\nproduct: {fake}\n<script>danger()</script>");
    put_glood(&mut input, &g);
    assert_eq!(normalized(&input)["product"]["title"], "Displayed NAS");
}

#[test]
fn glood_supplies_all_variants_when_the_picker_is_absent() {
    let mut input = fixture();
    input.as_object_mut().unwrap().remove("variants");
    assert_eq!(
        structured(&normalized(&input))["variations"]
            .as_array()
            .unwrap()
            .len(),
        2
    );
}

#[test]
fn records_must_agree_and_variant_sets_must_be_unique_and_complete() {
    for (pointer, value, label) in [
        (
            "/analytics/meta/product/variants/0/price",
            json!(1),
            "price",
        ),
        (
            "/analytics/meta/product/variants/0/sku",
            json!("wrong"),
            "sku",
        ),
        ("/analytics/meta/product/variants/1/id", json!(11), "ID"),
        ("/analytics/meta/product/variants/1/id", json!(99), "99"),
        (
            "/analytics/meta/product/gid",
            json!("gid://shopify/Product/8"),
            "ID",
        ),
        ("/analytics/meta/page/resourceId", json!(8), "page identity"),
        (
            "/selected_variant_id/value",
            json!("99"),
            "selected variant",
        ),
        ("/variants/data/0/available", json!(false), "picker"),
    ] {
        let mut input = fixture();
        *input.pointer_mut(pointer).unwrap() = value;
        rejects(&input, label);
    }
    for key in ["variants", "em_product_variants"] {
        if key.starts_with("em") && !HAS_BIS {
            continue;
        }
        let mut input = fixture();
        input[key]["data"].as_array_mut().unwrap().pop();
        rejects(&input, "count");
    }
    let mut input = fixture();
    let mut g = glood_product();
    g["variants"][1] = g["variants"][0].clone();
    put_glood(&mut input, &g);
    rejects(&input, "Glood variant set");
}

#[test]
fn analytics_order_wins_over_reordered_app_records() {
    let mut input = fixture();
    let mut g = glood_product();
    g["variants"].as_array_mut().unwrap().reverse();
    put_glood(&mut input, &g);
    assert_eq!(
        normalized(&input)["product"]["variants"][0]["id"],
        "gid://shopify/ProductVariant/11"
    );
}

#[test]
fn invalid_currency_missing_prices_and_display_disagreements_fail() {
    for currency in ["USD", "XXX"] {
        let mut input = fixture();
        input["paypal"]["data-currency"] = json!(currency);
        rejects(&input, "currency");
    }
    for value in [
        "",
        "10.999",
        "€1,2,3",
        "€99999999999999999999999999999,99",
        "£99999999999999999999999999999.99",
    ] {
        let mut input = fixture();
        input["price_regular"]["text"] = json!(value);
        rejects(&input, "price_regular");
    }
    let mut input = fixture();
    input.as_object_mut().unwrap().remove("price");
    rejects(&input, "price block");
    let mut input = fixture();
    input["price"]["class"] = json!("price price--on-sale price--sold-out");
    rejects(&input, "availability");
    let mut input = fixture();
    input["price_sale"]["text"] = json!(COMPARE);
    rejects(&input, "price_sale");
}

#[test]
fn grouped_locale_prices_and_compare_at_sentinels_are_exact() {
    let mut input = fixture();
    let mut g = glood_product();
    g["variants"][0]["price"] = json!(109999);
    g["variants"][0]["compare_at_price"] = json!(109999);
    input["analytics"]["meta"]["product"]["variants"][0]["price"] = json!(109999);
    input["variants"]["data"] = g["variants"].clone();
    if HAS_BIS {
        input["em_product_variants"]["data"] = g["variants"].clone();
        input["em_selected_variant"]["data"] = g["variants"][0].clone();
    }
    input["price"]["class"] = json!("price");
    for key in ["price_regular", "price_sale", "price_compare"] {
        input[key]["text"] = json!("£1,099.99");
    }
    put_glood(&mut input, &g);
    let n = normalized(&input);
    assert_eq!(n["product"]["variants"][0]["price"]["amount"], 109999);
    assert!(n["product"]["variants"][0]["compare_at_price"].is_null());
}

#[test]
fn purchase_protection_normalizes_without_a_price_block_and_has_no_offers() {
    let mut input = fixture();
    let mut g = glood_product();
    g["handle"] = json!("worry-free-purchase");
    put_glood(&mut input, &g);
    input["analytics"]["meta"]["product"]["handle"] = json!("worry-free-purchase");
    input["product_url"]["content"] = json!(format!("{ORIGIN}/products/worry-free-purchase"));
    for key in ["price", "price_regular", "price_sale", "price_compare"] {
        input.as_object_mut().unwrap().remove(key);
    }
    let n = normalized(&input);
    assert_eq!(n["product"]["variants"].as_array().unwrap().len(), 2);
    assert_eq!(structured(&n), json!({"variations":[]}));
}

#[test]
fn structuralizer_validates_identifiers_urls_and_single_variant_links() {
    let mut n = normalized(&fixture());
    n["product"]["variants"].as_array_mut().unwrap().truncate(1);
    n["product"]["variants"][0]["sku"] = Value::Null;
    n["product"]["variants"][0]["barcode"] = Value::Null;
    let s = structured(&n);
    assert_eq!(
        s["variations"][0]["url"],
        format!("{ORIGIN}/products/test-nas")
    );
    assert!(s["variations"][0]["identifiers"]["sku"].is_null());
    for (pointer, value) in [
        ("/product/variants/0/sku", json!("")),
        ("/product/variants/0/barcode", json!("")),
        (
            "/product/variants/0/id",
            json!("gid://shopify/ProductVariant/x"),
        ),
        (
            "/product/url",
            json!("https://nas.ugreen.com/products/test-nas"),
        ),
        ("/product/purchase_protection", json!(true)),
    ] {
        let mut bad = n.clone();
        *bad.pointer_mut(pointer).unwrap() = value;
        assert!(structuralize(&bad.to_string()).is_err(), "{pointer}");
    }
}

#[test]
fn regional_urls_reject_cross_store_hosts_and_wrong_handles() {
    for url in [
        "https://nas.ugreen.com/products/test-nas",
        "https://ai-other.ugreen.com/products/test-nas",
        "https://ai-uk.ugreen.com/products/other",
        "https://ai-uk.ugreen.com.evil.test/products/test-nas",
        "https://ai-uk.ugreen.com/products/test-nas?variant=11",
    ] {
        let mut input = fixture();
        input["product_url"]["content"] = json!(url);
        rejects(&input, "URL/handle");
    }
}

#[test]
fn normalizes_and_structuralizes_every_page() {
    let Some(pages) = super::tests::pages() else {
        return;
    };
    let arch = architecture();
    let mut failures = Vec::new();
    for page in pages {
        let html = std::fs::read_to_string(&page).unwrap();
        let result = (|| -> Result<(), String> {
            let extracted = Backend::Scraper
                .extract(&html, &arch)
                .map_err(|e| e.to_string())?
                .json;
            let n = normalize(&extracted.to_string())?
                .document_json
                .ok_or("no normalized document")?;
            let typed: super::normalized::UgreenNasUkNormalizedDocument =
                serde_json::from_str(&n).map_err(|e| e.to_string())?;
            assert_eq!(
                serde_json::from_str::<Value>(&n).unwrap(),
                serde_json::to_value(&typed).unwrap()
            );
            let s = structuralize(&n)?
                .document_json
                .ok_or("no structuralized document")?;
            let output: Value = serde_json::from_str(&s).map_err(|e| e.to_string())?;
            let expected = if typed.product.purchase_protection {
                0
            } else {
                typed.product.variants.len()
            };
            assert_eq!(
                output["variations"].as_array().unwrap().len(),
                expected,
                "{}",
                page.display()
            );
            assert_eq!(
                typed.product.url,
                extracted["product_url"]["content"].as_str().unwrap()
            );
            for (actual, variant) in output["variations"]
                .as_array()
                .unwrap()
                .iter()
                .zip(&typed.product.variants)
            {
                assert_eq!(actual["price"]["amount"], variant.price.amount());
                assert_eq!(actual["available"], variant.available);
            }
            Ok(())
        })();
        if let Err(error) = result {
            failures.push(format!("{}: {error}", page.display()));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
