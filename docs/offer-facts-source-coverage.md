# Offer facts source coverage

The 44 completed retailer structuralizers emit `was_price`, `availability`,
`item_condition`, and `content: {description}` for every variation. An absent
source fact is explicit `null`; it does not mean unavailable or new. Anker KR
and the Mi/WIP stubs do not have completed product structuralizers.

`was_price` is a source comparison price for that exact variant. It must use the
current price's currency and be strictly greater than the current amount. An
unpriced variant has no comparison price. The current price is unchanged.
Ugreen KR/NL/UK/US additionally read explicit schema.org `ListPrice` and
`StrikethroughPrice` specifications; conflicting valid comparisons yield null.
Sale prices, discount codes, price history, and sibling prices are not comparisons.

Purchase flags (`available`, `available_for_sale`, cart-button state) map to
`available`/`unavailable`. Explicit schema.org stock states map to
`in_stock`/`out_of_stock`/`sold_out`; the existing typed source readers keep their
validation rules. Shared schema.org constructors accept both HTTP and HTTPS.
Other accepted source strings retain their exact text under `other`. A purchase
flag can be true while inventory is not currently in stock.

Descriptions come only from the already-normalized primary product's
`description`. Existing text, HTML, empty strings, and whitespace remain exact.
There is no fallback to metadata, sibling islands, or DOM content. Title and
images keep their existing flat fields and source choices.

## Sources by storefront

“Record” below means a variant-specific Shopify record or its normalized value.
“Matched offer” means the primary selling offer, joined by the existing variant
identity or a unique SKU. An unscoped single offer supplies stock or condition
only when the product has one variant.

| Storefronts | Comparison price | Availability | Item condition | Description |
| --- | --- | --- | --- | --- |
| Anker AU, COM, EU, FR, NZ, PL, VN | Record compare-at | Matched offer; record purchase flag otherwise | Matched offer only | Primary product |
| Anker CA, DE, UK | Record compare-at | Single-variant offer; record purchase flag otherwise | Unknown | Primary product |
| Anker Japan | Record compare-at | Offer pinned to variant ID; record purchase flag without an offer | Unknown | Primary product |
| Anker Italy, Nordics | Record compare-at | Record purchase flag | Unknown | Primary product |
| Minisforum AU, CA, EU, FR, HK, JP, UK | Record compare-at | Matched theme offer; record purchase flag without an offer | Unknown | Primary product |
| Minisforum KR | Selected variant compare-at only | Matched offer, or unknown without one | Unknown | Primary product |
| Minisforum RU | Record compare-at | Record purchase flag | Unknown | Unknown |
| Minisforum US | Record compare-at | Single-variant aggregate offer; record purchase flag otherwise | Single-variant aggregate offer only | Primary product |
| Ugreen CA | Record compare-at | Unique matched offer; record purchase flag otherwise | Unique matched offer only | Primary product |
| Ugreen COM | Unknown | Unknown | Unknown | Unknown |
| Ugreen DE | Selected variant compare-at only | Selected cart/picker flag only | Unknown | Unknown |
| Ugreen EU, FR | Normalized variant compare-at, including selected automatic-discount comparison | Matched offer stock | Unknown | Unknown |
| Ugreen IN | Record compare-at | Matched theme offer; record purchase flag for older normalized documents | Unknown | Primary product |
| Ugreen JP | Normalized variant compare-at | Matched offer stock | Unknown | Primary product |
| Ugreen KR, NL, UK, US | Selected compare-at and scoped list/strikethrough specifications | Matched selling offer | Matched selling offer when stated | Primary product |
| Ugreen NAS, NAS EU, NAS FR, NAS IT, NAS UK | Record compare-at | Record purchase flag | Unknown | Unknown |
| Ugreen NAS AU, NAS CA, NAS DE, NAS ES | Normalized variant compare-at | Picker/cart purchase flag | Unknown | Unknown |
| Ugreen NAS JP | Unknown | Matched offer stock | Unknown | Primary product |

Other publisher, review, feed, and SEO islands excluded by the existing source
selection rules remain excluded. Their product summaries or condition labels
are not used to fill unknown fields. In particular, Ugreen COM analytics are
not an offer-availability source, and a product name containing “refurbished”
does not infer item condition.

## Normalized-document compatibility

Restored source fields are optional with serde defaults, so saved normalized
JSON that predates them still reads. Older primary records keep their original
purchase/stock semantics; unsupported condition remains null. Existing
retailer-specific `available`, `compare_at_price`, `condition`, and offer-price
fields remain present where they existed. Historical exported artifacts are
not rewritten by these source readers.

AU and Japanese Anker schema availability now survives normalization. Primary
Minisforum theme offers and Ugreen IN retain their explicit availability beside
the Shopify purchase flag. Ugreen CA retains uniquely matched offer stock and
condition. Ugreen KR/NL/UK/US retain `sold_out` instead of reducing it to false.
Minisforum US retains aggregate stock/condition only for a single variant.

## Validation

Portable tests cover same-currency comparisons, missing/equal/lower prices,
conflicting explicit comparisons, selected-variant scope, duplicate SKU
ambiguity, purchase flags without stock evidence, aggregate-offer scope,
unknown conditions, and exact descriptions. Existing complete output fixtures
also verify that title, current price, identifiers, images, and URLs are unchanged.

