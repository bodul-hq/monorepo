//! Evidence is captured separately after retailer islands, then carried through the facade.
use retailer_sauce::localization::{Localization, TargetMarket, validate_language};
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub const KEY: &str = "_localization_evidence";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Evidence {
    pub selected: Localization,
    pub candidates: Vec<Candidate>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Candidate {
    pub path: String,
    pub raw: String,
    pub decision: String,
}

pub fn extract(html: &str) -> Evidence {
    let document = Html::parse_document(html);
    let mut evidence = Evidence {
        selected: Localization::default(),
        candidates: vec![],
    };
    let mut languages = Vec::new();
    let mut fallback = Vec::new();
    let mut markets = Vec::new();
    let mut invalid_market = false;
    let mut invalid_fallback = false;
    rejected_candidates(&document, &mut evidence.candidates);
    for node in document.select(&Selector::parse("html[lang]").expect("static selector")) {
        let raw = node.value().attr("lang").unwrap_or_default();
        let valid = validate_language(raw).is_ok();
        evidence.candidates.push(Candidate {
            path: "html[lang]".into(),
            raw: raw.into(),
            decision: if valid {
                "document language"
            } else {
                "unsupported or malformed language"
            }
            .into(),
        });
        if valid {
            languages.push(raw.to_owned());
        }
    }
    for node in document.select(&Selector::parse("script").expect("static selector")) {
        if node
            .value()
            .attr("type")
            .is_some_and(|kind| !matches!(kind, "text/javascript" | "application/javascript"))
        {
            continue;
        }
        for (field, raw, is_literal) in shopify_assignments(&node.inner_html()) {
            let valid = is_literal
                && match field.as_str() {
                    "country" => match raw.parse() {
                        Ok(code) => {
                            markets.push(TargetMarket::Country { code });
                            true
                        }
                        Err(_) => false,
                    },
                    "locale" => {
                        let valid = validate_language(&raw).is_ok();
                        if valid {
                            fallback.push(raw.clone());
                        }
                        valid
                    }
                    _ => false,
                };
            if !valid {
                if field == "country" {
                    invalid_market = true;
                } else {
                    invalid_fallback = true;
                }
            }
            evidence.candidates.push(Candidate {
                path: format!("script: Shopify.{field}"),
                raw,
                decision: if valid {
                    "active storefront declaration"
                } else {
                    "unsupported or malformed declaration"
                }
                .into(),
            });
        }
    }
    evidence.selected.target_market = if invalid_market {
        None
    } else {
        select(markets, &mut evidence.candidates, "market")
    };
    evidence.selected.language = select(
        if languages.is_empty() {
            if invalid_fallback {
                Vec::new()
            } else {
                fallback
            }
        } else {
            languages
        },
        &mut evidence.candidates,
        "language",
    );
    evidence
}

// Retain known misleading source paths for review without making them eligible facts.
fn rejected_candidates(document: &Html, candidates: &mut Vec<Candidate>) {
    for (selector, fields, reason) in [
        (
            "script#__NEXT_DATA__",
            &["locale", "defaultLocale"] as &[_],
            "routing/default locale is not an established offer-content declaration",
        ),
        (
            "script#apple-pay-shop-capabilities",
            &["countryCode", "currencyCode"] as &[_],
            "payment configuration does not establish the active market",
        ),
    ] {
        for node in document.select(&Selector::parse(selector).expect("static selector")) {
            if let Ok(value) = serde_json::from_str::<Value>(&node.inner_html()) {
                for field in fields {
                    if let Some(raw) = value.get(*field) {
                        candidates.push(Candidate {
                            path: format!("{selector}.{field}"),
                            raw: raw.as_str().map_or_else(|| raw.to_string(), String::from),
                            decision: reason.into(),
                        });
                    }
                }
            }
        }
    }
    for (selector, attribute, reason) in [
        (
            "[data-klarna-locale]",
            "data-klarna-locale",
            "widget language",
        ),
        ("link[hreflang]", "hreflang", "alternate-page language"),
    ] {
        for node in document.select(&Selector::parse(selector).expect("static selector")) {
            candidates.push(Candidate {
                path: selector.into(),
                raw: node.value().attr(attribute).unwrap_or_default().into(),
                decision: format!(
                    "rejected: {reason} does not establish the current offer language"
                ),
            });
        }
    }
    for node in document.select(&Selector::parse("script[src]").expect("static selector")) {
        let src = node.value().attr("src").unwrap_or_default();
        if let Some((_, query)) = src.split_once('?') {
            for parameter in query.split('#').next().unwrap_or_default().split('&') {
                if let Some(raw) = parameter.strip_prefix("locale=") {
                    candidates.push(Candidate {
                        path: "script[src] query.locale".into(),
                        raw: raw.into(),
                        decision: "rejected: external script locale is not offer-content language"
                            .into(),
                    });
                }
            }
        }
    }
}

fn select<T: PartialEq>(
    mut values: Vec<T>,
    diagnostics: &mut Vec<Candidate>,
    field: &str,
) -> Option<T> {
    if values.windows(2).any(|pair| pair[0] != pair[1]) {
        diagnostics.push(Candidate {
            path: field.into(),
            raw: String::new(),
            decision: "conflicting applicable declarations; unknown".into(),
        });
        None
    } else {
        values.pop()
    }
}

// Parse only standalone top-level assignments of a quoted literal, never expressions.
// Skip comments, strings and nested scopes so widget code cannot impersonate an active setting.
fn shopify_assignments(script: &str) -> Vec<(String, String, bool)> {
    let bytes = script.as_bytes();
    let mut i = 0;
    let mut depth = 0usize;
    let mut start = true;
    let mut result = vec![];
    while i < bytes.len() {
        if bytes[i].is_ascii_whitespace() {
            i += 1;
            continue;
        }
        if bytes[i..].starts_with(b"//") {
            while i < bytes.len() && bytes[i] != b'\n' {
                i += 1;
            }
            continue;
        }
        if bytes[i..].starts_with(b"/*") {
            i += 2;
            while i + 1 < bytes.len() && &bytes[i..i + 2] != b"*/" {
                i += 1;
            }
            i = (i + 2).min(bytes.len());
            continue;
        }
        if depth == 0 && start && bytes[i..].starts_with(b"Shopify.") {
            let rest = &script[i + 8..];
            for field in ["country", "locale"] {
                if let Some(rest) = rest.strip_prefix(field)
                    && let Some(rest) = rest.trim_start().strip_prefix('=')
                {
                    let rest = rest.trim_start();
                    if let Some((value, consumed)) = literal(rest)
                        && rest[consumed..].trim_start().starts_with(';')
                    {
                        result.push((field.into(), value, true));
                    } else {
                        result.push((
                            field.into(),
                            rest.split(';')
                                .next()
                                .unwrap_or(rest)
                                .chars()
                                .take(512)
                                .collect(),
                            false,
                        ));
                    }
                }
            }
        }
        match bytes[i] {
            b'\'' | b'"' | b'`' => {
                let quote = bytes[i];
                i += 1;
                while i < bytes.len() {
                    if bytes[i] == b'\\' {
                        i = (i + 2).min(bytes.len());
                    } else if bytes[i] == quote {
                        i += 1;
                        break;
                    } else {
                        i += 1;
                    }
                }
                start = false;
            }
            b'{' | b'(' | b'[' => {
                depth += 1;
                start = false;
                i += 1;
            }
            b'}' | b')' | b']' => {
                depth = depth.saturating_sub(1);
                start = false;
                i += 1;
            }
            b';' => {
                start = true;
                i += 1;
            }
            _ => {
                start = false;
                i += 1;
            }
        }
    }
    result
}
fn literal(input: &str) -> Option<(String, usize)> {
    let quote = *input.as_bytes().first()?;
    if !matches!(quote, b'"' | b'\'') {
        return None;
    }
    let end = input.as_bytes()[1..]
        .iter()
        .position(|byte| *byte == quote)?
        + 1;
    let text = &input[1..end];
    if text.contains('\\') || text.chars().any(char::is_control) {
        return None;
    }
    Some((text.into(), end + 1))
}

pub fn split(json: &str) -> Result<(String, Option<Evidence>), String> {
    let mut value: Value = serde_json::from_str(json).map_err(|error| error.to_string())?;
    let evidence = value
        .as_object_mut()
        .and_then(|object| object.remove(KEY))
        .map(serde_json::from_value)
        .transpose()
        .map_err(|error| error.to_string())?;
    Ok((value.to_string(), evidence))
}
pub fn attach(
    json: &mut Option<String>,
    evidence: Option<Evidence>,
    variations: bool,
) -> Result<(), String> {
    if let (Some(json), Some(evidence)) = (json, evidence) {
        let mut value: Value = serde_json::from_str(json).map_err(|error| error.to_string())?;
        if variations
            && let Some(variants) = value.get_mut("variations").and_then(Value::as_array_mut)
        {
            for variant in variants {
                variant["localization"] =
                    serde_json::to_value(&evidence.selected).map_err(|error| error.to_string())?;
            }
        }
        value[KEY] = serde_json::to_value(evidence).map_err(|error| error.to_string())?;
        *json = value.to_string();
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn current_page_facts_exclude_payment_widgets_defaults_and_other_scopes() {
        let html = include_str!("../tests/fixtures/localization/ugreen-fr.html");
        let evidence = extract(html);
        assert_eq!(
            serde_json::to_value(&evidence.selected).unwrap(),
            serde_json::json!({"target_market":{"kind":"country","code":"CA"},"language":"fr"})
        );
        assert!(
            evidence.candidates.iter().any(|candidate| {
                candidate.raw == "US" && candidate.decision.contains("payment")
            })
        );
        assert!(evidence.candidates.iter().any(|candidate| {
            candidate.raw == "fr_FR" && candidate.decision.contains("rejected")
        }));
        let anker = extract(include_str!("../tests/fixtures/localization/anker-au.html"));
        assert_eq!(anker.selected.language.as_deref(), Some("en-AU"));
        assert!(anker.selected.target_market.is_none());
        assert!(
            anker.candidates.iter().any(|candidate| {
                candidate.raw == "us" && candidate.decision.contains("default")
            })
        );
        let malicious = extract(
            r#"<script>/* Shopify.country = "US"; */ var a = 'Shopify.country = "FR";'; if (true) { Shopify.country = "DE"; } Shopify.country = "CA";</script>"#,
        );
        assert!(malicious.selected.target_market.is_none()); // unsupported non-standalone setting
        assert_eq!(
            extract(
                "<html><script>Shopify.country = \"CA\"; Shopify.country = \"US\";</script></html>"
            )
            .selected,
            Localization::default()
        );
        assert_eq!(
            extract("<html lang=bad_tag><script>Shopify.country = \"EU\";</script></html>")
                .selected,
            Localization::default()
        );
    }
    #[test]
    fn dynamic_or_malformed_overrides_invalidate_static_storefront_facts() {
        for assignment in [
            "Shopify.country = getMarket();",
            "Shopify.country = CA;",
            "Shopify.country = 'US' + suffix;",
            "Shopify.country = 'EU';",
        ] {
            let html = format!(
                "<script>Shopify.country = 'CA'; Shopify.locale = 'fr'; {assignment} Shopify.locale = getLanguage();</script>"
            );
            let evidence = extract(&html);
            assert_eq!(evidence.selected, Localization::default());
            assert!(
                evidence.candidates.iter().any(|candidate| {
                    candidate.decision == "unsupported or malformed declaration"
                })
            );
        }
    }
    #[test]
    fn evidence_survives_both_intermediate_stages() {
        let evidence = extract(include_str!(
            "../tests/fixtures/localization/ugreen-fr.html"
        ));
        let mut document = Some("{}".into());
        attach(&mut document, Some(evidence), false).unwrap();
        let (_, evidence) = split(document.as_deref().unwrap()).unwrap();
        let mut structuralized = Some(r#"{"variations":[{},{}]}"#.into());
        attach(&mut structuralized, evidence, true).unwrap();
        let value: Value = serde_json::from_str(structuralized.as_deref().unwrap()).unwrap();
        assert_eq!(value["variations"][0]["localization"]["language"], "fr");
        assert_eq!(
            value["variations"][0]["localization"],
            value["variations"][1]["localization"]
        );
    }
}
