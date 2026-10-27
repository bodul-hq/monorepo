# Item 22: Derive missing market from the source locale

Status: completed; CI, PostgreSQL and fresh-source live pipeline checks passed.

## Goal and confirmed scope

The user requests that a source locale fill a missing target market and supply
the language component: a new observation with no established market and
`language: "it-IT"` must become
`{"target_market":{"kind":"country","code":"IT"},"language":"it"}`.
This changes item 19's decision to retain complete language tags and never
derive a market from them. Apply the change to the downloaded page's accepted
locale evidence, then carry the resulting observation through the existing
pipeline to customers.

The user also confirmed that language is reduced to its primary subtag for
**every new localization observation**, including observations with an explicit
market or a locale without a country. Country derivation remains a fallback
for a missing market. Preserve the complete source locale in evidence.

The rules below for conflict handling, unsupported regions and historical data
are proposed implementation choices, not additional user confirmations. Keep
the work bounded to source selection and propagation verification. No new
source families, UI labels, filtering, translation, shipping inference,
database repair job or automatic historical backfill is proposed.

## Starting behavior

Grounded at monorepo `ed55cb1eb34740ef1e2ace2a8ba0174a8a078443`.

- [Extraction](../../lib/retailer_data_extractor/src/localization.rs) already
  captures `html[lang]` and standalone top-level `Shopify.locale` and
  `Shopify.country` literal assignments. Valid document language wins over the
  Shopify language fallback. Payment settings, defaults, alternate links,
  widgets and nested/dynamic JavaScript are excluded. Scripts are never run.
- The `Evidence` sidecar has `selected` facts and candidates containing `path`,
  `raw` and `decision`. `select` compares original language strings, so even
  spelling-only case differences can currently look like conflicts.
  `target_market: None` currently conflates absence, invalid explicit market
  declarations and conflicting explicit markets. A final null check alone
  cannot implement a provenance-aware fallback safely.
- [Shared localization](../../lib/retailer_sauce/src/localization.rs) validates
  ICU Unicode BCP47 locale syntax while preserving full source spelling. It
  rejects underscores and inputs over 128 bytes. The crate already depends on
  `icu_locale_core`; the [country type](../../lib/everyday/src/country_code.rs)
  validates supported ISO alpha-2 codes, uppercases them and maps `UK` to `GB`.
  It does not accept `EU`, `ZZ` or numeric macroregions as countries.
- The [facade](../../lib/retailer_data_extractor/src/lib.rs) selects facts only
  during extraction, detaches the sidecar before strict retailer decoding,
  preserves it during normalization, and copies it into structuralized
  variations. Existing artifacts are not recalculated by these later stages.
- [Export conversion](../../services/retailer_sourcing/src/offer_discovery/processing_for_export/conversion.rs)
  copies selected facts. The current export version is **v5**, localization
  was added in v4, and v1–v3 omit it. Offers already emits
  `OfferLocalizationChanged.v1`; both projections store market JSON and
  language text. Customer-facing REST/GraphQL and the
  [customers client](../../apps/customers/src/offers.rs) already carry both.
- The [coverage survey](../localization-source-coverage.md) records three
  Anker Italy pages with known language and unknown market, and analogous gaps
  for several Anker shops. That count alone does not establish each page's
  exact locale; inspect the saved sources before claiming affected coverage.
  The portable Anker AU fixture currently asserts `en-AU` with an unknown
  market, while the Ugreen fixture asserts explicit `CA` with `fr`.

## Proposed selection rules

1. **Keep source eligibility and precedence.** Choose the applicable document
   or active-storefront locale using the current source rules. Retain the raw
   declaration and path. Do not derive anything from a URL, retailer code,
   currency, default locale, alternative page or payment/widget configuration.
   Do not broaden supported JavaScript syntax as part of this fix.
2. **Parse the complete accepted locale.** Reuse the existing ICU parser and
   validation limits. Read its language identifier's primary language and
   region fields; do not split on a hyphen and assume the second token is a
   country. Normalize syntactic case for comparison before deciding whether
   equivalent declarations conflict. Preserve the original raw spelling in
   evidence. Compare full locale identities before reducing the language:
   `en-US` and `en-GB` must not silently become an apparent agreement on `en`.
