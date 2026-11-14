use retailer_sauce::shopify::initial_fill::StorefrontToken;
use serde_json::Value;
#[path = "module_tokens.rs"]
mod module_tokens;
/// Scan configuration as data, without evaluating JavaScript. Recognizes
/// `storeTokens: { au: "..." }`, `storeTokens: JSON.parse("...")`,
/// assignments, `storeTokens.au = "..."`, and literal Webpack module imports.
pub fn extract_storefront_token(
    source: &str,
    market: &str,
) -> Result<Option<StorefrontToken>, String> {
    let tokens = lex(source)?;
    let mut found = None;
    // Generic public storefront settings are used only for stores without an
    // explicit regional token map. Ignore empty plugin settings and references.
    if market == "__public_storefront__" {
        for window in tokens.windows(3) {
            if let [
                Token::Word(key) | Token::String(key),
                Token::Punct(':' | '='),
                Token::String(value),
            ] = window
                && matches!(
                    key.as_str(),
                    "storefrontAccessToken"
                        | "storefrontApiAccessToken"
                        | "storefrontToken"
                        | "storefront_access_token"
                )
                && !value.is_empty()
            {
                merge_token(&mut found, StorefrontToken::new(value.clone())?)?;
            }
        }
        return Ok(found);
    }

    // Some bootstrap bundles wrap configuration in JSON.parse("...").
    for token in &tokens {
        if let Token::String(text) = token
            && text.contains("storeTokens")
            && let Ok(value) = serde_json::from_str::<Value>(text)
        {
            token_from_json(&value, market, &mut found)?;
        }
    }
    for (i, token) in tokens.iter().enumerate() {
        if !matches!(token, Token::Word(s) | Token::String(s) if s == "storeTokens") {
            continue;
        }
        let tail = &tokens[i + 1..];
        if let [
            Token::Punct(':' | '='),
            Token::Word(json),
            Token::Punct('.'),
            Token::Word(parse),
            Token::Punct('('),
            Token::String(encoded),
            Token::Punct(')'),
            ..,
        ] = tail
            && json == "JSON"
            && parse == "parse"
        {
            // Decode only the literal map attached to storeTokens. Never run
            // JavaScript or accept an unrelated JSON map as authentication.
            let map: Value = serde_json::from_str(encoded)
                .map_err(|_| "invalid JSON in Storefront token map")?;
            let map = map
                .as_object()
                .ok_or("Storefront token map is not an object")?;
            if let Some(value) = map.get(market) {
                let value = value
                    .as_str()
                    .ok_or("Storefront token is not a string literal")?;
                merge_token(&mut found, StorefrontToken::new(value.into())?)?;
            }
        }
        if let [
            Token::Punct('.'),
            Token::Word(key),
            Token::Punct('='),
            Token::String(value),
            ..,
        ] = tail
            && key == market
        {
            merge_token(&mut found, StorefrontToken::new(value.clone())?)?;
        }
        if !matches!(tail, [Token::Punct(':' | '='), Token::Punct('{'), ..]) {
            continue;
        }
        let mut depth = 1;
        for j in 2..tail.len() {
            match &tail[j] {
                Token::Punct('{') => depth += 1,
                Token::Punct('}') => {
                    depth -= 1;
                    if depth == 0 {
                        break;
                    }
                }
                Token::Word(key) | Token::String(key)
                    if depth == 1
                        && key == market
                        && (j == 2 || matches!(tail[j - 1], Token::Punct(','))) =>
                {
                    if let Some([Token::Punct(':'), Token::String(value)]) = tail.get(j + 1..j + 3)
                    {
                        merge_token(&mut found, StorefrontToken::new(value.clone())?)?;
                    } else {
                        return Err("Storefront token is not a string literal".into());
                    }
                }
                _ => {}
            }
        }
        if depth != 0 {
            return Err("unterminated Storefront token map".into());
        }
    }
    module_tokens::extract(&tokens, market, &mut found)?;
    Ok(found)
}

fn token_from_json(
    value: &Value,
    market: &str,
    found: &mut Option<StorefrontToken>,
) -> Result<(), String> {
    match value {
        Value::Object(object) => {
            if let Some(value) = object
                .get("storeTokens")
                .and_then(|tokens| tokens.get(market))
            {
                let token = value
                    .as_str()
                    .ok_or("Storefront token is not a string literal")?;
                merge_token(found, StorefrontToken::new(token.into())?)?;
            }
            for value in object.values() {
                token_from_json(value, market, found)?;
            }
        }
        Value::Array(values) => {
            for value in values {
                token_from_json(value, market, found)?;
            }
        }
        _ => {}
    }
    Ok(())
}

pub fn merge_token(
    current: &mut Option<StorefrontToken>,
    next: StorefrontToken,
) -> Result<(), String> {
    if current.as_ref().is_some_and(|old| old != &next) {
        return Err("conflicting Storefront tokens for the selected market".into());
    }
    *current = Some(next);
    Ok(())
}

#[derive(Debug)]
enum Token {
    Word(String),
    String(String),
    Punct(char),
}

fn regex_position(previous: Option<&Token>) -> bool {
    matches!(
        previous,
        None | Some(Token::Punct(
            '=' | '(' | '[' | '{' | ',' | ':' | ';' | '!' | '?' | '&' | '|'
        ))
    ) || matches!(previous, Some(Token::Word(word)) if word == "return" || word == "throw")
}

