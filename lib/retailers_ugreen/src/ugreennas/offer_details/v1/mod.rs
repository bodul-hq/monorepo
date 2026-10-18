//! UGREEN NAS uses Shopify Liquid (the saved pages now live at ai.ugreen.com).
//! All 30 pages carry analytics, Apple Pay currency, and EcomSend's complete and selected
//! variant records. Their IDs, SKUs and prices agree. Prices are integer USD cents, including
//! the $30 preorder deposits; no future balance or promotional discount is inferred.
//! Product JSON-LD is absent on several pages and is not read. The EcomSend title supplies
//! the product title even where the DOM repeats its h1. The main media gallery supplies images,
//! excluding discount badges, thumbnails and recommendations. Missing SKUs/barcodes stay absent.

use html_extractor::io::{Architecture, Attr, collection, particle, trash};
use html_extractor::serde_json;

mod normalized;
mod raw;
mod structuralize;

pub fn architecture() -> Architecture {
    Architecture::new(vec![
        trash("svg"),
        particle(
            r#"link[rel="dns-prefetch"][href="https://monorail-edge.shopifysvc.com"] + script + script + script + script"#,
            "analytics",
            vec![Attr::all(), Attr::javascript_vars("meta", "meta")],
        ),
        particle(
            "script#apple-pay-shop-capabilities",
            "shop",
            vec![Attr::all(), Attr::json("", "data")],
        ),
        particle(
            "script#em_product_variants",
            "variants",
            vec![Attr::all(), Attr::json("", "data")],
        ),
        particle(
            "script#em_product_selected_or_first_available_variant",
            "selected_variant",
            vec![Attr::all(), Attr::json("", "data")],
        ),
        particle(
            "script#ecomsend_bis_product_title",
            "title",
            vec![Attr::all(), Attr::json("", "data")],
        ),
        particle(
            "link[rel=canonical]",
            "canonical",
            vec![Attr::standard("href")],
        ),
        collection(
            r#"media-gallery[id$="__main"] img.global-media-settings, media-gallery[id$="__main"] .product__modal-opener--image img"#,
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
    let document =
        normalized::UgreenNasNormalizedDocument::from_raw(raw).map_err(|e| e.to_string())?;
    Ok(bodul_shared::NormalizedData::just_document_json(
        serde_json::to_string(&document).map_err(|e| e.to_string())?,
    ))
}

pub fn structuralize(
    json: &str,
) -> Result<retailer_sauce::structuralized::StructuralizedData, String> {
    let normalized = serde_json::from_str(json).map_err(|e| e.to_string())?;
    let document =
        structuralize::UgreenNasDocument::from_normalized(normalized).map_err(|e| e.to_string())?;
    Ok(
        retailer_sauce::structuralized::StructuralizedData::just_document_json(
            serde_json::to_string(&document).map_err(|e| e.to_string())?,
        ),
    )
}

#[cfg(test)]
mod tests;