3. **Distinguish missing market from rejected market.** Represent selection
   outcomes internally as absent, accepted, invalid or conflicting, or an
   equivalent explicit result. Apply locale-to-country fallback only when no
   applicable explicit market was declared. A malformed, unsupported, blank,
   dynamic or contradictory explicit declaration blocks the fallback and
   remains unknown with diagnostics. A language conflict also blocks derivation.
   Do not infer this distinction later by searching diagnostic prose.
4. **Respect explicit market evidence.** Keep a valid explicit country or
   regional market even if the locale names a different region. For example,
   explicit `CA` with document `en-US` retains `CA`; record that locale fallback
   was unnecessary, rather than inventing an equal-scope market conflict.
   Explicit markets outrank a derived fallback. Existing invalid/conflicting
   explicit-market behavior stays conservative.
5. **Validate the region as a country.** Convert only a parsed region that the
   existing `CountryCode` accepts. Preserve its current `UK` → `GB` behavior
   and test it explicitly. Missing regions, `419`, `001`, `EU`, `ZZ` and other
   unsupported codes cannot establish a country. Do not create a region label,
   map a numeric region to a country, or run likely-subtag maximization to fill
   missing information. A syntactically valid tag may still have no usable
   country; that is not a malformed offer.
6. **Handle extra subtags deliberately.** `zh-Hant-TW` exposes `zh` and `TW`;
   `sl-SI-rozaj` exposes `sl` and `SI`. Script, variants, extensions and private
   use remain in raw evidence. A Unicode extension such as `en-u-rg-itzzzz`
   does not provide the language identifier's region; `rg`/`sd` extensions must
   not override it. Reject unsupported complete tags according to the existing
   parser contract. Preserve `und` as the explicit undetermined-language marker
   rather than inventing a natural language; an explicit valid region in
   `und-IT` can still establish `IT` under this proposal.
7. **Reduce language for every new source observation.** A valid selected
   locale produces its lowercase primary language whether the market is
   explicit, derived or still unknown. Successful fallback also produces an
   uppercase country. Preserve full tags in evidence and old serialized
   observations unchanged. Invalid optional source declarations remain unknown
   without rejecting the offer. Typed payload validation continues rejecting
   malformed values.

Record explicit reasons for selected direct market, selected locale-derived
market, missing/unsupported region, and blocked fallback. A derived market's
provenance must point to the same winning locale candidate that supplied its
language, including its full raw value. Preserve rejected candidates as today.
If additive structured provenance or a selection-policy version is introduced,
deserialize old sidecars without those fields; treat them as legacy evidence,
not permission to recompute their selected facts.

## Implementation sequence

1. **Record the behavior and decision.** Add a new indexed ADR explicitly
   superseding ADR 28's full-tag/source-selection policy to the extent changed
   here, including the corresponding no-derivation rule in
   [item 19](19-offer-market-and-language.md). Leave accepted ADR 28 unchanged.
   Retain its evidence exclusions, independent unknowns, observation replacement
   and replay rules. Allocate the next available ADR number at implementation
   time; do not assume item 23 will use a particular number.
2. **Capture representative fixtures.** Inspect saved Anker Italy HTML and
   retain a minimal portable `html[lang="it-IT"]` regression. Add a standalone
   Shopify locale fallback fixture and a case where document language outranks
   a disagreeing Shopify locale. Update the Anker AU expectation to `AU`/`en`,
   while retaining rejected Next.js/widget candidates. Do not modify the user's
   saved corpus or treat its availability as a test prerequisite.
3. **Implement extraction-time selection.** Refactor market/locale selection
   to retain explicit outcome and winning-candidate provenance. Reuse parsing
   from `retailer_sauce` through a small helper if needed rather than installing
   locale interpretation in generic deserialization. Prefer the existing ICU
   dependency; any newly shared dependency must follow workspace conventions.
   Keep `Localization` capable of reading old complete tags. Preserve facade
   ordering and the strict retailer schemas; no per-retailer special-case
   default is needed for a document locale.
4. **Verify stage and downstream contracts.** Carry the selected observation
   unchanged through normalization, structuralization, export v5, import,
   creation/update events, both projections, REST/GraphQL and the customers
   response model. Do not derive market independently in export conversion,
   API serialization, an aggregate replay handler or a database read. Add a
   shared `IT`/`it` contract example to existing integration tests. Preserve
   title, price, identifiers, images, scrape time and card presentation.
