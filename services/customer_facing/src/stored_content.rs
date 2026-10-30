//! Private projection envelope; upstream event content remains unchanged.
use retailer_sauce::offer_content::{self, Content};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Entry {
    #[serde(deserialize_with = "retailer_sauce::observation::required_nullable")]
    pub language: Option<String>,
    pub title: String,
    #[serde(deserialize_with = "retailer_sauce::observation::required_nullable")]
    pub description: Option<String>,
    pub url: String,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct StoredContent {
    #[serde(deserialize_with = "retailer_sauce::observation::required_nullable")]
    pub selected_language: Option<String>,
    pub entries: Vec<Entry>,
}
impl StoredContent {
    pub fn contents(&self) -> Vec<Content> {
        self.entries
            .iter()
            .map(|e| Content {
                language: e.language.clone(),
                title: e.title.clone(),
                description: e.description.clone(),
            })
            .collect()
    }
    pub fn guard(&self) -> Result<(), String> {
        offer_content::guard(&self.contents(), &self.selected_language)?;
        if self
            .entries
            .iter()
            .any(|e| e.url.len() > 2048 || e.url.contains('\0'))
        {
            return Err("invalid content URL".into());
        }
        Ok(())
    }
    pub fn selected(&self) -> Result<&Entry, String> {
        self.guard()?;
        self.entries
            .iter()
            .find(|e| e.language == self.selected_language)
            .ok_or_else(|| "selected content missing".into())
    }
    pub fn new(content: &[Content], language: &Option<String>, url: &str) -> Result<Self, String> {
        let result = Self {
            selected_language: language.clone(),
            entries: content
                .iter()
                .map(|e| Entry {
                    language: e.language.clone(),
                    title: e.title.clone(),
                    description: e.description.clone(),
                    url: url.into(),
                })
                .collect(),
        };
        result.guard()?;
        Ok(result)
    }
    pub fn reconcile(
        &self,
        content: &[Content],
        language: &Option<String>,
        renamed: bool,
    ) -> Result<Self, String> {
        let fallback = self.selected()?.url.clone();
        let mut result = Self::new(content, language, &fallback)?;
        for e in &mut result.entries {
            if renamed && e.language == *language {
                continue;
            }
            if let Some(old) = self.entries.iter().find(|old| old.language == e.language) {
                e.url = old.url.clone();
            }
        }
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn entry(language: Option<&str>, title: &str) -> Content {
        Content {
            language: language.map(str::to_owned),
            title: title.into(),
            description: None,
        }
    }
    #[test]
    fn reconciliation_keeps_urls_and_uses_selected_url_for_new_languages() {
        let mut stored = StoredContent::new(
            &[
                entry(None, "Unknown"),
                entry(Some("en"), "English"),
                entry(Some("fr"), "French"),
            ],
            &Some("en".into()),
            "https://example.com/en",
        )
        .unwrap();
        stored.entries[2].url = "https://example.com/fr".into();
        let next = stored
            .reconcile(
                &[
                    entry(Some("en"), "New English"),
                    entry(Some("fr"), "New French"),
                    entry(Some("de"), "German"),
                ],
                &Some("fr".into()),
                false,
            )
            .unwrap();
        assert_eq!(next.selected().unwrap().url, "https://example.com/fr");
        assert_eq!(next.entries[2].url, "https://example.com/en");
        let moved = stored
            .reconcile(
                &[entry(None, "Unknown"), entry(Some("fr"), "English")],
                &Some("fr".into()),
                true,
            )
            .unwrap();
        assert_eq!(moved.selected().unwrap().url, "https://example.com/en");
    }
    #[test]
    fn selection_and_urls_are_validated_without_normalization() {
        let url = "https://example.com/A?q=1#part";
        let mut stored = StoredContent::new(&[entry(None, "Title")], &None, url).unwrap();
        assert_eq!(stored.selected().unwrap().url, url);
        stored.selected_language = Some("fr".into());
        assert!(stored.guard().is_err());
        stored.selected_language = None;
        stored.entries.push(stored.entries[0].clone());
        assert!(stored.guard().is_err());
        assert!(StoredContent::new(&[entry(None, "Title")], &None, &"x".repeat(2049)).is_err());
        assert!(
            StoredContent::new(&[entry(None, "Title")], &None, "https://example.com/\0").is_err()
        );
        assert!(serde_json::from_str::<StoredContent>(r#"{"entries":[]}"#).is_err());
    }
}
