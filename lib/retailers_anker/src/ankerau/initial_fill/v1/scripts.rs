//! Read script contents as data; downloading referenced assets belongs to the caller.
use super::token::{extract_storefront_token, merge_token};
use retailer_sauce::shopify::initial_fill::StorefrontToken;
use scraper::{Html, Selector};

pub struct StorefrontScripts {
    pub token: Option<StorefrontToken>,
    pub sources: Vec<String>,
}

pub fn inspect_storefront_html(html: &str, market: &str) -> Result<StorefrontScripts, String> {
    let document = Html::parse_document(html);
    let scripts = Selector::parse("script").expect("constant selector");
    let mut token = None;
    let mut sources = Vec::new();
    for script in document.select(&scripts) {
        if let Some(src) = script.attr("src") {
            if !sources.iter().any(|source| source == src) {
                sources.push(src.to_owned());
            }
        } else {
            let body: String = script.text().collect();
            if (body.contains("storeTokens")
                || body.contains("storefrontAccessToken")
                || body.contains("storefrontApiAccessToken")
                || body.contains("storefrontToken")
                || body.contains("storefront_access_token"))
                && let Some(found) = extract_storefront_token(&body, market)?
            {
                merge_token(&mut token, found)?;
            }
        }
    }
    Ok(StorefrontScripts { token, sources })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn finds_inline_configuration_and_external_assets() {
        let parsed = inspect_storefront_html(r#"<script src="/app.js"></script><script src="/app.js"></script><script>{"storeTokens":{"au":"synthetic-au","ca":"synthetic-ca"}}</script>"#, "au").unwrap();
        assert_eq!(parsed.token.unwrap().as_str(), "synthetic-au");
        assert_eq!(parsed.sources, ["/app.js"]);
    }
    #[test]
    fn ignores_text_outside_scripts_and_rejects_conflicting_inline_values() {
        assert!(
            inspect_storefront_html("<p>storeTokens:{au:'fake'}</p>", "au")
                .unwrap()
                .token
                .is_none()
        );
        assert!(
            inspect_storefront_html(
                "<script>storeTokens:{au:'one'}</script><script>storeTokens:{au:'two'}</script>",
                "au"
            )
            .is_err()
        );
    }
}

#[cfg(test)]
mod public_tests {
    use super::*;
    #[test]
    fn explicit_public_settings_ignore_empty_and_unrelated_values() {
        let result=inspect_storefront_html(r#"<script>var settings={storefrontAccessToken:'', adminToken:'not-a-storefront-token', storefrontApiAccessToken:'synthetic-public'};</script>"#, "__public_storefront__").unwrap();
        assert_eq!(result.token.unwrap().as_str(), "synthetic-public");
        let conflicting =
            r#"<script>var a={storefrontToken:'one',storefront_access_token:'two'};</script>"#;
        assert!(inspect_storefront_html(conflicting, "__public_storefront__").is_err());
        assert!(
            inspect_storefront_html("<p>storefrontToken:'fake'</p>", "__public_storefront__")
                .unwrap()
                .token
                .is_none()
        );
    }
}

#[cfg(test)]
mod captured_tests {
    use super::*;
    #[test]
    fn downloaded_au_homepage_requires_external_token_configuration() {
        let html = include_str!("fixtures/storefront.html");
        let scripts = inspect_storefront_html(html, "au").unwrap();
        // The actual homepage has a different Basic storefront_key and a Judge.me
        // public_token. Neither is evidence of a Shopify Storefront access token.
        assert!(html.contains("storefront_key"));
        assert!(!html.contains("storeTokens"));
        assert!(scripts.token.is_none());
        assert!(
            scripts
                .sources
                .iter()
                .any(|source| { source.starts_with("https://one.anker.com/_next/static/chunks/") })
        );
    }
}
