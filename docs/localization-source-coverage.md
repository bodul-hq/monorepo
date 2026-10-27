# Source-backed localization coverage

Surveyed the saved corpus on 2026-10-26 through the shared extraction,
normalization and structuralization facade. No corpus files were modified.
All values come from `html[lang]`, or standalone top-level `Shopify.country`
and `Shopify.locale` literal assignments. Shopify language is only a fallback
when document language is absent. Payment configuration, Next.js default locale,
alternate links and widget settings are not accepted evidence. Equal-scope
conflicts produce unknown. ICU locale syntax validation preserves full source
tags; unsupported extlang/grandfathered forms are reported as unknown.

The `_localization_evidence` sidecar stores raw declarations, paths and diagnostics.
Known Apple Pay, Next.js routing/default locale, Klarna, external-script locale
and alternate-link candidates retain explicit rejection reasons. Dynamic Shopify
assignments invalidate the corresponding static declaration instead of guessing.
It is detached before strict retailer schemas are decoded, then reattached after
each stage; structuralization copies selected facts into each represented variation.
This avoids claiming the HTML root before retailer data islands are extracted.

| Storefront | Saved pages | Known market | Known language | Full pipelines | Errors |
| --- | ---: | ---: | ---: | ---: | ---: |
| ankerau | 3 | 0 | 3 | 3 | 0 |
| ankerca | 3 | 0 | 3 | 3 | 0 |
| ankercom | 3 | 0 | 3 | 3 | 0 |
| ankerde | 3 | 0 | 3 | 3 | 0 |
| ankereu | 3 | 0 | 3 | 3 | 0 |
| ankerfr | 3 | 0 | 3 | 3 | 0 |
| ankeritalycom | 3 | 0 | 3 | 3 | 0 |
| ankerjapancom | 3 | 3 | 3 | 3 | 0 |
| ankerkr | 3 | 0 | 3 | 0 | 0 |
| ankernordicscom | 3 | 0 | 3 | 3 | 0 |
| ankernz | 3 | 0 | 3 | 3 | 0 |
| ankerpl | 3 | 0 | 3 | 3 | 0 |
| ankeruk | 3 | 0 | 3 | 3 | 0 |
| ankervn | 3 | 0 | 3 | 3 | 0 |
| minisforumau | 3 | 3 | 3 | 3 | 0 |
| minisforumca | 3 | 3 | 3 | 3 | 0 |
| minisforumeu | 3 | 3 | 3 | 3 | 0 |
| minisforumfr | 3 | 3 | 3 | 3 | 0 |
| minisforumhk | 3 | 3 | 3 | 3 | 0 |
| minisforumjp | 3 | 3 | 3 | 3 | 0 |
| minisforumkr | 3 | 3 | 3 | 3 | 0 |
| minisforumru | 3 | 3 | 3 | 3 | 0 |
| minisforumuk | 3 | 3 | 3 | 3 | 0 |
| minisforumus | 3 | 3 | 3 | 3 | 0 |
| ugreenca | 3 | 3 | 3 | 3 | 0 |
| ugreencom | 3 | 3 | 3 | 3 | 0 |
| ugreende | 3 | 3 | 3 | 3 | 0 |
| ugreeneu | 3 | 3 | 3 | 3 | 0 |
| ugreenfr | 3 | 3 | 3 | 3 | 0 |
| ugreenin | 3 | 3 | 3 | 3 | 0 |
| ugreenjp | 3 | 3 | 3 | 3 | 0 |
| ugreenkr | 3 | 3 | 3 | 3 | 0 |
| ugreennas | 3 | 3 | 3 | 3 | 0 |
| ugreennasau | 3 | 3 | 3 | 3 | 0 |
| ugreennasca | 3 | 3 | 3 | 3 | 0 |
| ugreennasde | 2 | 2 | 2 | 2 | 0 |
| ugreennases | 3 | 3 | 3 | 3 | 0 |
| ugreennaseu | 3 | 3 | 3 | 3 | 0 |
| ugreennasfr | 3 | 3 | 3 | 3 | 0 |
| ugreennasit | 3 | 3 | 3 | 3 | 0 |
| ugreennasjp | 3 | 3 | 3 | 3 | 0 |
| ugreennasuk | 2 | 2 | 2 | 2 | 0 |
| ugreennl | 3 | 3 | 3 | 3 | 0 |
| ugreenuk | 3 | 3 | 3 | 3 | 0 |
| ugreenus | 3 | 3 | 3 | 3 | 0 |

Anker Korea has no structuralized document yet. Other unlisted storefronts lack
saved pages in this corpus; no coverage or inferred defaults are claimed for them.
Portable Ugreen CA and Anker AU fixtures preserve the conflicting payment/widget
examples. Regions are supported in the typed contract only when explicitly
supplied; no region-specific source path was established by this corpus.

Run `cargo test -p retailer_data_extractor --test localization_corpus -- --nocapture`
from `lib` for the read-only survey. Portable tests run without the corpus.
