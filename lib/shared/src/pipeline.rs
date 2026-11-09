//! Pipeline data containers adapted from bodul_shared 0.1.1.

#[derive(Debug, Clone)]
pub struct SitemapConfig {
    pub sitemap_url: Vec<String>,
}

pub struct NormalizedData {
    pub document_json: Option<String>,
    pub recommendation_offers: Vec<String>,
    pub upsell_offers: Vec<String>,
}

impl NormalizedData {
    pub fn just_document_json(document_json: impl Into<Option<String>>) -> Self {
        NormalizedData {
            document_json: document_json.into(),
            recommendation_offers: vec![],
            upsell_offers: vec![],
        }
    }
}
