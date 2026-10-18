use super::{architecture, normalize};
use html_extractor::{
    Backend,
    serde_json::{self, Value, json},
};

fn document() -> Value {
    json!({
        "analytics": {"meta": {
            "product": {"id": 1, "gid":"gid://shopify/Product/1", "vendor":"UGREEN", "type":"Cable", "handle":"cable", "variants":[{"id":2, "price":1200, "name":"Cable", "public_title":null, "sku":"123"}], "remote":false},
            "page": {"pageType":"product", "resourceType":"product", "resourceId":1,"requestId":"request"}
        }},
        "ld-json":[{"type":"application/ld+json", "data":{
            "@context":"http://schema.org/", "@type":"Product", "name":"Cable", "url":"https://nl.ugreen.com/products/cable", "image":"//example.com/a.jpg?width=200", "description":"A cable", "brand":{"@type":"Brand","name":"UGREEN"}, "sku":"123", "gtin":"1234567890123",
            "offers":{"@type":"Offer","url":"https://nl.ugreen.com/products/cable?variant=2", "price":"12.00", "priceCurrency":"EUR", "availability":"http://schema.org/InStock"}
        }}],
        "title":{"text":"Cable"}, "price":{"product_id":"1","base_price_cents":"1200","base_compare_cents":"1500","currency":"EUR"},
        "product_form":{"product_id":"1","product_url":"/products/cable"}, "selected_variant_id":{"value":"2"}, "add_to_cart":{}, "option_names":[], "images":[{"src":"//example.com/a.jpg?width=200"}]
    })
}
fn normalized(input: Value) -> Value {
    let output = normalize(&input.to_string())
        .expect("normalizes")
        .document_json
        .expect("document");
    serde_json::from_str(&output).unwrap()
}
#[test]
fn a_product_keeps_rich_offer_facts_and_round_trips() {
    let product = normalized(document());
    assert_eq!(product["product"]["variants"][0]["available"], true);
    assert_eq!(product["product"]["variants"][0]["gtin"], "1234567890123");
    assert_eq!(
        product["product"]["variants"][0]["url"],
        "https://nl.ugreen.com/products/cable?variant=2"
    );
    assert_eq!(
        product["product"]["images"],
        json!(["https://example.com/a.jpg"])
    );
    assert_eq!(product["product"]["description"], "A cable");
    let typed: super::normalized::UgreenNlNormalizedDocument =
        serde_json::from_value(product.clone()).unwrap();
    assert_eq!(serde_json::to_value(typed).unwrap(), product);
}
#[test]
fn malformed_or_conflicting_islands_fail() {
    for (pointer, value) in [
        ("/analytics/meta/product/id", json!(999)),
        ("/ld-json/0/data/offers/priceCurrency", json!("CAD")),
        ("/ld-json/0/data/offers/price", json!("12.001")),
        ("/ld-json/0/data/offers/price", json!("13.00")),
        (
            "/ld-json/0/data/offers/availability",
            json!("http://schema.org/PreOrder"),
        ),
        (
            "/ld-json/0/data/offers/url",
            json!("https://nl.ugreen.com/products/other?variant=2"),
        ),
        ("/ld-json/0/data/sku", json!("wrong")),
        ("/selected_variant_id/value", json!("999")),
    ] {
        let mut d = document();
        *d.pointer_mut(pointer).unwrap() = value;
        assert!(normalize(&d.to_string()).is_err(), "{pointer}");
    }
    let mut d = document();
    d["unexpected"] = json!(true);
    assert!(normalize(&d.to_string()).is_err());
    let mut d = document();
    d["ld-json"][0]["data"]["unexpected"] = json!(true);
    assert!(normalize(&d.to_string()).is_err());
    let mut d = document();
    d.as_object_mut().unwrap().remove("analytics");
    assert!(normalize(&d.to_string()).is_err());
}
#[test]
fn a_bare_offer_url_is_only_unambiguous_for_one_variant() {
    let mut d = document();
    d["ld-json"][0]["data"]["offers"]["url"] = json!("https://nl.ugreen.com/products/cable");
    normalized(d.clone());
    let mut second = d["analytics"]["meta"]["product"]["variants"][0].clone();
    second["id"] = json!(3);
    d["analytics"]["meta"]["product"]["variants"]
        .as_array_mut()
        .unwrap()
        .push(second);
    assert!(normalize(&d.to_string()).is_err());
}
#[test]
fn missing_identifiers_and_both_stock_spellings_are_supported() {
    for scheme in ["http", "https"] {
        let mut d = document();
        d["analytics"]["meta"]["product"]["variants"][0]["sku"] = Value::Null;
        d["ld-json"][0]["data"]
            .as_object_mut()
            .unwrap()
            .remove("sku");
        d["ld-json"][0]["data"]
            .as_object_mut()
            .unwrap()
            .remove("gtin");
        d["ld-json"][0]["data"]["offers"]["availability"] =
            json!(format!("{scheme}://schema.org/OutOfStock"));
        d["add_to_cart"]["disabled"] = json!("");
        let result = normalized(d);
        assert_eq!(result["product"]["variants"][0]["available"], false);
        assert!(result["product"]["variants"][0]["sku"].is_null());
        assert!(result["product"]["variants"][0]["gtin"].is_null());
    }
}
#[test]
fn a_product_group_matches_offers_by_id_not_position() {
    let mut d = document();
    let p = d["ld-json"][0]["data"].clone();
    let mut one = json!({"@type":"Product","@id":"/products/cable?variant=2#variant","name":"Cable","image":"https://example.com/a.jpg","sku":"123","offers":p["offers"]});
    let mut two = one.clone();
    two["@id"] = json!("/products/cable?variant=3#variant");
    two["offers"]["url"] = json!("https://nl.ugreen.com/products/cable?variant=3");
    one["gtin"] = json!("1234567890123");
    d["ld-json"][0]["data"] = json!({"@type":"ProductGroup","@context":p["@context"],"@id":"/products/cable#product","name":p["name"],"url":p["url"],"description":p["description"],"brand":p["brand"],"category":"Cable","productGroupID":"1","hasVariant":[two,one]});
    let mut second = d["analytics"]["meta"]["product"]["variants"][0].clone();
    second["id"] = json!(3);
    d["analytics"]["meta"]["product"]["variants"]
        .as_array_mut()
        .unwrap()
        .push(second);
    let result = normalized(d.clone());
    assert_eq!(
        result["product"]["variants"][0]["id"],
        "gid://shopify/ProductVariant/2"
    );
    d["ld-json"][0]["data"]["hasVariant"]
        .as_array_mut()
        .unwrap()
        .remove(0);
    assert!(normalize(&d.to_string()).is_err());
}
#[test]
fn extraction_preserves_json_attributes_and_ignores_svg() {
    let html = r#"<html><body><svg><title>ignore</title></svg><script type="application/ld+json" class="example">{"@type":"Organization","name":"UGREEN","url":"https://example.com","logo":"logo"}</script><link href="https://monorail-edge.shopifysvc.com" rel="dns-prefetch"><script></script><script></script><script></script><script>var meta = {"product":{"id":1}};</script><script class="analytics"></script></body></html>"#;
    for backend in Backend::ALL {
        let json = backend.extract(html, &architecture()).unwrap().json;
        assert_eq!(json["ld-json"][0]["class"], "example");
        assert_eq!(json["analytics"]["meta"]["product"]["id"], 1);
    }
}
fn pages() -> Option<Vec<std::path::PathBuf>> {
    let dir = std::path::Path::new(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../../data/ugreennl/offer_details/sources"
    ));
    if !dir.is_dir() {
        eprintln!("skipping: {} not found", dir.display());
        return None;
    }
    let mut pages: Vec<_> = std::fs::read_dir(dir)
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().is_some_and(|e| e == "html"))
        .collect();
    pages.sort();
    assert!(!pages.is_empty(), "no HTML pages in {}", dir.display());
    Some(pages)
}
#[test]
fn saved_pages_normalize() {
    let Some(pages) = pages() else { return };
    let arch = architecture();
    let mut failures = Vec::new();
    for page in &pages {
        let html = std::fs::read_to_string(page).unwrap();
        let result = Backend::Scraper.extract(&html, &arch).unwrap().json;
        match normalize(&result.to_string()) {
            Ok(doc) => {
                let text = doc.document_json.unwrap();
                let typed: super::normalized::UgreenNlNormalizedDocument =
                    serde_json::from_str(&text).unwrap();
                assert_eq!(
                    serde_json::to_value(typed).unwrap(),
                    serde_json::from_str::<Value>(&text).unwrap()
                );
            }
            Err(e) => failures.push(format!("{}: {e}", page.display())),
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    eprintln!("ugreennl: {} saved pages normalized", pages.len());
}
#[test]
fn saved_pages_backends_agree() {
    let Some(pages) = pages() else { return };
    let arch = architecture();
    for page in &pages {
        let html = std::fs::read_to_string(page).unwrap();
        let expected = Backend::Scraper.extract(&html, &arch).unwrap().json;
        for backend in Backend::ALL {
            if *backend == Backend::Scraper {
                continue;
            }
            let actual = backend.extract(&html, &arch).unwrap().json;
            assert_eq!(actual, expected, "{} {backend:?}", page.display());
        }
    }
    eprintln!(
        "ugreennl: all backends agree on {} saved pages",
        pages.len()
    );
}

#[test]
fn automatic_discount_is_separate_from_the_variant_base_price() {
    let mut d = document();
    d["price"]["base_price_cents"] = json!("1000");
    d["price"]["base_compare_cents"] = json!("1200");
    let result = normalized(d);
    assert_eq!(
        result["product"]["variants"][0]["price"],
        serde_json::to_value(money::Money::new(1200, "EUR".parse().unwrap())).unwrap()
    );
    assert_eq!(
        result["product"]["selected_variant"]["automatic_discount_price"],
        serde_json::to_value(money::Money::new(1000, "EUR".parse().unwrap())).unwrap()
    );
    assert!(result["product"]["selected_variant"]["compare_at_price"].is_null());
}

#[test]
fn duplicate_offers_and_extra_script_attributes_fail() {
    let mut d = document();
    let offer = d["ld-json"][0]["data"]["offers"].clone();
    d["ld-json"][0]["data"]["offers"] = json!([offer.clone(), offer]);
    assert!(normalize(&d.to_string()).is_err());
    let mut d = document();
    d["ld-json"][0]["unexpected"] = json!("attribute");
    assert!(normalize(&d.to_string()).is_err());
}

#[test]
fn a_group_with_one_variant_is_still_a_selling_product() {
    let mut d = document();
    let p = d["ld-json"][0]["data"].clone();
    d["ld-json"][0]["data"] = json!({"@type":"ProductGroup","@context":p["@context"],"@id":"/products/cable#product","name":p["name"],"url":p["url"],"description":p["description"],"brand":p["brand"],"category":"Cable","productGroupID":"1","hasVariant":[{"@type":"Product","@id":"/products/cable?variant=2#variant","name":"Cable","image":"https://example.com/a.jpg","offers":p["offers"]}]});
    assert_eq!(
        normalized(d)["product"]["variants"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
}

#[test]
fn price_specifications_keep_their_labels_without_overriding_the_price() {
    let mut d = document();
    d["ld-json"][0]["data"]["offers"]["priceSpecification"] = json!([
        {"@type":"UnitPriceSpecification","priceType":"https://schema.org/SalePrice","price":12,"priceCurrency":"EUR"},
        {"@type":"UnitPriceSpecification","priceType":"https://schema.org/ListPrice","price":10,"priceCurrency":"EUR"}
    ]);
    let result = normalized(d);
    let prices = &result["product"]["variants"][0]["offer_prices"];
    assert_eq!(prices[0]["kind"], "https://schema.org/SalePrice");
    assert_eq!(prices[1]["kind"], "https://schema.org/ListPrice");
    assert_eq!(
        prices[0]["price"],
        result["product"]["variants"][0]["price"]
    );
}

#[test]
fn extraction_scopes_the_main_product_and_excludes_duplicate_media() {
    let fixture = document();
    let markup = r#"<div id="shopify-section-template--1__ug_main_product_main"><h1>Cable</h1><product-price data-product-id="1" data-base-price-cents="1200" data-base-compare-cents="1500" data-currency="EUR"></product-price><product-form-component data-section-id="main-sticky-bar" data-product-id="999" data-product-url="/products/wrong"><input name="id" value="999"><button name="add" disabled></button></product-form-component><product-form-component data-section-id="main" data-product-id="1" data-product-url="/products/cable"><input name="id" value="2"><button name="add"></button></product-form-component><media-gallery><slideshow-slide class="product-media-container--image"><img src="//example.com/a.jpg?width=200"></slideshow-slide><slideshow-slide class="product-media-container--video"><img src="wrong"></slideshow-slide></media-gallery></div>"#;
    let html = format!(
        r#"<html><head><script type="application/ld+json">{}</script><link href="https://monorail-edge.shopifysvc.com"><script></script><script></script><script></script><script>var meta = {};</script><script class="analytics"></script></head><body><h1>Recommendation</h1><svg><title>Icon</title></svg>{markup}</body></html>"#,
        fixture["ld-json"][0]["data"], fixture["analytics"]["meta"]
    );
    for backend in Backend::ALL {
        let extracted = backend.extract(&html, &architecture()).unwrap().json;
        assert_eq!(extracted, fixture, "{backend:?}");
        normalized(extracted);
    }
}

#[test]
fn a_published_offer_can_state_the_selected_automatic_discount() {
    let mut d = document();
    d["price"]["base_price_cents"] = json!("1000");
    d["price"]["base_compare_cents"] = json!("1200");
    d["ld-json"][0]["data"]["offers"]["price"] = json!("10.00");
    let result = normalized(d);
    assert_eq!(
        result["product"]["variants"][0]["offer_price"],
        result["product"]["selected_variant"]["automatic_discount_price"]
    );
}

#[test]
fn an_automatic_discount_can_stack_with_an_existing_markdown() {
    let mut d = document();
    d["price"]["base_price_cents"] = json!("1000");
    let result = normalized(d);
    let selected = &result["product"]["selected_variant"];
    assert!(selected["compare_at_price"].is_object());
    assert!(selected["automatic_discount_price"].is_object());
}

#[test]
fn picker_details_are_checked_against_the_selected_offer() {
    let mut d = document();
    d["variant_picker"] = json!({"product_id":"1","product_url":"/products/cable"});
    d["option_names"] = json!([{"text":"Length:"}]);
    d["selected_variant"] = json!({"type":"application/json","data":{
        "id":2,"title":"1M","option1":"1M","option2":null,"option3":null,"sku":"123",
        "requires_shipping":true,"taxable":true,"featured_image":null,"available":true,
        "name":"Cable - 1M","public_title":"1M","options":["1M"],"price":1200,"weight":0,
        "compare_at_price":1500,"inventory_management":"shopify","barcode":"1234567890123",
        "featured_media":null,"requires_selling_plan":false,"selling_plan_allocations":[],
        "quantity_rule":{"min":1,"max":null,"increment":1}
    }});
    let result = normalized(d.clone());
    assert_eq!(
        result["product"]["selected_variant"]["options"],
        json!([{"name":"Length","value":"1M"}])
    );
    d["selected_variant"]["data"]["available"] = json!(false);
    assert!(normalize(&d.to_string()).is_err());
    d["selected_variant"]["data"]["available"] = json!(true);
    d["selected_variant"]["data"]["barcode"] = json!("different");
    assert!(normalize(&d.to_string()).is_err());
}

#[test]
fn all_hundred_purchase_protection_variants_are_kept() {
    let mut d = document();
    let p = d["ld-json"][0]["data"].clone();
    let template = d["analytics"]["meta"]["product"]["variants"][0].clone();
    let mut analytics = Vec::new();
    let mut offers = Vec::new();
    for id in 2..102 {
        let mut variant = template.clone();
        variant["id"] = json!(id);
        analytics.push(variant);
        let mut offer = p["offers"].clone();
        offer["url"] = json!(format!("https://nl.ugreen.com/products/cable?variant={id}"));
        offers.push(json!({"@type":"Product","@id":format!("/products/cable?variant={id}#variant"),"name":"Protection","image":"https://example.com/a.jpg","offers":offer}));
    }
    d["analytics"]["meta"]["product"]["variants"] = json!(analytics);
    d["ld-json"][0]["data"] = json!({"@type":"ProductGroup","@context":p["@context"],"@id":"/products/cable#product","name":p["name"],"url":p["url"],"description":p["description"],"brand":p["brand"],"category":"Protection","productGroupID":"1","hasVariant":offers});
    assert_eq!(
        normalized(d)["product"]["variants"]
            .as_array()
            .unwrap()
            .len(),
        100
    );
}
