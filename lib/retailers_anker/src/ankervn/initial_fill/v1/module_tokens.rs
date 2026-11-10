//! Resolve the literal token map explicitly imported by a Webpack storefront.
//! This is a narrow data reader, not a JavaScript evaluator: module IDs, export
//! getters and JSON.parse arguments must all be literal and unambiguous.
use super::{StorefrontToken, Token, merge_token};
use serde_json::Value;

struct Module<'a> {
    id: &'a str,
    exports: &'a str,
    loader: &'a str,
    body: &'a [Token],
}

fn word(token: &Token, expected: &str) -> bool {
    matches!(token, Token::Word(value) if value == expected)
}

fn closing_brace(tokens: &[Token], open: usize) -> Option<usize> {
    let mut depth = 0;
    for (index, token) in tokens.iter().enumerate().skip(open) {
        match token {
            Token::Punct('{') => depth += 1,
            Token::Punct('}') => {
                depth -= 1;
                if depth == 0 {
                    return Some(index);
                }
            }
            _ => {}
        }
    }
    None
}

fn modules(tokens: &[Token]) -> Vec<Module<'_>> {
    let mut modules = Vec::new();
    for (index, window) in tokens.windows(11).enumerate() {
        if let [
            Token::Word(id),
            Token::Punct(':'),
            function,
            Token::Punct('('),
            Token::Word(_),
            Token::Punct(','),
            Token::Word(exports),
            Token::Punct(','),
            Token::Word(loader),
            Token::Punct(')'),
            Token::Punct('{'),
        ] = window
            && word(function, "function")
            && id.bytes().all(|b| b.is_ascii_digit())
            && let Some(end) = closing_brace(tokens, index + 10)
        {
            modules.push(Module {
                id,
                exports,
                loader,
                body: &tokens[index + 11..end],
            });
        }
    }
    modules
}

/// Only module-scope assignments can supply imported configuration. Local
/// variables inside unrelated functions may reuse the same minified name.
fn top_level_windows(tokens: &[Token], size: usize) -> impl Iterator<Item = &[Token]> {
    let mut depth = 0;
    tokens.windows(size).filter(move |window| {
        let top_level = depth == 0;
        match window[0] {
            Token::Punct('{') => depth += 1,
            Token::Punct('}') => depth -= 1,
            _ => {}
        }
        top_level
    })
}

fn imported_module<'a>(module: &Module<'a>, alias: &str) -> Result<Option<&'a str>, String> {
    let mut found = None;
    for window in top_level_windows(module.body, 6) {
        if let [
            name,
            Token::Punct('='),
            loader,
            Token::Punct('('),
            Token::Word(id),
            Token::Punct(')'),
        ] = window
            && word(name, alias)
            && word(loader, module.loader)
        {
            if found.is_some_and(|previous| previous != id) {
                return Err("ambiguous Storefront token module import".into());
            }
            found = Some(id.as_str());
        }
    }
    Ok(found)
}

fn exported_local<'a>(module: &Module<'a>, export: &str) -> Result<Option<&'a str>, String> {
    let mut found = None;
    for (index, window) in module.body.windows(7).enumerate() {
        if let [
            loader,
            Token::Punct('.'),
            define,
            Token::Punct('('),
            exports,
            Token::Punct(','),
            Token::Punct('{'),
        ] = window
            && word(loader, module.loader)
            && word(define, "d")
            && word(exports, module.exports)
            && let Some(end) = closing_brace(module.body, index + 6)
        {
            for entry in module.body[index + 7..end].windows(9) {
                if let [
                    name,
                    Token::Punct(':'),
                    function,
                    Token::Punct('('),
                    Token::Punct(')'),
                    Token::Punct('{'),
                    returns,
                    Token::Word(local),
                    Token::Punct('}'),
                ] = entry
                    && word(name, export)
                    && word(function, "function")
                    && word(returns, "return")
                {
                    if found.is_some_and(|previous| previous != local) {
                        return Err("ambiguous Storefront token module export".into());
                    }
                    found = Some(local.as_str());
                }
            }
        }
    }
    Ok(found)
}