The read-only facade survey is
`cargo test -p retailer_data_extractor --test offer_facts_corpus -- --ignored --nocapture`
from `lib/`. This opt-in test uses four bounded storefront workers and is ignored
by default because the external HTML corpus is several gigabytes. Its
`OFFER_FACTS` rows report source pages, successful pages,
variations, nonnull comparison prices, nonnull availability, nonnull conditions,
descriptions, and pipeline failures. `OFFER_FACTS_ERROR` rows identify source
schema failures separately. It never modifies the source corpus.


## Recorded representative survey (2026-10-26)

The deterministic sample below passed in 4.21 seconds: the first source filename
in sorted order from each of all 44 completed storefronts. Run it with:

```sh
OFFER_FACTS_MAX_PAGES_PER_STOREFRONT=1 cargo test -p retailer_data_extractor --test offer_facts_corpus -- --ignored --nocapture
```

Omit the environment variable for the full corpus. The sample contains 44
successful pages and 82 variations. Nonnull counts are 27 comparison prices,
81 availability values, 10 item conditions, and 49 descriptions; corresponding
null counts are 55, 1, 72, and 33. These are sampled observations, not an estimate
of full-storefront completeness. Every emitted status and content object passed
its shared guard.

Each row covers one successful source page. The fact columns count nonnull
variation fields; subtract from the variation count to obtain null counts.

| Storefront | Variations | was_price | Availability | Item condition | Description |
| --- | ---: | ---: | ---: | ---: | ---: |
| ankerau | 3 | 0 | 3 | 1 | 3 |
| ankerca | 1 | 0 | 1 | 0 | 1 |
| ankercom | 2 | 1 | 2 | 1 | 2 |
| ankerde | 1 | 0 | 1 | 0 | 1 |
| ankereu | 1 | 0 | 1 | 1 | 1 |
| ankerfr | 1 | 0 | 1 | 1 | 0 |
| ankeritalycom | 3 | 0 | 3 | 0 | 3 |
| ankerjapancom | 1 | 0 | 1 | 0 | 1 |
| ankernordicscom | 1 | 0 | 1 | 0 | 1 |
| ankernz | 2 | 0 | 2 | 1 | 2 |
| ankerpl | 1 | 0 | 1 | 1 | 1 |
| ankeruk | 3 | 0 | 3 | 0 | 3 |
| ankervn | 5 | 0 | 5 | 1 | 5 |
| minisforumau | 1 | 0 | 1 | 0 | 1 |
| minisforumca | 1 | 0 | 1 | 0 | 0 |
| minisforumeu | 5 | 5 | 5 | 0 | 5 |
| minisforumfr | 7 | 0 | 7 | 0 | 0 |
| minisforumhk | 3 | 2 | 3 | 0 | 3 |
| minisforumjp | 4 | 3 | 4 | 0 | 4 |
| minisforumkr | 1 | 0 | 1 | 0 | 0 |
| minisforumru | 4 | 0 | 4 | 0 | 0 |
| minisforumuk | 2 | 2 | 2 | 0 | 2 |
| minisforumus | 1 | 0 | 1 | 1 | 1 |
| ugreenca | 2 | 2 | 2 | 2 | 2 |
| ugreencom | 1 | 0 | 0 | 0 | 0 |
| ugreende | 1 | 1 | 1 | 0 | 0 |
| ugreeneu | 3 | 1 | 3 | 0 | 0 |
| ugreenfr | 1 | 0 | 1 | 0 | 0 |
| ugreenin | 1 | 0 | 1 | 0 | 1 |
| ugreenjp | 1 | 0 | 1 | 0 | 1 |
| ugreenkr | 1 | 0 | 1 | 0 | 1 |
| ugreennas | 3 | 3 | 3 | 0 | 0 |
| ugreennasau | 1 | 0 | 1 | 0 | 0 |
| ugreennasca | 1 | 1 | 1 | 0 | 0 |
| ugreennasde | 2 | 2 | 2 | 0 | 0 |
| ugreennases | 1 | 0 | 1 | 0 | 0 |
| ugreennaseu | 1 | 1 | 1 | 0 | 0 |
| ugreennasfr | 1 | 0 | 1 | 0 | 0 |
| ugreennasit | 1 | 0 | 1 | 0 | 0 |
| ugreennasjp | 1 | 0 | 1 | 0 | 1 |
| ugreennasuk | 2 | 2 | 2 | 0 | 0 |
| ugreennl | 1 | 1 | 1 | 0 | 1 |
| ugreenuk | 1 | 0 | 1 | 0 | 1 |
| ugreenus | 1 | 0 | 1 | 0 | 1 |

The interrupted full-corpus scan also completed entire storefronts before it
was stopped; the broad family regression suites provide the remaining
compatibility evidence. The 44-row sample above is the completed, reproducible
facade acceptance run, and no full-corpus success is claimed.

Known unrelated source-schema failures observed in the longer scan:

| Storefront / source file | Failure |
| --- | --- |
| `ankercom/products-a2628.html` | The strict Next.js source model does not recognize `filterPrice`. |
| `ugreennasca/products-worry-free-purchase.html` | A floating-point source value fails under the facade's `arbitrary_precision` JSON feature before purchase-protection exclusion. |
| `ugreennasau/products-worry-free-purchase.html` | The same floating-point source parsing issue. |

No source HTML or saved extraction/normalization files were changed. Portable
and established family checks passed: 577 Anker tests, 468 Minisforum tests,
and 783 Ugreen tests followed by all 57 NAS pipeline tests after updating four
complete-output expectations. The extractor's 15 portable tests, scoped
formatting, and clippy with `-D warnings` also passed.
