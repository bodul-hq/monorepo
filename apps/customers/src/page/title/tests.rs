use super::*;

#[test]
fn extracts_readable_text_without_reparsing_entities() {
    for (input, expected) in [
        ("<b>USB charger</b>", "USB charger"),
        ("USB<b>-C</b> charger", "USB-C charger"),
        ("<p>Fast</p><p>charger</p>", "Fast charger"),
        ("Fast<br>charger", "Fast charger"),
        ("A &amp; B&nbsp;charger", "A & B charger"),
        (
            "<div>Fast<span> USB</span><div>charger</div>today</div>",
            "Fast USB charger today",
        ),
        ("<ul><li>Fast<li>charger</ul>", "Fast charger"),
        (
            "<table><tr><td>USB</td><td>charger</td></tr></table>",
            "USB charger",
        ),
        ("<!doctype html>Charger<!-- note -->", "Charger"),
        (
            "Charger<script>alert(1)</script><style>p{}</style><template>hidden</template>",
            "Charger",
        ),
        (
            "<SCRIPT>bad</SCRIPT><STYLE>bad</STYLE><TEMPLATE><b>bad</b></TEMPLATE>",
            "Untitled offer",
        ),
        ("<b></b>", "Untitled offer"),
        (" \n\t&nbsp; ", "Untitled offer"),
        ("  USB\n \tcharger  ", "USB charger"),
        ("2 < 3", "2 < 3"),
        ("&lt;b&gt;literal&lt;/b&gt;", "<b>literal</b>"),
        ("&amp;lt;b&amp;gt;literal", "&lt;b&gt;literal"),
        ("&#85;&#x53;B &#38; charger", "USB & charger"),
        ("<b><i>USB charger", "USB charger"),
        ("<span title='a > b'>USB</span> charger", "USB charger"),
        (
            "<img src=x onerror=alert(1)>USB &quot;charger&quot;",
            "USB \"charger\"",
        ),
        (
            "<svg><script>bad</script><text>USB</text></svg> charger",
            "USB charger",
        ),
        ("<b>充電器 ž ₹</b>", "充電器 ž ₹"),
    ] {
        assert_eq!(display_title(input), expected, "{input}");
    }
}

#[test]
fn deeply_nested_markup_does_not_use_recursive_text_extraction() {
    let input = format!(
        "{}USB charger{}",
        "<span>".repeat(1024),
        "</span>".repeat(1024)
    );
    assert_eq!(display_title(&input), "USB charger");
}