5. **Document and roll out.** Update the source survey, extraction/shared-type
   READMEs and [offer facts walkthrough](../offer-facts-walkthrough.md) with the
   changed provenance and observed coverage. State exactly which source tags
   remain unsupported. Rebuild/redeploy sourcing with the updated extraction
   library after verifying existing consumers accept both old full tags and
   new primary-language strings. No database migration, export-shape version,
   creation-event version or new event type is needed for this selection fix.

## Historical data and deployment boundaries

Apply the new policy to new extractions. Old extracted/normalized/structuralized
objects retain their selected facts when read or processed further. An already
prepared export retains its bytes, version, entry IDs and message payloads on
retry; never decorate it using a newer source or the current selection policy.
Retain v1–v5 export readers and existing queued commands' version/defaults.

Do not mutate existing projection rows or stored Testimony/Mulac events merely
because they contain `it-IT`. Historical event replay and old imports must
reproduce their original observations. Missing import localization still means
no observation; an explicit object still replaces both fields, including nulls.
New source observations for an existing offer use the ordinary localization
change event. An identical reimport emits no additional localization event.

For a deployment demonstration, obtain a new source observation using the
normal download/extraction pipeline, then drive its queues explicitly. Simply
replaying an old export cannot correct it. Any later historical repair needs a
separately scoped workflow that creates new source-derived artifacts/events
and delivery identities; never overwrite immutable objects or reuse message IDs
with changed facts. Rollback stops producing the new selection policy but does
not automatically reverse observations already applied. Existing processing-order
updates and the offers publication failure window remain outside this item.

## Acceptance and verification

Freeze these expectations in portable tests. Rules beyond the confirmed
locale fallback and all-new-observation language reduction remain proposed
edge-case policy as identified above:

| Applicable evidence | Expected market | Expected language |
| --- | --- | --- |
| No explicit market; document `it-IT` | Country `IT` | `it` |
| No explicit market; case variant `IT-it` | Country `IT` | `it` |
| No document language; standalone Shopify locale `it-IT` | Country `IT` | `it` |
| Document `it-IT`; Shopify locale `en-US` | Country `IT` | `it` |
| No explicit market; `zh-Hant-TW` / `sl-SI-rozaj` | Country `TW` / `SI` | `zh` / `sl` |
| No explicit market; `en-UK` | Country `GB` | `en` |
| No explicit market; `it` or `zh-Hant` | Unknown | `it` / `zh` |
| No explicit market; `es-419`, `en-001`, `en-EU`, `en-ZZ` | Unknown | `es` / `en` / `en` / `en` |
| No explicit market; `en-u-rg-itzzzz` | Unknown | `en` |
| Explicit country `CA`; document `en-US` | Country `CA` | `en` |
| Explicit regional market `Europe`; document `it-IT` | Region `Europe` | `it` |
| Conflicting/dynamic/invalid explicit market; document `it-IT` | Unknown | `it` |
| No explicit market; equal-scope `en-US` and `en-GB` | Unknown | Unknown |
| No explicit market; `und-IT` | Country `IT` | `und` (undetermined; proposed) |
| Only payment/widget/default/alternate locale evidence | Unknown | Unknown |

- Test complete-tag parsing, equivalent casing, invalid/blank/oversized/control
  strings, underscores, unsupported grammar, script/variant/private-use tags,
  extension regions, missing country and missing language. Preserve malicious
  script exclusion tests; never execute source JavaScript. Test equivalent
  locale deduplication without collapsing genuine regional conflicts.
- Assert raw values, exact source paths and derived/blocked decisions survive
  all stages and reach the same variations. Old sidecars and artifacts without
  evidence remain readable and unchanged. The existing
  [corpus test](../../lib/retailer_data_extractor/tests/localization_corpus.rs)
  is useful for parity; add exact regression assertions because known-count
  increases alone do not prove the selected country is correct.
- Verify export conversion preserves new `IT`/`it` and old null/`it-IT`
  observations as supplied. Existing prepared snapshots, legacy version shapes,
  retries and delivery identities must stay unchanged. Test a fresh creation,
  null/`it-IT` → `IT`/`it` update, identical reimport, explicit null observation,
  omitted localization and mixed historical event replay.
