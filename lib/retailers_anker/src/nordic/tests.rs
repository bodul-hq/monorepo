use super::*;

const CASES: [(Market, &str, i64); 5] = [
    (
        Market::Sweden,
        include_str!("../ankerse/offer_details/v1/graphql/response.json"),
        49900,
    ),
    (
        Market::Norway,
        include_str!("../ankerno/offer_details/v1/graphql/response.json"),
        49900,
    ),
    (
        Market::Finland,
        include_str!("../ankerfi/offer_details/v1/graphql/response.json"),
        4499,
    ),
    (
        Market::Iceland,
        include_str!("../ankeris/offer_details/v1/graphql/response.json"),
        6490,
    ),
    (
        Market::Denmark,
        include_str!("../ankerdk/offer_details/v1/graphql/response.json"),
        34900,
    ),
];
const URL: &str = "https://www.ankernordics.com/products/t90r0121";

#[test]
fn variants_retain_independent_prices_comparisons_and_availability() {
    let mut value: serde_json::Value = serde_json::from_str(CASES[2].1).unwrap();
    let variants = value["data"]["product"]["variants"]["nodes"]
        .as_array_mut()
        .unwrap();
    variants[0]["compareAtPrice"] = json!({"amount":"54.99","currencyCode":"EUR"});
    let mut second = variants[0].clone();
    second["id"] = json!("gid://shopify/ProductVariant/1234");
    second["price"]["amount"] = json!("59.99");
    second["availableForSale"] = json!(false);
    second["sku"] = json!("");
    variants.push(second);
    let normalized = Market::Finland
        .normalize(&value.to_string())
        .unwrap()
        .document_json
        .unwrap();
    let result = Market::Finland
        .structuralize(&normalized)
        .unwrap()
        .document_json
        .unwrap();
    let result: serde_json::Value = serde_json::from_str(&result).unwrap();
    let variants = result["variations"].as_array().unwrap();
    assert_eq!(variants.len(), 2);
    assert_eq!(variants[0]["was_price"]["amount"], 5499);
    assert_eq!(variants[1]["price"]["amount"], 5999);
    assert!(variants[1]["was_price"].is_null());
    assert!(variants[1]["identifiers"]["sku"].is_null());
    assert_ne!(variants[0]["availability"], variants[1]["availability"]);
    assert!(
        variants[1]["url"]
            .as_str()
            .unwrap()
            .ends_with("?variant=1234")
    );
}

#[test]
fn real_market_responses_preserve_prices_identifiers_and_localization() {
    for (market, body, amount) in CASES {
        market.validate_download(URL, body).unwrap();
        let request: serde_json::Value =
            serde_json::from_str(&market.request(URL).unwrap()).unwrap();
        assert_eq!(request["variables"]["country"], market.country());
        assert_eq!(request["variables"]["handle"], "t90r0121");
        let extracted = market.extract(body).unwrap();
        let normalized = market.normalize(&extracted).unwrap().document_json.unwrap();
        let structuralized = market
            .structuralize(&normalized)
            .unwrap()
            .document_json
            .unwrap();
        let value: serde_json::Value = serde_json::from_str(&structuralized).unwrap();
        let variant = &value["variations"][0];
        assert_eq!(
            variant["price"],
            json!({"amount":amount,"currency":market.currency().as_str()})
        );
        assert_eq!(
            variant["localization"],
            json!({"target_market":{"kind":"country","code":market.country()},"language":"en"})
        );
        assert_eq!(
            variant["identifiers"]["shopify_product_gid"],
            "gid://shopify/Product/15441693442326"
        );
        assert_eq!(variant["identifiers"]["sku"], "T90R0321");
        assert_eq!(variant["url"], format!("{URL}?variant=67788280791318"));
        assert!(
            variant["content"]["description"]
                .as_str()
                .unwrap()
                .contains("110 dB")
        );
        assert_eq!(variant["images"].as_array().unwrap().len(), 1);
        for (other, _, _) in CASES {
            if other != market {
                assert!(other.normalize(body).is_err());
                assert!(other.structuralize(&normalized).is_err());
            }
        }
    }
}

