use scraper::{Html, Node};

pub(super) fn display_title(title: &str) -> String {
    let fragment = Html::parse_fragment(title);
    let mut text = String::new();
    // Explicit traversal avoids recursion for deeply nested retailer markup.
    // None marks the end of a block, where adjacent words need a separator.
    let mut pending = vec![Some(fragment.tree.root())];
    while let Some(entry) = pending.pop() {
        let Some(node) = entry else {
            text.push(' ');
            continue;
        };
        match node.value() {
            Node::Text(value) => text.push_str(value),
            Node::Element(element) => {
                if matches!(element.name(), "script" | "style" | "template") {
                    continue;
                }
                if separates_words(element.name()) {
                    text.push(' ');
                    pending.push(None);
                }
            }
            _ => {}
        }
        pending.extend(node.children().rev().map(Some));
    }
    let text = text.split_whitespace().collect::<Vec<_>>().join(" ");
    if text.is_empty() {
        "Untitled offer".into()
    } else {
        text
    }
}

fn separates_words(name: &str) -> bool {
    matches!(
        name,
        "address"
            | "article"
            | "aside"
            | "blockquote"
            | "br"
            | "caption"
            | "dd"
            | "details"
            | "dialog"
            | "div"
            | "dl"
            | "dt"
            | "fieldset"
            | "figcaption"
            | "figure"
            | "footer"
            | "form"
            | "h1"
            | "h2"
            | "h3"
            | "h4"
            | "h5"
            | "h6"
            | "header"
            | "hgroup"
            | "hr"
            | "legend"
            | "li"
            | "main"
            | "menu"
            | "nav"
            | "ol"
            | "p"
            | "pre"
            | "section"
            | "summary"
            | "table"
            | "tbody"
            | "td"
            | "tfoot"
            | "th"
            | "thead"
            | "tr"
            | "ul"
    )
}

#[cfg(test)]
mod tests;
