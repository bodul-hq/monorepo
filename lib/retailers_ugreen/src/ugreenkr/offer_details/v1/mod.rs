//! Korean Shopify theme: JSON-LD supplies offers in whole won; analytics prices are
//! hundredths of a won. The main section supplies the title, gallery and displayed prices.
use html_extractor::io::{Architecture, Attr, collection, particle, trash};
use html_extractor::serde_json;
pub mod normalized;
pub mod raw;
mod structuralize;

pub fn architecture() -> Architecture {
    let main = |selector: &str| format!(r#"section[id^="MainProduct-"] {selector}"#);
    Architecture::new(vec![
        trash("svg"),
        collection(
            r#"script[type="application/ld+json"]"#,
            "ld-json",
            vec![particle(
                "script",
                "",
                vec![Attr::all(), Attr::json("", "data")],
            )],
        ),
        particle(
            r#"link[href="https://monorail-edge.shopifysvc.com"] + script + script + script + script"#,
            "analytics",
            vec![Attr::all(), Attr::javascript_vars("meta", "meta")],
        ),
        particle(
            r#"link[rel="canonical"]"#,
            "canonical",
            vec![Attr::standard("href")],
        ),
        particle(&main("h1"), "title", vec![Attr::inner_text("text")]),
        particle(
            &main(r#"input[name="id"]"#),
            "selected_variant_id",
            vec![Attr::standard("value")],
        ),
        particle(
            &main(r#"[id^="price-"] .price__sale .price-item--sale"#),
            "displayed_price",
            vec![Attr::inner_text("text")],
        ),
        particle(
            &main(r#"[id^="price-"] s.price-item--regular"#),
            "compare_price",
            vec![Attr::inner_text("text")],
        ),
        // Only the main slider, excluding the modal's duplicate gallery and thumbnails.
        collection(
            &main(".product__media-item.swiper-slide img"),
            "images",
            vec![particle(
                "img",
                "",
                vec![Attr::standard("src"), Attr::standard("alt")],
            )],
        ),
    ])
}

pub fn normalize(json: &str) -> Result<bodul_shared::NormalizedData, String> {
    let raw = serde_json::from_str(json).map_err(|e| e.to_string())?;
    let normalized =
        normalized::UgreenKrNormalizedDocument::from_raw(raw).map_err(|e| e.to_string())?;
    let json = serde_json::to_string(&normalized).map_err(|e| e.to_string())?;
    Ok(bodul_shared::NormalizedData::just_document_json(json))
}
#[cfg(test)]
mod tests;

/// Converts the normalized product into validated offer variations.
pub fn structuralize(
    json: &str,
) -> Result<retailer_sauce::structuralized::StructuralizedData, String> {
    let input = serde_json::from_str(json).map_err(|error| error.to_string())?;
    let document =
        structuralize::Document::from_normalized(input).map_err(|error| error.to_string())?;
    let json = serde_json::to_string(&document).map_err(|error| error.to_string())?;
    Ok(retailer_sauce::structuralized::StructuralizedData::just_document_json(json))
}
