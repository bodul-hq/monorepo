//! Reads the locale and the handle from the product URL.
//!
//! `eu.ugreen.com` serves one shop under 65 locale prefixes and the unprefixed root:
//! `https://eu.ugreen.com[/<locale>]/products/<handle>`, where `<locale>` is a language (`de`)
//! or a language and a country (`en-se`).

/// The storefront's origin, which every product URL in the corpus is on.
const ORIGIN: &str = "https://eu.ugreen.com/";

#[derive(Debug, Clone, PartialEq)]
pub(super) struct PageUrl {
    /// `None` on the unprefixed root.
    pub locale: Option<String>,
    /// Percent-decoded, as the analytics state it.
    pub handle: String,
}

impl PageUrl {
    /// `None` where the URL is not a product page on the storefront's origin.
    pub(super) fn parse(url: &str) -> Option<PageUrl> {
        let path = url.strip_prefix(ORIGIN)?;
        let (locale, rest) = match path.split_once('/') {
            Some((first, rest)) if first != "products" => (Some(first), rest),
            _ => (None, path),
        };
        if locale.is_some_and(|locale| !is_locale(locale)) {
            return None;
        }

        let handle = rest.strip_prefix("products/")?;
        if handle.is_empty() || handle.contains(['/', '?', '#']) {
            return None;
        }

        Some(PageUrl {
            locale: locale.map(str::to_string),
            handle: percent_decoded(handle)?,
        })
    }
}

// `de` or `de-at`, lower case, as Shopify's market paths render them.
fn is_locale(segment: &str) -> bool {
    let lower = |part: &str| part.len() == 2 && part.bytes().all(|b| b.is_ascii_lowercase());
    match segment.split_once('-') {
        Some((language, country)) => lower(language) && lower(country),
        None => lower(segment),
    }
}

// A non-ASCII handle reaches the URL as escaped UTF-8 (`%D0%BA`), in upper case on the island
// and in lower case in the canonical link.
fn percent_decoded(value: &str) -> Option<String> {
    let bytes = value.as_bytes();
    let mut decoded = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while let Some(&byte) = bytes.get(index) {
        if byte == b'%' {
            let hex = std::str::from_utf8(bytes.get(index + 1..index + 3)?).ok()?;
            decoded.push(u8::from_str_radix(hex, 16).ok()?);
            index += 3;
        } else {
            decoded.push(byte);
            index += 1;
        }
    }
    String::from_utf8(decoded).ok()
}

#[cfg(test)]
mod tests {
    use super::PageUrl;

    fn parsed(url: &str) -> Option<(Option<String>, String)> {
        PageUrl::parse(url).map(|page| (page.locale, page.handle))
    }

    #[test]
    fn a_prefixed_url_names_its_locale() {
        assert_eq!(
            parsed("https://eu.ugreen.com/de-at/products/pb561"),
            Some((Some("de-at".to_string()), "pb561".to_string()))
        );
        assert_eq!(
            parsed("https://eu.ugreen.com/sv/products/pb561"),
            Some((Some("sv".to_string()), "pb561".to_string()))
        );
    }

    #[test]
    fn the_root_has_no_locale() {
        assert_eq!(
            parsed("https://eu.ugreen.com/products/pb561"),
            Some((None, "pb561".to_string()))
        );
    }

    #[test]
    fn an_escaped_handle_is_decoded_in_either_case() {
        let handle = Some((Some("bg-bg".to_string()), "кутия-25053".to_string()));
        assert_eq!(
            parsed("https://eu.ugreen.com/bg-bg/products/%D0%BA%D1%83%D1%82%D0%B8%D1%8F-25053"),
            handle
        );
        assert_eq!(
            parsed("https://eu.ugreen.com/bg-bg/products/%d0%ba%d1%83%d1%82%d0%b8%d1%8f-25053"),
            handle
        );
        assert_eq!(parsed("https://eu.ugreen.com/products/a%2"), None);
        assert_eq!(parsed("https://eu.ugreen.com/products/a%FF"), None);
    }

    #[test]
    fn anything_but_a_product_page_on_the_origin_is_refused() {
        for url in [
            "https://de.ugreen.com/products/pb561",
            "https://eu.ugreen.com/collections/pb561",
            "https://eu.ugreen.com/DE/products/pb561",
            "https://eu.ugreen.com/deu/products/pb561",
            "https://eu.ugreen.com/de/at/products/pb561",
            "https://eu.ugreen.com/products/",
            "https://eu.ugreen.com/products/pb561?variant=1",
        ] {
            assert_eq!(parsed(url), None, "{url}");
        }
    }
}