pub(super) fn extract(
    tokens: &[Token],
    market: &str,
    found: &mut Option<StorefrontToken>,
) -> Result<(), String> {
    let modules = modules(tokens);
    for consumer in &modules {
        for (index, window) in consumer.body.windows(5).enumerate() {
            let [
                property,
                Token::Punct(':'),
                Token::Word(alias),
                Token::Punct('.'),
                Token::Word(export),
            ] = window
            else {
                continue;
            };
            if !(word(property, "storeTokens")
                || (word(property, "storefrontAccessToken")
                    && matches!(consumer.body.get(index + 5), Some(Token::Punct('[')))))
            {
                continue;
            }
            let Some(id) = imported_module(consumer, alias)? else {
                continue;
            };
            let mut matching = modules.iter().filter(|module| module.id == id);
            let Some(provider) = matching.next() else {
                continue;
            };
            if matching.next().is_some() {
                return Err("ambiguous Storefront token module identity".into());
            }
            let Some(local) = exported_local(provider, export)? else {
                continue;
            };
            for assignment in top_level_windows(provider.body, 8) {
                if let [
                    name,
                    Token::Punct('='),
                    json,
                    Token::Punct('.'),
                    parse,
                    Token::Punct('('),
                    Token::String(encoded),
                    Token::Punct(')'),
                ] = assignment
                    && word(name, local)
                    && word(json, "JSON")
                    && word(parse, "parse")
                {
                    let value: Value = serde_json::from_str(encoded)
                        .map_err(|_| "invalid JSON in imported Storefront token map")?;
                    let map = value
                        .as_object()
                        .ok_or("imported Storefront token map is not an object")?;
                    if let Some(value) = map.get(market) {
                        let value = value
                            .as_str()
                            .ok_or("Storefront token is not a string literal")?;
                        merge_token(found, StorefrontToken::new(value.into())?)?;
                    }
                }
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::super::extract_storefront_token;

    fn bundle(map: &str) -> String {
        format!(
            r#"(self.webpackChunk=self.webpackChunk||[]).push([[1],{{
            10:function(e,t,n){{n.d(t,{{qj:function(){{return value}}}});
                let value=JSON.parse('{map}');
                function unrelated(){{let value=JSON.parse('{{"fr":"wrong-shadow"}}');}}
            }},
            20:function(e,t,n){{let L=n(10);start({{storeTokens:L.qj||{{}}}});}},
            30:function(e,t,n){{let value=JSON.parse('{{"fr":"wrong-module"}}');}}
        }}]);"#
        )
    }

    #[test]
    fn follows_import_and_export_without_selecting_unrelated_or_shadowed_maps() {
        let source = bundle(r#"{"fr":"public-fr","eu-pl":"public-pl"}"#);
        for (market, expected) in [("fr", "public-fr"), ("eu-pl", "public-pl")] {
            assert_eq!(
                extract_storefront_token(&source, market)
                    .unwrap()
                    .unwrap()
                    .as_str(),
                expected
            );
        }
        assert!(extract_storefront_token(&source, "nz").unwrap().is_none());
        for source in [
            source.replace("L=n(10)", "L=n(99)"),
            source.replace("storeTokens:L.qj", "other:L.qj"),
        ] {
            assert!(extract_storefront_token(&source, "fr").unwrap().is_none());
        }
    }

    #[test]
    fn follows_indexed_storefront_access_token_but_not_unrelated_imports() {
        let source = bundle(r#"{"se":"public-se"}"#)
            .replace("storeTokens:L.qj||{}", "storefrontAccessToken:L.qj[locale]");
        assert_eq!(
            extract_storefront_token(&source, "se")
                .unwrap()
                .unwrap()
                .as_str(),
            "public-se"
        );
        assert!(
            extract_storefront_token(&source.replace("L.qj[locale]", "L.qj"), "se")
                .unwrap()
                .is_none()
        );
    }

    #[test]
    fn rejects_invalid_selected_maps_and_conflicting_tokens() {
        for map in ["{broken}", "[]", r#"{"fr":42}"#, r#"{"fr":""}"#] {
            assert!(extract_storefront_token(&bundle(map), "fr").is_err());
        }
        let source = bundle(r#"{"fr":"public-fr"}"#);
        assert!(
            extract_storefront_token(&(source.clone() + ";storeTokens.fr='different'"), "fr")
                .is_err()
        );
        assert!(
            extract_storefront_token(
                &source.replace("let L=n(10);", "let L=n(10);L=n(30);"),
                "fr"
            )
            .is_err()
        );
    }
}