- Use existing isolated PostgreSQL suites for sourcing snapshot replay,
  offers localization projection, and customer-facing transactional receipt
  replay. Check REST/GraphQL selected values and customers deserialization;
  card HTML and page language remain unchanged. No new route is proposed.
- Run `make fmt` then root `make ci`; run `make audit` only if Cargo dependencies
  or lockfiles change. Run affected service/app smoke tests. Start sourcing,
  offers, customer-facing and customers with distinct `PORT=<port> make serve`
  values and isolated fixture databases; verify health/pages and explicitly
  process commands, events, inboxes and outboxes until the `IT`/`it` observation
  appears in customer-facing and the customers response model.

Before marking complete, record the implemented language scope, final ADR,
portable/corpus coverage, actual CI/PostgreSQL results, immutable replay checks
and one fresh-source end-to-end result. Report unavailable corpus or database
checks as unverified rather than passed.

## Implementation verification (2026-10-26)

ADR 31 records the implemented policy: primary language for every new valid
observation, locale-country fallback only for absent direct market declarations.
No Cargo dependency, migration, export/event version or historical data changed.

- `make fmt` and `make ci` pass in retailer_sauce (12 tests) and
  retailer_data_extractor (14 unit tests plus the saved-corpus integration test).
- The read-only survey passed across 133 saved pages in 45 storefronts, with
  130 full pipelines and no errors; Anker Korea still has no structuralized
  document. All three Anker Italy sources contain `it-IT`; exact IT/it selection
  and raw/path provenance passed. The updated coverage table records results.
- Export conversion's new/legacy observation and byte-replay test passes.
  Offers' old null/it-IT → IT/it → explicit null transition, identical/omitted
  reimports and mixed-history replay test passes. Customers' new/legacy model
  preservation test passes.
- Root CI is blocked by existing inbox-client TCP listener tests in the sandbox.
  Changed service/app format checks and clippy pass. Offers tests: 67 pass,
  four socket tests blocked, 23 PostgreSQL tests ignored. Customers: 22 pass,
  six socket tests blocked. Sourcing main tests: 123 pass, two socket failures,
  one pre-existing stale test expects schema v3 to be unsupported although the
  listener accepts v1–v5; 68 database tests ignored. Sourcing CI also encounters
  an existing export CLI socket fixture failure before its main binary tests.
- Service startup attempts cannot complete: offers requires DATABASE_URL;
  sourcing's existing configuration has an invalid object-storage base URL.
  Customer-facing/app startup restrictions are recorded in item 21. Isolated
  database replay and fresh-source end-to-end health/page verification remain
  unverified. No database or saved corpus was modified.

That initial run left the TODO unchecked. The completion run below resolves
the outstanding environment restrictions.

## Completion verification (2026-10-26)

Root `make fmt` and `make ci` pass on the working tree at `cc57884`, including
all socket tests and the read-only saved-corpus survey. PostgreSQL suites pass:
sourcing 60 tests, offers 23 and customer-facing 29. This includes immutable
snapshot/retry checks and transactional projection/replay coverage. The stale
unsupported-export fixtures now use version 6; version 3 remains supported.

Object storage, sourcing, offers, customer-facing and customers were started
with distinct `PORT=<port> make serve` settings and disposable PostgreSQL 16
databases. Every health endpoint/page responded. A local HTTP fixture served
the saved Anker Italy `products-a2154.html` bytes without changing the corpus.
A fresh DownloadOfferPage command then ran through extraction, normalization,
structuralization, v5 export, sourcing outbox, offers inbox/commands/events/outbox
and customer-facing inbox/commands. Both resulting offers carried IT/it, scrape
time, ordered images and identifiers through REST/GraphQL and customers search.
The queues drained without processing errors.

The user's additional EU requirement is recorded separately in
[ADR 33](../adr/0033-european-union-locale-market.md), which supersedes only the
EU exclusion above. Portable/corpus tests now also verify `en-EU` produces the
regional market European Union and language en, while legacy null/en-EU facts
remain unchanged through later stages and immutable exports. All three saved
Anker EU pages pass exact market/language/provenance assertions. A fresh download
of `eu-en-products-a110j-anker-zolo-power-bank-10k-30w.html` was driven through
the same live flow; its offer reached REST, GraphQL and the customers search
page with European Union/en. No historical data or saved corpus was rewritten.