fn lex(source: &str) -> Result<Vec<Token>, String> {
    let mut chars = source.chars().peekable();
    let mut tokens = Vec::new();
    while let Some(c) = chars.next() {
        if c == '/' && chars.peek() == Some(&'/') {
            for ch in chars.by_ref() {
                if ch == '\n' {
                    break;
                }
            }
        } else if c == '/' && chars.peek() == Some(&'*') {
            chars.next();
            while let Some(ch) = chars.next() {
                if ch == '*' && chars.peek() == Some(&'/') {
                    chars.next();
                    break;
                }
            }
        } else if c == '/' && regex_position(tokens.last()) {
            // Regex literals in unrelated minified code may contain quotes or braces.
            let mut bracket = false;
            while let Some(ch) = chars.next() {
                if ch == '\\' {
                    chars.next();
                    continue;
                }
                if ch == '[' {
                    bracket = true;
                }
                if ch == ']' {
                    bracket = false;
                }
                if ch == '/' && !bracket {
                    break;
                }
            }
            while chars.peek().is_some_and(|c| c.is_ascii_alphabetic()) {
                chars.next();
            }
        } else if c == '"' || c == '\'' || c == '`' {
            let mut value = String::new();
            let mut closed = false;
            while let Some(ch) = chars.next() {
                if ch == c {
                    closed = true;
                    break;
                }
                if ch == '\\' {
                    let escaped = chars.next().ok_or("unterminated JavaScript string")?;
                    match escaped {
                        'u' | 'x' => {
                            let count = if escaped == 'u' { 4 } else { 2 };
                            let hex: String = chars.by_ref().take(count).collect();
                            let code = u32::from_str_radix(&hex, 16).ok().and_then(char::from_u32);
                            // Surrogate escapes in unrelated storefront strings are irrelevant.
                            value.push(code.unwrap_or('\u{fffd}'));
                        }
                        'n' => value.push('\n'),
                        'r' => value.push('\r'),
                        't' => value.push('\t'),
                        other => value.push(other),
                    }
                } else {
                    value.push(ch);
                }
            }
            if !closed {
                return Err("unterminated JavaScript string".into());
            }
            tokens.push(Token::String(value));
        } else if c.is_ascii_alphanumeric() || c == '_' || c == '$' {
            let mut word = c.to_string();
            while chars
                .peek()
                .is_some_and(|ch| ch.is_ascii_alphanumeric() || *ch == '_' || *ch == '$')
            {
                word.push(chars.next().unwrap());
            }
            tokens.push(Token::Word(word));
        } else if !c.is_whitespace() {
            tokens.push(Token::Punct(c));
        }
    }
    Ok(tokens)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn selects_only_the_requested_market_and_decodes_literals() {
        for source in [
            r#"const x={storeTokens:{ca:'canada',au:'australia'}}"#,
            r#"window.storeTokens = {"au":"austral\u0069a"}"#,
            r#"storeTokens.au = "australia""#,
        ] {
            assert_eq!(
                extract_storefront_token(source, "au")
                    .unwrap()
                    .unwrap()
                    .as_str(),
                "australia"
            );
        }
    }
    #[test]
    fn handles_regex_literals_encoded_json_and_market_names_in_values() {
        for source in [
            r#"const regex=/["']/g; window.config={storeTokens:{au:'australia',other:'au'}};"#,
            r#"const config=JSON.parse('{"storeTokens":{"au":"australia"}}');"#,
        ] {
            assert_eq!(
                extract_storefront_token(source, "au")
                    .unwrap()
                    .unwrap()
                    .as_str(),
                "australia"
            );
        }
        assert!(extract_storefront_token("storeTokens:{au:'unfinished'", "au").is_err());
    }
    #[test]
    fn decodes_json_parse_maps_only_when_assigned_to_store_tokens() {
        for source in [
            r#"config={storeTokens:JSON.parse('{"ca":"canada","au":"australia"}')}"#,
            r#"config.storeTokens = JSON.parse('{"au":"australia"}')"#,
        ] {
            assert_eq!(
                extract_storefront_token(source, "au")
                    .unwrap()
                    .unwrap()
                    .as_str(),
                "australia"
            );
            assert!(
                extract_storefront_token(source, "unknown")
                    .unwrap()
                    .is_none()
            );
        }
        assert!(
            extract_storefront_token(
                r#"other=JSON.parse('{"au":"unrelated"}'); config.storeTokens=variable"#,
                "au"
            )
            .unwrap()
            .is_none()
        );
        for source in [
            r#"storeTokens:JSON.parse('{broken}')"#,
            r#"storeTokens:JSON.parse('[]')"#,
            r#"storeTokens:JSON.parse('{"au":42}')"#,
            r#"storeTokens:JSON.parse('{"au":""}')"#,
            r#"storeTokens:JSON.parse('{"au":"one"}'); storeTokens.au='two'"#,
        ] {
            assert!(extract_storefront_token(source, "au").is_err());
        }
    }

    #[test]
    fn missing_blank_conflicting_and_duplicate_tokens() {
        assert!(
            extract_storefront_token("storeTokens:{ca:'canada'}", "au")
                .unwrap()
                .is_none()
        );
        for source in [
            "storeTokens:{au:''}",
            "storeTokens:{au:123}",
            "storeTokens:{au:'one',au:'two'}",
        ] {
            assert!(extract_storefront_token(source, "au").is_err());
        }
        assert!(
            extract_storefront_token("storeTokens:{au:'one',au:'one'}", "au")
                .unwrap()
                .is_some()
        );
        assert!(
            extract_storefront_token("// storeTokens:{au:'fake'}\n const x=1", "au")
                .unwrap()
                .is_none()
        );
        assert!(
            !format!("{:?}", StorefrontToken::new("secret".into()).unwrap()).contains("secret")
        );
    }
}
