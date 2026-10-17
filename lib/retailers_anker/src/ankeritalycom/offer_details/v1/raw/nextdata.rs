pub mod product;

use html_extractor::serde_json::Value;

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
    pub handle: String,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PageProps {
    #[serde(rename = "type")]
    pub type_: String,
    pub seo: Seo,

    pub product: product::Product,
    pub shop: Value,

    /// Keyed by SKU. Shape is not inspected, so it is left untyped. Absent from older pages.
    #[serde(default)]
    pub legal_guarantee: Value,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Seo {
    pub title: String,
    pub description: String,

    pub canonical: String,

    pub noindex: bool,

    pub thumbnail: String,

    pub language_alternates: Vec<LanguageAlternate>,
    pub open_graph: OpenGraph,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct LanguageAlternate {
    pub href: String,
    pub href_lang: String,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OpenGraph {
    pub title: String,
    pub description: String,
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
    pub width: String,
    pub height: String,
}

#[cfg(test)]
mod tests {
    use super::{NextDataScript, PageProps};
    use html_extractor::serde_json;

    #[test]
    fn the_seo_region_is_typed_and_the_product_stays_named() {
        let props: PageProps = serde_json::from_str(PAGE_PROPS).expect("pageProps deserializes");

        assert_eq!(props.type_, "product");
        assert_eq!(props.seo.open_graph.site_name, "Anker");
        assert_eq!(props.seo.open_graph.images[0].height, "630");
        assert!(!props.seo.noindex);

        assert_eq!(props.product.id, "i");
        assert_eq!(props.product.price.amount, 1.0);
        assert_eq!(props.product.price_range.min_variant_price.amount, "1.0");
    }

    #[test]
    fn the_open_graph_site_name_is_read_from_its_snake_case_key() {
        let camel = PAGE_PROPS.replace(r#""site_name""#, r#""siteName""#);
        assert!(
            serde_json::from_str::<PageProps>(&camel).is_err(),
            "the camelCase spelling must not be accepted - the wire does not use it"
        );
    }

    #[test]
    fn the_query_is_keyed_by_handle_not_slug() {
        let script = script(r#"{"handle":"h"}"#);
        let script: NextDataScript = serde_json::from_str(&script).expect("handle deserializes");
        assert_eq!(script.data.query.handle, "h");
        assert_eq!(script.data.page, "/products/[handle]");

        assert!(
            serde_json::from_str::<NextDataScript>(&script_with(r#"{"slug":"s"}"#)).is_err(),
            "the [slug] application's query shape is a different storefront"
        );
    }

    fn script(query: &str) -> String {
        script_with(query)
    }

    fn script_with(query: &str) -> String {
        format!(
            r#"{{"id":"__NEXT_DATA__","type":"application/json","data":{{
               "buildId":"b","page":"/products/[handle]","locale":"it",
               "defaultLocale":"it","locales":["it"],"gsp":true,"isFallback":false,
               "isExperimentalCompile":false,"dynamicIds":[1],"scriptLoader":[],"query":{query},
               "props":{{"__N_SSG":true,"pageProps":{PAGE_PROPS}}}}}}}"#
        )
    }

    const PAGE_PROPS: &str = r#"{
        "type":"product",
        "seo":{"title":"t","description":"d","canonical":"c","noindex":false,"thumbnail":"th",
            "languageAlternates":[{"href":"h","hrefLang":"it"}],
            "openGraph":{"title":"t","description":"d","type":"website","site_name":"Anker",
            "images":[{"url":"u","alt":"a","width":"1200","height":"630"}]}},
        "product":{"id":"i","handle":"h","slug":"s","path":"/p","title":"t","description":"d","descriptionHtml":"<p>d</p>","vendor":"Anker","productType":"pt","updatedAt":"2026-01-01T00:00:00Z","availableForSale":true,"tags":[],"price":{"amount":1.0,"currencyCode":"EUR"},"priceRange":{"minVariantPrice":{"amount":"1.0","currencyCode":"EUR"},"maxVariantPrice":{"amount":"2.0","currencyCode":"EUR"}},"compareAtPriceRange":{"minVariantPrice":{"amount":"0.0","currencyCode":"EUR"}},"featuredImage":{"url":"u","altText":null,"width":1,"height":1},"images":[],"seo":{"title":null,"description":null},"media":[],"options":[],"variants":[],"metafields":{}},"shop":{}}"#;
}
