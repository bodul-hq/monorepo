//! Parses sitemap XML into an intermediate shape.
//!
//! A sitemap document is either a `<sitemapindex>` (references to child
//! sitemaps) or a `<urlset>` (page entries). This module turns the raw XML into
//! [`Parsed`]; fetching the children and assembling the tree is up to the
//! caller.

use std::str::FromStr;

use crate::document::{ChangeFrequency, SitemapImage, SitemapUrl};
use chrono::{DateTime, NaiveDate, TimeZone, Utc};
use quick_xml::Reader;
use quick_xml::escape;
use quick_xml::events::{BytesRef, Event};

#[derive(Debug, thiserror::Error, Clone)]
pub enum SitemapParseError {
    #[error("XML error: {0}")]
    Xml(#[from] quick_xml::Error),

    #[error("XML escape error: {0}")]
    Escape(#[from] quick_xml::escape::EscapeError),

    #[error("encoding error: {0}")]
    Encoding(#[from] quick_xml::encoding::EncodingError),

    #[error("unknown XML entity: &{0};")]
    UnknownEntity(String),

    #[error("unrecognized sitemap document (no <sitemapindex> or <urlset> root)")]
    UnrecognizedDocument,
}

/// The two kinds of sitemap document.
#[derive(Debug, PartialEq)]
pub enum Parsed {
    /// A `<sitemapindex>`: references to child sitemap documents.
    Index(Vec<ChildRef>),
    /// A `<urlset>`: page entries.
    UrlSet(Vec<SitemapUrl>),
}

/// A child sitemap referenced by an index.
#[derive(Debug, Clone, PartialEq)]
pub struct ChildRef {
    pub location: String,
    pub last_modified: Option<DateTime<Utc>>,
}

/// Parses sitemap XML, returning the index references or the URL entries.
pub fn parse(xml: &str, source: &str) -> Result<Parsed, SitemapParseError> {
    let mut reader = Reader::from_str(xml);

    let mut mode: Option<Mode> = None;
    let mut children: Vec<ChildRef> = Vec::new();
    let mut urls: Vec<SitemapUrl> = Vec::new();

    let mut child: Option<ChildBuilder> = None;
    let mut url: Option<UrlBuilder> = None;
    let mut image: Option<ImageBuilder> = None;

    let mut leaf: Option<String> = None;
    let mut text = String::new();
    let mut root_seen = false;

    loop {
        match reader.read_event()? {
            Event::Start(event) => match event.local_name().as_ref() {
                "sitemapindex" => {
                    if !root_seen {
                        mode = Some(Mode::Index);
                        root_seen = true;
                    }
                }
                "urlset" => {
                    if !root_seen {
                        mode = Some(Mode::UrlSet);
                        root_seen = true;
                    }
                }
                _ if !root_seen => return Err(SitemapParseError::UnrecognizedDocument),
                "sitemap" => child = Some(ChildBuilder::default()),
                "url" => url = Some(UrlBuilder::default()),
                "image" => image = Some(ImageBuilder::default()),
                "loc" | "lastmod" | "changefreq" | "priority" | "title" | "caption" => {
                    leaf = Some(event.local_name().as_ref().to_string());
                    text.clear();
                }
                _ => {}
            },
            Event::Text(event) if leaf.is_some() => {
                text.push_str(&escape::unescape(event.as_ref())?);
            }
            Event::CData(event) if leaf.is_some() => {
                text.push_str(event.as_ref());
            }
            // quick-xml emits entity references (e.g. `&amp;` in a `<loc>`) as
            // their own event, separate from the surrounding text.
            Event::GeneralRef(event) if leaf.is_some() => {
                text.push(resolve_entity(&event)?);
            }
            Event::End(event) => match event.local_name().as_ref() {
                "sitemap" => {
                    if let Some(builder) = child.take()
                        && let Some(location) = builder.location
                    {
                        let child_ref = ChildRef {
                            location,
                            last_modified: builder.last_modified,
                        };
                        children.push(child_ref);
                    }
                }
                "url" => {
                    if let Some(builder) = url.take()
                        && let Some(entry) = builder.build(source)
                    {
                        urls.push(entry);
                    }
                }
                "image" => {
                    if let (Some(builder), Some(parent)) = (image.take(), url.as_mut())
                        && let Some(location) = builder.location
                        && !location.is_empty()
                    {
                        let mut entry = SitemapImage::new(location);
                        entry.title = builder.title;
                        entry.caption = builder.caption;
                        parent.images.push(entry);
                    }
                }
                name if leaf.as_deref() == Some(name) => {
                    let value = text.trim().to_string();
                    assign(name, value, child.as_mut(), url.as_mut(), image.as_mut());
                    leaf = None;
                    text.clear();
                }
                _ => {}
            },
            Event::Eof => break,
            _ => {}
        }
    }

    match mode {
        Some(Mode::Index) => Ok(Parsed::Index(children)),
        Some(Mode::UrlSet) => Ok(Parsed::UrlSet(urls)),
        None => Err(SitemapParseError::UnrecognizedDocument),
    }
}

enum Mode {
    Index,
    UrlSet,
}

#[derive(Default)]
struct ChildBuilder {
    location: Option<String>,
    last_modified: Option<DateTime<Utc>>,
}

#[derive(Default)]
struct UrlBuilder {
    location: Option<String>,
    last_modified: Option<DateTime<Utc>>,
    change_frequency: Option<ChangeFrequency>,
    priority: Option<f32>,
    images: Vec<SitemapImage>,
}

#[derive(Default)]
struct ImageBuilder {
    location: Option<String>,
    title: Option<String>,
    caption: Option<String>,
}

impl UrlBuilder {
    fn build(self, source: &str) -> Option<SitemapUrl> {
        let location = self.location.filter(|location| !location.is_empty())?;

        let entry = SitemapUrl {
            location,
            source: source.to_string(),
            last_modified: self.last_modified,
            change_frequency: self.change_frequency,
            priority: self.priority,
            images: self.images,
        };

        Some(entry)
    }
}

/// Assigns a leaf element's text to the currently open builder. Image fields win
/// when an `<image:image>` block is open, then URL fields, then index entries.
fn assign(
    name: &str,
    value: String,
    child: Option<&mut ChildBuilder>,
    url: Option<&mut UrlBuilder>,
    image: Option<&mut ImageBuilder>,
) {
    if let Some(image) = image {
        match name {
            "loc" => image.location = Some(value),
            "title" => image.title = Some(value),
            "caption" => image.caption = Some(value),
            _ => {}
        }
        return;
    }
    if let Some(url) = url {
        match name {
            "loc" => url.location = Some(value),
            "lastmod" => url.last_modified = parse_lastmod(&value),
            "changefreq" => url.change_frequency = ChangeFrequency::from_str(&value).ok(),
            "priority" => {
                url.priority = value
                    .parse::<f32>()
                    .ok()
                    .filter(|priority| (0.0..=1.0).contains(priority));
            }
            _ => {}
        }
        return;
    }
    if let Some(child) = child {
        match name {
            "loc" => child.location = Some(value),
            "lastmod" => child.last_modified = parse_lastmod(&value),
            _ => {}
        }
    }
}

/// Resolves an XML entity reference to its character: numeric refs (`&#38;`,
/// `&#x26;`) via quick-xml, plus the five predefined named entities. Any other
/// name is an error, so a URL is never silently corrupted.
fn resolve_entity(reference: &BytesRef) -> Result<char, SitemapParseError> {
    if let Some(character) = reference.resolve_char_ref()? {
        return Ok(character);
    }
    match &**reference {
        "amp" => Ok('&'),
        "lt" => Ok('<'),
        "gt" => Ok('>'),
        "quot" => Ok('"'),
        "apos" => Ok('\''),
        unknown => Err(SitemapParseError::UnknownEntity(unknown.to_string())),
    }
}

/// Parses a `<lastmod>` value best-effort: RFC 3339 first, then a bare
/// `YYYY-MM-DD` date at midnight UTC; an unparseable value yields `None` rather
/// than failing the whole document.
fn parse_lastmod(value: &str) -> Option<DateTime<Utc>> {
    let value = value.trim();
    if let Ok(parsed) = DateTime::parse_from_rfc3339(value) {
        return Some(parsed.with_timezone(&Utc));
    }
    if let Ok(date) = NaiveDate::parse_from_str(value, "%Y-%m-%d")
        && let Some(naive) = date.and_hms_opt(0, 0, 0)
    {
        return Some(Utc.from_utc_datetime(&naive));
    }
    None
}

#[cfg(test)]
mod tests {
    use super::{Parsed, parse};
    use crate::document::ChangeFrequency;

    #[test]
    fn parses_index_with_lastmod() {
        let xml = r#"<sitemapindex>
            <sitemap>
                <loc>https://minisforumpc.eu/sitemap_products_1.xml</loc>
                <lastmod>2024-05-01</lastmod>
            </sitemap>
            <sitemap><loc>https://minisforumpc.eu/sitemap_collections_1.xml</loc></sitemap>
        </sitemapindex>"#;

        match parse(xml, "fixture.xml").expect("parses") {
            Parsed::Index(children) => {
                assert_eq!(children.len(), 2);
                assert_eq!(
                    children[0].location,
                    "https://minisforumpc.eu/sitemap_products_1.xml"
                );
                assert!(children[0].last_modified.is_some());
                assert!(children[1].last_modified.is_none());
            }
            other => panic!("expected index, got {other:?}"),
        }
    }

    #[test]
    fn parses_urlset_with_image_and_fields() {
        let xml = r#"<urlset xmlns:image="http://www.google.com/schemas/sitemap-image/1.1">
            <url>
                <loc>https://minisforumpc.eu/products/um890</loc>
                <lastmod>2024-05-01T10:00:00+00:00</lastmod>
                <changefreq>daily</changefreq>
                <priority>0.8</priority>
                <image:image>
                    <image:loc>https://minisforumpc.eu/img/um890.jpg</image:loc>
                    <image:title>UM890 Pro</image:title>
                </image:image>
            </url>
        </urlset>"#;

        match parse(xml, "fixture.xml").expect("parses") {
            Parsed::UrlSet(urls) => {
                assert_eq!(urls.len(), 1);
                let entry = &urls[0];
                assert_eq!(entry.location, "https://minisforumpc.eu/products/um890");
                assert_eq!(entry.source, "fixture.xml");
                assert!(entry.last_modified.is_some());
                assert_eq!(entry.change_frequency, Some(ChangeFrequency::Daily));
                assert_eq!(entry.priority, Some(0.8));
                assert_eq!(entry.images.len(), 1);
                assert_eq!(
                    entry.images[0].location,
                    "https://minisforumpc.eu/img/um890.jpg"
                );
                assert_eq!(entry.images[0].title.as_deref(), Some("UM890 Pro"));
            }
            other => panic!("expected urlset, got {other:?}"),
        }
    }

    #[test]
    fn rejects_unrecognized_root() {
        assert!(parse("<html><body>nope</body></html>", "fixture.xml").is_err());
    }

    #[test]
    fn decodes_amp_entity_in_loc() {
        let xml = r#"<sitemapindex><sitemap>
            <loc>https://minisforumpc.eu/sitemap_products_1.xml?from=1&amp;to=2</loc>
        </sitemap></sitemapindex>"#;
        match parse(xml, "fixture.xml").expect("parses") {
            Parsed::Index(children) => assert_eq!(
                children[0].location,
                "https://minisforumpc.eu/sitemap_products_1.xml?from=1&to=2"
            ),
            other => panic!("expected index, got {other:?}"),
        }
    }

    #[test]
    fn parses_prefixed_elements_and_cdata() {
        let xml = r#"<sm:urlset xmlns:sm="http://www.sitemaps.org/schemas/sitemap/0.9">
            <sm:url><sm:loc><![CDATA[https://example.com/a?x=1&y=2]]></sm:loc></sm:url>
        </sm:urlset>"#;

        match parse(xml, "prefixed.xml").expect("parses") {
            Parsed::UrlSet(urls) => {
                assert_eq!(urls[0].location, "https://example.com/a?x=1&y=2");
                assert_eq!(urls[0].source, "prefixed.xml");
            }
            other => panic!("expected urlset, got {other:?}"),
        }
    }

    #[test]
    fn ignores_priority_outside_protocol_range() {
        let xml =
            r#"<urlset><url><loc>https://example.com/a</loc><priority>2</priority></url></urlset>"#;

        match parse(xml, "fixture.xml").expect("parses") {
            Parsed::UrlSet(urls) => assert_eq!(urls[0].priority, None),
            other => panic!("expected urlset, got {other:?}"),
        }
    }

    #[test]
    fn rejects_unknown_entities_instead_of_corrupting_urls() {
        let xml = r#"<urlset><url><loc>https://example.com/a&unknown;</loc></url></urlset>"#;

        assert!(parse(xml, "fixture.xml").is_err());
    }
}
