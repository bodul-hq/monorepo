use html_extractor::io::{Architecture, Attr, collection, particle};

pub fn shopify_app_router_architecture() -> Architecture {
    let structures = vec![
        collection(
            "script[type=\"application/ld+json\"]",
            "ld-json",
            vec![particle("script", "", vec![Attr::json("", "data")])],
        ),
        // Must precede any `trash("script")` a caller adds: order is execution order, and trash
        // detaches its matches.
        collection(
            "script:not([src]):not([type]):not([id])",
            "flight",
            vec![particle("script", "", vec![Attr::inner_text("text")])],
        ),
        // Keep this last: valueless particles detach their matched element, so the document's
        // descendant segments must be processed before the root <html> element.
        particle("html[lang]", "", vec![Attr::pick_as("lang", "locale")]),
    ];

    Architecture::new(structures)
}
