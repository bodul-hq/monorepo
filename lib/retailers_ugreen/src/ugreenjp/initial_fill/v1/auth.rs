use super::token::{extract_storefront_token, merge_token};
use retailer_sauce::initial_fill::Http;
use retailer_sauce::shopify::initial_fill::StorefrontToken;
use std::collections::HashSet;
pub fn resolve(http: &dyn Http, url: &str) -> Result<Option<StorefrontToken>, String> {
    let config = super::CONFIG;
    if url != config.storefront_url {
        return Err("token acquisition URL does not match retailer configuration".into());
    }
    let configured = match config.token_environment {
        Some(name) => http.configured_token(name)?,
        None if !config.requires_token() => http.configured_token(&format!(
            "{}_STOREFRONT_TOKEN",
            super::CODE.to_ascii_uppercase()
        ))?,
        None => None,
    };
    let storefront_token = if let Some(token) = configured {
        Some(token)
    } else {
        let market = config.token_market.unwrap_or("__public_storefront__");
        let html = http.get(url)?;
        let scripts = super::scripts::inspect_storefront_html(&html, market)?;
        let mut token = scripts.token;
        if token.is_none() {
            let base = url::Url::parse(config.storefront_url).map_err(|e| e.to_string())?;
            let mut visited = HashSet::new();
            for source in scripts.sources {
                let asset = base
                    .join(&source)
                    .map_err(|_| "invalid storefront script URL")?;
                let host = asset.host_str().unwrap_or("");
                if asset.scheme() != "https"
                    || !(host == base.host_str().unwrap_or("")
                        || host.ends_with(".anker.com")
                        || host == "cdn.shopify.com")
                {
                    continue;
                }
                if !visited.insert(asset.to_string()) {
                    continue;
                }
                let source = http.get(asset.as_str())?;
                if (source.contains("storeTokens")
                    || source.contains("storefrontAccessToken")
                    || source.contains("storefrontApiAccessToken")
                    || source.contains("storefrontToken")
                    || source.contains("storefront_access_token"))
                    && let Some(found) = extract_storefront_token(&source, market)?
                {
                    merge_token(&mut token, found)?;
                }
            }
        }
        if config.requires_token() && token.is_none() {
            return Err("public Storefront token not found for retailer".into());
        }
        token
    };
    Ok(storefront_token)
}
