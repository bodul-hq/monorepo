pub mod graphql;
use ::shared::NormalizedData;
use html_extractor::io::Architecture;
use retailer_sauce::structuralized::StructuralizedData;

// API responses enter through extract_for; there are no HTML islands to select.
pub fn architecture() -> Architecture {
    Architecture::new(vec![])
}
pub fn normalize(json: &str) -> Result<NormalizedData, String> {
    graphql::Store.normalize(json)
}
pub fn structuralize(json: &str) -> Result<StructuralizedData, String> {
    graphql::Store.structuralize(json)
}
