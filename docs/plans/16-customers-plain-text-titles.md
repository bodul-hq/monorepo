# Item 16: Customers plain-text offer titles

Status: completed and verified on 2026-10-26.

## Goal

Display readable title text without HTML tags in the customers app. Apply the
same cleaned title to the visible heading, image alternative text and the
accessible “View offer” description. Preserve the original title in API data
and storage; this is a presentation change.

## Starting behavior

[Customers rendering](../../apps/customers/src/page.rs) escapes the raw title
separately in the heading, image `alt` and offer-link description. This prevents
markup execution, but a title such as `<b>USB charger</b>` visibly includes tags.
The heading substitutes “Untitled offer” only when the original string is blank.
Existing [rendering tests](../../apps/customers/src/page/tests.rs) assert escaped
raw markup and must be revised to assert the new plain-text behavior.

## Text contract

- Parse the title as an HTML fragment and collect text in document order.
  Remove element markup, attributes, comments and declarations.
- Exclude `script`, `style` and `template` subtrees, including their content.
  Embedded executable/style text must not become a product title.
- Preserve inline text adjacency (`USB<b>-C</b>` becomes `USB-C`). Treat `br`
  and block boundaries as word separators so adjacent paragraphs do not merge.
  Collapse runs of whitespace to one space and trim the result.
- Decode character references once as part of parsing: `A &amp; B` becomes
  `A & B`. Do not repeatedly decode or reparse extracted text. Encoded literal
  angle brackets remain text, rather than being treated as a second HTML layer.
- Preserve Unicode product text and ordinary comparisons such as `2 < 3`.
  Let the parser recover malformed HTML deterministically; do not remove text
  using a regular expression or a scanner that deletes everything between
  angle brackets.
- If the extracted, trimmed title is empty, use “Untitled offer” everywhere
  that the title is displayed, including image and link descriptions.
- HTML-escape the final plain text at every output boundary. Text extraction
  does not replace escaping or the existing URL validation.

## Implementation steps

1. Select an HTML fragment parser already used in the repository's dependency
   graph where practical. `apps/Cargo.lock` currently contains `scraper` 0.27.0
   and `html5ever` transitively; inspect their available APIs and features before
   declaring a direct dependency. A transitive lockfile entry alone does not
   make a crate available to customers. Avoid pulling a retailer integration
   into the app solely to obtain text extraction.
2. Implement a private title-to-text helper with explicit subtree exclusion and
   whitespace handling. Keep parser details out of the page layout code.
3. Compute the display title once per offer, then pass it to the heading,
   image renderer and link description. Use the same fallback at all three
   locations, without mutating the deserialized offer.
4. Replace raw-title expectations and extend focused rendering tests. Preserve
   escaping assertions for other fields and for the extracted text itself.
5. Update the customers README to explain display cleanup. Declare any shared
   dependency in `apps/Cargo.toml` only if multiple apps use it; otherwise use
   the customers manifest. Keep dependency entries alphabetically sorted.

## Acceptance examples and tests

| Input | Display text |
| --- | --- |
| `<b>USB charger</b>` | `USB charger` |
| `USB<b>-C</b> charger` | `USB-C charger` |
| `<p>Fast</p><p>charger</p>` | `Fast charger` |
| `Fast<br>charger` | `Fast charger` |
| `A &amp; B&nbsp;charger` | `A & B charger` |
| `Charger<!-- note --><script>alert(1)</script>` | `Charger` |
| `<b></b>` or only excluded content | `Untitled offer` |
| `2 < 3` | `2 < 3` |
| `&lt;b&gt;literal&lt;/b&gt;` | `<b>literal</b>` as escaped text |

Also cover nested and unclosed tags, attributes containing `>`, numeric and
double-encoded character references, style/template content, quotes, Unicode,
plain-text whitespace, and malicious markup. Verify the visible heading, alt
text and link description agree, with and without an image and with an unsafe
store link. Confirm no input can inject tags or break an attribute.

Run `make fmt` then `make ci` in `apps/customers`. If Cargo dependencies or
`apps/Cargo.lock` change, also run `make -C apps audit` and report new advisories;
run apps-wide CI if shared dependency declarations change. No new route is
introduced, but existing page smoke tests must remain passing.

Serve with `CUSTOMER_FACING_URL=<local fixture> PORT=<port> make serve` and verify
HTTP 200 for a mixed-title fixture page. Review desktop and 320px/390px layouts,
image alternative text and link names, ensuring markup is absent as specified
and long cleaned titles still fit. Preserve new-tab behavior, retailer badges
and price/action alignment. Stop the app and fixture afterward.

## Dependencies and completion

Builds on completed items 9 and 11–14 and is independent of
[currency formatting](15-customers-currency-formatting.md). Implement in its
own commit and verify the combined layout after both items are finished.
There is no API/schema/event change, historical rewrite or queue work, and no
new architecture decision is required. Mark TODO and the plan index complete
only after recording actual test and live-page evidence here.

## Completion evidence

- Added the private `page::title` helper using `scraper` 0.27.0 without default
  features. It traverses parsed fragments iteratively, excludes script/style/
  template subtrees, separates block text, normalizes whitespace and supplies
  the empty-title fallback. Each offer computes one display title and escapes
  it independently in the heading, image alt and link description. API data is
  left intact.
- `make fmt` and `make ci` in `apps/customers` passed: 21 tests. Coverage includes
  all acceptance examples, malformed/nested markup, deep nesting, character
  references decoded once, Unicode, attribute escaping, fallback consistency,
  missing images and unsafe store links. Log: `/tmp/customer-display-ci.log`.
- The live 31-offer fixture returned HTTP 200 and correct 30/1/empty pagination.
  Browser assertions verified cleaned heading/alt/link agreement, empty-title
  fallbacks and absence of injected elements. Combined responsive layout and
  navigation checks are detailed in item 15. All temporary servers were stopped.
- `make -C apps audit` ran. It reports existing vulnerabilities in
  `rustls-webpki` 0.103.15 (RUSTSEC-2026-0382) and `webpki-root-certs` 1.0.9
  (RUSTSEC-2026-0383), plus existing warnings for fxhash (RUSTSEC-2025-0057),
  kuchiki (RUSTSEC-2023-0019), rand (RUSTSEC-2026-0097) and tokio
  (RUSTSEC-2026-0386). Audit is not clean; no new advisory was introduced.
  All locked package versions and checksums match the pre-change lockfile;
  its only change adds customers' direct dependency on the existing scraper
  package. Log: `/tmp/customer-display-audit.log`.
