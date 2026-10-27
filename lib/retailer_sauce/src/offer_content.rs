//! Localized offer content. Source/export description observations keep their
//! separate `structuralized::content` contract.

use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Content {
    pub language: Option<String>,
    pub title: String,
    pub description: Option<String>,
}

impl<'de> Deserialize<'de> for Content {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Fields {
            #[serde(deserialize_with = "crate::observation::required_nullable")]
            language: Option<String>,
            title: String,
            #[serde(deserialize_with = "crate::observation::required_nullable")]
            description: Option<String>,
        }
        let fields: Fields = crate::observation::object(deserializer)?;
        Ok(Self {
            language: fields.language,
            title: fields.title,
            description: fields.description,
        })
    }
}

impl Content {
    pub fn guard(&self) -> Result<(), String> {
        if let Some(language) = &self.language {
            crate::localization::validate_language(language)?;
        }
        if self.title.trim().is_empty() || self.title.contains('\0') {
            return Err(err_title());
        }
        if self
            .description
            .as_ref()
            .is_some_and(|value| value.contains('\0'))
        {
            return Err(err_description());
        }
        Ok(())
    }
}

pub fn guard(content: &[Content], language: &Option<String>) -> Result<(), String> {
    if content.is_empty() {
        return Err(err_empty());
    }
    let mut keys = HashSet::new();
    for entry in content {
        entry.guard()?;
        if !keys.insert(&entry.language) {
            return Err(err_duplicate());
        }
    }
    selected(content, language)?;
    Ok(())
}

pub fn selected<'a>(
    content: &'a [Content],
    language: &Option<String>,
) -> Result<&'a Content, String> {
    content
        .iter()
        .find(|entry| &entry.language == language)
        .ok_or_else(err_missing_selection)
}

pub fn sort(content: &mut [Content]) {
    content.sort_by(|left, right| left.language.cmp(&right.language));
}

pub fn singleton(
    title: String,
    language: Option<String>,
    description: Option<String>,
) -> Vec<Content> {
    vec![Content {
        language,
        title,
        description,
    }]
}

/// Missing description preserves only the destination entry, never another language.
pub fn upsert(
    content: &[Content],
    language: Option<String>,
    title: String,
    description: Option<Option<String>>,
) -> Result<Vec<Content>, String> {
    if let Some(first) = content.first() {
        guard(content, &first.language)?;
    }
    let previous = content.iter().find(|entry| entry.language == language);
    let entry = Content {
        language: language.clone(),
        title,
        description: description
            .unwrap_or_else(|| previous.and_then(|entry| entry.description.clone())),
    };
    entry.guard()?;
    let mut result = content.to_vec();
    result.retain(|entry| entry.language != language);
    result.push(entry);
    sort(&mut result);
    guard(&result, &language)?;
    Ok(result)
}

pub fn legacy_title(
    content: &[Content],
    active: &Option<String>,
    title: String,
) -> Result<Vec<Content>, String> {
    selected(content, active)?;
    upsert(content, active.clone(), title, None)
}

pub fn legacy_description(
    content: &[Content],
    active: &Option<String>,
    description: Option<String>,
) -> Result<Vec<Content>, String> {
    let current = selected(content, active)?;
    upsert(
        content,
        active.clone(),
        current.title.clone(),
        Some(description),
    )
}

/// Old language changes corrected one tuple; they did not observe a translation.
pub fn legacy_language(
    content: &[Content],
    active: &Option<String>,
    language: Option<String>,
) -> Result<Vec<Content>, String> {
    guard(content, active)?;
    let mut moved = selected(content, active)?.clone();
    moved.language = language.clone();
    moved.guard()?;
    let mut result = content.to_vec();
    result.retain(|entry| &entry.language != active && entry.language != language);
    result.push(moved);
    sort(&mut result);
    guard(&result, &language)?;
    Ok(result)
}

fn err_title() -> String {
    "content title must be nonblank and contain no NUL".into()
}
fn err_description() -> String {
    "content description must contain no NUL".into()
}
fn err_empty() -> String {
    "offer content must contain at least one entry".into()
}
fn err_duplicate() -> String {
    "offer content language keys must be unique".into()
}
fn err_missing_selection() -> String {
    "selected language must identify a content entry".into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn merge_preserves_only_the_destination_description_and_exact_values() {
        let en = Some("en-US".into());
        let fr = Some("fr".into());
        let original = singleton(
            " English ž ".into(),
            en.clone(),
            Some(" <p>text</p>\n".into()),
        );
        let french = upsert(&original, fr.clone(), "Français".into(), None).unwrap();
        assert_eq!(selected(&french, &en).unwrap(), &original[0]);
        assert_eq!(selected(&french, &fr).unwrap().description, None);
        let french = upsert(
            &french,
            fr.clone(),
            "Français".into(),
            Some(Some("".into())),
        )
        .unwrap();
        let french = upsert(&french, fr.clone(), "Autre".into(), None).unwrap();
        assert_eq!(
            selected(&french, &fr).unwrap().description.as_deref(),
            Some("")
        );
        let cleared = upsert(&french, fr.clone(), "Autre".into(), Some(None)).unwrap();
        assert_eq!(selected(&cleared, &fr).unwrap().description, None);
        let unknown = upsert(&cleared, None, "Unknown".into(), Some(Some(" \t\n".into()))).unwrap();
        assert_eq!(
            unknown
                .iter()
                .map(|entry| entry.language.as_deref())
                .collect::<Vec<_>>(),
            [None, Some("en-US"), Some("fr")]
        );
        assert_eq!(
            serde_json::from_value::<Content>(serde_json::to_value(&unknown[0]).unwrap()).unwrap(),
            unknown[0]
        );
    }

    #[test]
    fn legacy_rekey_replaces_collisions_without_manufacturing_translations() {
        let en = Some("en-US".into());
        let fr = Some("fr".into());
        let entries = upsert(
            &singleton("Old".into(), en.clone(), Some("Exact".into())),
            fr.clone(),
            "French".into(),
            Some(None),
        )
        .unwrap();
        let moved = legacy_language(&entries, &en, fr.clone()).unwrap();
        assert_eq!(
            moved,
            singleton("Old".into(), fr.clone(), Some("Exact".into()))
        );
        assert_eq!(
            legacy_title(&moved, &fr, "New".into()).unwrap()[0]
                .description
                .as_deref(),
            Some("Exact")
        );
        assert_eq!(
            legacy_description(&moved, &fr, None).unwrap()[0].title,
            "Old"
        );
        assert_eq!(legacy_language(&moved, &fr, fr.clone()).unwrap(), moved);
    }

    #[test]
    fn guards_reject_invalid_lists_and_shapes_without_mutation() {
        let original = singleton("Valid".into(), None, None);
        assert!(guard(&[], &None).is_err());
        assert!(guard(&[original[0].clone(), original[0].clone()], &None).is_err());
        assert!(guard(&original, &Some("en".into())).is_err());
        for value in [
            json!(null),
            json!([]),
            json!({"title":"T"}),
            json!({"language":null,"title":"T","description":null,"extra":1}),
        ] {
            assert!(serde_json::from_value::<Content>(value).is_err());
        }
        for (language, title, description) in [
            (Some("en_US".into()), "T", None),
            (None, " ", None),
            (None, "T", Some("bad\0".into())),
        ] {
            assert!(upsert(&original, language, title.into(), Some(description)).is_err());
        }
        assert_eq!(original, singleton("Valid".into(), None, None));
    }
}
