//! Exact URL validation shared by image producers and consumers.
use std::collections::HashSet;

pub fn is_valid(value: &str) -> bool {
    value.len() <= 2048
        && value.trim() == value
        && !value.chars().any(char::is_control)
        && url::Url::parse(value).is_ok_and(|url| {
            matches!(url.scheme(), "http" | "https")
                && url.has_host()
                && url.username().is_empty()
                && url.password().is_none()
                && !value.split_once("://").is_some_and(|(_, rest)| {
                    rest.split(['/', '?', '#'])
                        .next()
                        .unwrap_or("")
                        .contains('@')
                })
        })
}

/// Preserve source order and exact strings; count invalid URLs, not duplicates.
pub fn filter(images: Vec<String>) -> (Vec<String>, usize) {
    let mut seen = HashSet::new();
    let mut skipped = 0;
    let images = images
        .into_iter()
        .filter(|image| {
            if !is_valid(image) {
                skipped += 1;
                false
            } else {
                seen.insert(image.clone())
            }
        })
        .collect();
    (images, skipped)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn validates_without_rewriting_and_deduplicates_in_order() {
        let a = "https://shop.example/A?q=1&x=2#front";
        let b = "http://shop.example/B";
        let invalid = [
            "/relative",
            "ftp://shop.example/a",
            " https://shop.example/a",
            "https://shop.example/a\n",
            "https://user@shop.example/a",
            "https://@shop.example/a",
            "https://shop.example:bad/a",
            "",
        ];
        for value in invalid {
            assert!(!is_valid(value), "{value:?}");
        }
        assert!(!is_valid(&format!(
            "https://shop.example/{}",
            "a".repeat(2048)
        )));
        let (images, skipped) = filter([a, a, invalid[0], b, a].map(String::from).to_vec());
        assert_eq!(images, [a, b]);
        assert_eq!(skipped, 1);
    }
}
