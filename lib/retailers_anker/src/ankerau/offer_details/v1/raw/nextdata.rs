//! The `script#__NEXT_DATA__` payload. Types the envelope, the small regions of `pageProps` and
//! the product record; the large storefront-configuration regions (`buildProps`, `metafields`,
//! `shop`) are named but left as [`Value`] rather than typed, since they carry no offer fact.

pub mod product;

use html_extractor::serde_json::Value;
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NextDataScript {
    pub id: String,
    #[serde(rename = "type")]
    pub type_: String,
    pub data: NextData,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct NextData {
    pub build_id: String,
    pub page: String,
    pub locale: String,
    pub default_locale: String,
    pub locales: Vec<String>,
    pub gsp: bool,
    pub is_fallback: bool,
    pub is_experimental_compile: bool,
    pub dynamic_ids: Vec<i64>,
    pub script_loader: Vec<Value>,
    pub props: Props,
    pub query: Query,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Props {
    #[serde(rename = "__N_SSG")]
    pub n_ssg: bool,
    #[serde(rename = "pageProps")]
    pub page_props: PageProps,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Query {
    pub slug: String,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PageProps {
    pub slug: String,
    pub page_type: String,
    #[serde(rename = "type")]
    pub type_: String,
    pub related_products: Vec<Value>,
    pub brands: Vec<BrandEdge>,
    pub seo: Seo,
    pub nav_categories: Vec<NavCategory>,

    pub product: product::Product,

    pub build_props: Value,
    pub metafields: Value,
    pub shop: Value,

    /// Keyed by SKU, bundles included. Every list is empty on the saved pages, so the entry
    /// shape is unknown and left untyped. Absent from older pages, which read as an empty map.
    #[serde(default)]
    pub legal_guarantee: BTreeMap<String, Vec<Value>>,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BrandEdge {
    pub node: Brand,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Brand {
    pub entity_id: String,
    pub name: String,
    pub path: String,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Seo {
    pub title: String,
    pub description: Option<String>,
    pub canonical: String,
    pub language_alternates: Vec<LanguageAlternate>,
    pub open_graph: OpenGraph,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct LanguageAlternate {
    pub href: String,
    pub href_lang: String,
}

// No `rename_all` here, deliberately: this object spells its key `site_name`, in snake_case,
// while every sibling object in the payload is camelCase. A blanket `rename_all = "camelCase"`
// would look for `siteName` and, with `deny_unknown_fields`, fail every document.
#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OpenGraph {
    pub title: String,
    pub description: Option<String>,
    #[serde(rename = "type")]
    pub type_: String,
    pub site_name: String,
    pub images: Vec<OpenGraphImage>,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OpenGraphImage {
    pub url: String,
    pub alt: String,
    // Strings here; the same two numbers are integers on `NavProductImage` below.
    pub width: String,
    pub height: String,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NavCategory {
    pub id: String,
    pub name: String,
    pub path: String,
    pub slug: String,
    pub products: Vec<NavProduct>,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NavProduct {
    pub name: String,
    pub path: String,
    pub images: Vec<NavProductImage>,
    pub tags: Vec<String>,
    pub metafields: NavProductMetafields,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct NavProductImage {
    pub url: String,
    pub alt_text: Option<String>,
    pub width: i64,
    pub height: i64,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct NavProductMetafields {
    pub product_note: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::{NextData, NextDataScript, PageProps};
    use html_extractor::serde_json;

    #[test]
    fn the_envelope_is_typed_and_the_large_regions_are_named() {
        let props: PageProps = serde_json::from_str(PAGE_PROPS).expect("pageProps deserializes");

        assert_eq!(
            props.page_type, "charger",
            "the wire's value, not `product`"
        );
        assert_eq!(props.type_, "product");
        assert_eq!(props.brands[0].node.name, "Anker");
        assert_eq!(props.seo.open_graph.images[0].height, "630");
        assert_eq!(props.seo.open_graph.site_name, "Anker");

        let nav_product = &props.nav_categories[0].products[0];
        assert_eq!(nav_product.images[0].height, 800);
        assert_eq!(nav_product.metafields.product_note.as_deref(), Some("note"));

        assert_eq!(props.product.variants[0].sku, "A110BH11");
        assert_eq!(props.product.price.value, 229.99);
        assert_eq!(props.product.list_price, 0.0, "a bare integer on the wire");
        assert!(
            props.product.variants[0].list_price.is_none(),
            "null, not absent"
        );
        assert!(
            props.product.seo.title.is_none(),
            "product.seo is an empty object on half the corpus"
        );
        assert!(
            props.product.metafields.is_object(),
            "product.metafields stays named"
        );

        assert!(props.build_props.is_object(), "buildProps stays named");
        assert!(props.metafields.is_object(), "metafields stays named");
        assert!(props.shop.is_object(), "shop stays named");
    }

    #[test]
    fn the_open_graph_site_name_is_read_from_its_snake_case_key() {
        let props: PageProps = serde_json::from_str(PAGE_PROPS).expect("pageProps deserializes");
        assert_eq!(props.seo.open_graph.site_name, "Anker");

        let camel = PAGE_PROPS.replace(r#""site_name""#, r#""siteName""#);
        assert!(
            serde_json::from_str::<PageProps>(&camel).is_err(),
            "the camelCase spelling must not be accepted - the wire does not use it"
        );
    }

    #[test]
    fn a_document_without_a_description_still_deserializes() {
        let without = PAGE_PROPS
            .replace(r#""description":"d","canonical""#, r#""canonical""#)
            .replace(
                r#""description":"d","type":"website""#,
                r#""type":"website""#,
            );

        let props: PageProps = serde_json::from_str(&without).expect("both descriptions optional");
        assert!(props.seo.description.is_none());
        assert!(props.seo.open_graph.description.is_none());
    }

    #[test]
    fn a_product_without_a_description_still_deserializes() {
        let without = PAGE_PROPS.replace(r#""description":"d","descriptionHtml":"<p>d</p>","#, "");

        let props: PageProps =
            serde_json::from_str(&without).expect("a reservation page states no description");
        assert!(props.product.description.is_none());
        assert!(props.product.description_html.is_none());
    }

    #[test]
    fn the_script_particle_carries_its_attributes_and_the_payload() {
        let script = format!(
            r#"{{"id":"__NEXT_DATA__","type":"application/json","data":{}}}"#,
            next_data()
        );
        let script: NextDataScript = serde_json::from_str(&script).expect("script deserializes");

        assert_eq!(script.id, "__NEXT_DATA__");
        assert_eq!(script.type_, "application/json");
        assert!(script.data.gsp, "these pages are statically generated");
        assert_eq!(script.data.page, "/products/[slug]");
    }

    #[test]
    fn an_unmodelled_envelope_key_is_rejected() {
        let extra = next_data().replace(r#"{"buildId""#, r#"{"surprise":1,"buildId""#);
        assert!(serde_json::from_str::<NextData>(&extra).is_err());
    }

    fn next_data() -> String {
        format!(
            r#"{{"buildId":"b","page":"/products/[slug]","locale":"au","defaultLocale":"au",
               "locales":["au"],"gsp":true,"isFallback":false,"isExperimentalCompile":false,
               "dynamicIds":[1],"scriptLoader":[],"query":{{"slug":"s"}},
               "props":{{"__N_SSG":true,"pageProps":{PAGE_PROPS}}}}}"#
        )
    }

    const PAGE_PROPS: &str = r##"{
        "slug":"s","pageType":"charger","type":"product","relatedProducts":[],
        "brands":[{"node":{"entityId":"1","name":"Anker","path":"/p"}}],
        "seo":{"title":"t","description":"d","canonical":"c","languageAlternates":[
            {"href":"h","hrefLang":"en-US"}],
            "openGraph":{"title":"t","description":"d","type":"website","site_name":"Anker",
            "images":[{"url":"u","alt":"a","width":"1200","height":"630"}]}},
        "navCategories":[{"id":"1","name":"n","path":"/p","slug":"s","products":[
            {"name":"p","path":"/x","images":[{"url":"u","altText":"a","width":800,"height":800}],
             "tags":["t"],"metafields":{"productNote":"note"}}]}],
        "buildProps":{},"metafields":{},"shop":{},
        "product":{"id":"gid://shopify/Product/1","handle":"h","slug":"s","path":"/p",
            "name":"n","title":"t","description":"d","descriptionHtml":"<p>d</p>",
            "vendor":"Anker","publishedAt":"2026-01-01T00:00:00Z",
            "price":{"value":229.99,"currencyCode":"AUD"},"listPrice":0,
            "availableForSale":true,"totalInventory":13,
            "images":[{"url":"u","width":800,"height":800}],"tags":["t"],
            "options":[],"collections":{"nodes":[{"id":"c","handle":"ch","title":"ct"}]},
            "seo":{},"requiresSellingPlan":false,"sellingPlanGroups":[],
            "metafields":{},"metafieldsMap":{"product":{"infos":"productInfos"}},
            "variants":[{"id":"gid://shopify/ProductVariant/1","sku":"A110BH11",
                "barcode":"194644239350","name":"v","price":229.99,"listPrice":null,
                "quantityAvailable":0,"availableForSale":false,"currentlyNotInStock":false,
                "requiresShipping":true,"weight":2.0216,
                "image":{"url":"u","width":800,"height":800},
                "options":[{"__typename":"MultipleChoiceOption","id":"o","name":"Color",
                    "displayName":"color","values":[{"label":"Black","hexColors":["#000"]}]}],
                "sellingPlanAllocations":[],"metafields":{},
                "metafieldsMap":{"product":{"infos":"productInfos"}}}]}}"##;
}