#[test]
fn incomplete_wrong_product_and_error_responses_fail_before_storage() {
    let base: serde_json::Value = serde_json::from_str(CASES[0].1).unwrap();
    for (pointer, invalid) in [
        ("/data/localization/country/isoCode", json!("NO")),
        ("/data/localization/language/isoCode", json!("SV")),
        ("/data/product", serde_json::Value::Null),
        ("/data/product/handle", json!("another-product")),
        ("/data/product/images/pageInfo/hasNextPage", json!(true)),
        ("/data/product/variants/pageInfo/hasNextPage", json!(true)),
        (
            "/data/product/variants/nodes/0/price/currencyCode",
            json!("NOK"),
        ),
        ("/data/product/variants/nodes/0/price/amount", json!("NaN")),
        (
            "/data/product/variants/nodes/0/price/amount",
            json!("0.001"),
        ),
        ("/data/product/variants/nodes/0/id", json!("invalid")),
        (
            "/data/product/onlineStoreUrl",
            json!("https://other.test/products/t90r0121"),
        ),
    ] {
        let mut value = base.clone();
        *value.pointer_mut(pointer).unwrap() = invalid;
        assert!(
            Market::Sweden
                .validate_download(URL, &value.to_string())
                .is_err(),
            "{pointer}"
        );
    }
    let mut errors = base.clone();
    errors["errors"] = json!([{"message":"partial response"}]);
    assert!(Market::Sweden.extract(&errors.to_string()).is_err());
    assert!(
        Market::Sweden
            .extract("<html>Swedish default</html>")
            .is_err()
    );
    assert!(
        Market::Sweden
            .validate_download("https://www.ankernordics.com/products/other", CASES[0].1)
            .is_err()
    );
}

#[test]
fn decimal_prices_are_exact_and_checked() {
    for (market, amount, expected, fixture) in [
        (Market::Finland, "89.99", 8999, CASES[2].1),
        (Market::Finland, "0.1", 10, CASES[2].1),
        (Market::Iceland, "6490.0", 6490, CASES[3].1),
        (Market::Sweden, "0", 0, CASES[0].1),
    ] {
        let mut value: serde_json::Value = serde_json::from_str(fixture).unwrap();
        value["data"]["product"]["variants"]["nodes"][0]["price"]["amount"] = json!(amount);
        let document = market
            .normalize(&value.to_string())
            .unwrap()
            .document_json
            .unwrap();
        let document: serde_json::Value = serde_json::from_str(&document).unwrap();
        assert_eq!(document["variants"][0]["price"]["amount"], expected);
    }
    for amount in [
        "-1",
        "NaN",
        "1e3",
        "",
        "0.001",
        "99999999999999999999999999",
    ] {
        let mut value: serde_json::Value = serde_json::from_str(CASES[2].1).unwrap();
        value["data"]["product"]["variants"]["nodes"][0]["price"]["amount"] = json!(amount);
        assert!(Market::Finland.normalize(&value.to_string()).is_err());
    }
}

#[test]
fn shared_catalog_rejects_foreign_hosts_and_invented_country_paths() {
    use ::shared::link::LinkKind;
    for source in [
        "main",
        "https://www.ankernordics.com/server-sitemap-index-products.xml",
    ] {
        assert_eq!(sitemap::classify_link(URL, source, 0), LinkKind::Product);
        assert_eq!(
            sitemap::classify_link(&URL.to_uppercase(), source, 0),
            LinkKind::Product
        );
        for foreign in [
            "https://www.ankernordics.com.evil.test/products/a",
            "https://www.anker.com/se/products/a",
            "https://www.ankernordics.com/no/products/a",
        ] {
            assert_eq!(
                sitemap::classify_link(foreign, source, 0),
                LinkKind::NotInterested
            );
            assert!(Market::Norway.request(foreign).is_err());
        }
    }
    assert_eq!(
        sitemap::classify_link(
            "https://www.ankernordics.com/collections/solar-panels",
            "main",
            0
        ),
        LinkKind::Catalog
    );
    assert_eq!(
        sitemap::classify_link("https://www.ankernordics.com/blogs/se/news", "main", 0),
        LinkKind::Content
    );
}
