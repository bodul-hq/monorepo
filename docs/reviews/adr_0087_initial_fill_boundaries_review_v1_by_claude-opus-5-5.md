# Review: ADR 87, Initial-fill boundaries (v1)

- Date: 2026-11-16
- Reviewer: Claude Opus 5.5 (`claude-opus-5-5`)
- Branch: `chore/use-bodul-registry` at `d2393fa1`
- Scope: [`docs/adr/0087-initial-fill-boundaries.md`](../adr/0087-initial-fill-boundaries.md),
  compared with the ADRs it cites and with the code it describes.

Line numbers such as "ADR line 22" refer to the ADR at `d2393fa1`. Paths that
start with `retailer_sauce/` refer to the local checkout that `lib/Cargo.toml`
points to (`../../../space/bodul_hq/lib_retailer_sauce`, at `fb4def1`). All
other paths are relative to the repository root.

## Method

Claude wrote this review from the point of view of a panel of 3 principal
engineers, 5 staff engineers and 10 tech leads:

- **Principal engineers:** boundaries, dependency direction, how the ADR fits
  with ADRs 57–86, and governance across the two repositories.
- **Staff engineers:** failure semantics, rollout and mixed-version safety,
  provenance, security and observability.
- **Tech leads:** what the change means for the code each team owns:
  `retailer_sourcing`, `retailer_data_extractor`, the three brand crates, the
  `retailer_sauce` engine, tests and CI, and documentation conventions.

The panel read the ADR, ADRs 57–86, the initial-fill slice in
`retailer_sourcing`, `retailer_data_extractor`, the brand configurations, and
`retailer_sauce`'s source, README and manifesto. Findings marked ✔ were checked
against the code.

## Summary

The direction is right, and every reviewer agreed. Token acquisition as a
process stage is why each change in access policy (ADRs 70–78) also changed
process steps, queue handling and rollout. Moving credential resolution into
the Shopify engine removes that coupling.

The ADR is not ready to be Accepted:

- Some claims contradict the code: the event's consumers, the module mapping,
  and the dependency direction.
- It conflicts with `retailer_sauce`'s own manifesto, and it needs a breaking
  release in another repository that it doesn't mention.
- It leaves the hard parts undecided: where the credential cache lives, what
  counts as an authentication failure, the shape of the transport port, and
  mixed-version rollout.

**Recommendation:** set the status to Proposed. Address findings 1–7, then
mark it Accepted. A proposed revision is in the [appendix](#appendix-proposed-revision).

## Must fix before acceptance

### 1. Nothing consumes the output event ✔

ADR lines 22 and 56–57 say that `offers` and `customer_facing` are consumers
and that they subscribe to `OfferDetailsFetched`.

- `services/retailer_sourcing/src/initial_offer_fill/offer_details/prelude.rs:9`
  says: "OfferDetailsFetched is deliberately terminal: no subscriber or outbox
  registration."
- Neither `services/offers` nor `services/customer_facing` references the event
  or the initial-fill artifact.

**Fix:** state that the event stays terminal and that delivering it to other
services needs its own ADR, as ADRs 16 and 20 did for other events. Remove the
Consumers row, or mark it as having no consumers yet.

### 2. It conflicts with `retailer_sauce`'s manifesto and needs a cross-repository release ✔

- `retailer_sauce/manifesto.md` item 2: "Initial fill is our business process.
  Preserve its signatures; redesigning Shopify coupling is deferred."
- Item 10: "Limit changes to this repository; document consumer migration."
  Item 11 freezes the manifesto until that effort is complete.
- `retailer_sauce/README.md:17-18`: "redesigning this boundary is deferred.
  Retailers own source interpretation and network execution."

ADR 87 changes exactly that boundary. It removes the token parameters from
`InitialFillIntegration` and `LocaleInitialFillIntegration`
(`retailer_sauce/src/initial_fill.rs:30`, `:146`), removes `resolve_access`
(`:33`), and moves network policy into `retailer_sauce`. That is a breaking
release of a crate that lives in another repository. The monorepo depends on it
as `retailer_sauce 0.2.0` through the `bodul` registry and a local path.

**Fix:** decide the order with the `retailer_sauce` effort, either after its
manifesto is complete or by amending the manifesto. Name the breaking release
in the ADR, and update `retailer_sauce`'s README in the same release.

### 3. The boundary table doesn't match the code, and dependencies don't only point downwards ✔

- `retailer_sauce::shopify::graphql` contains `auth.rs`, `token.rs`,
  `scripts.rs`, `module_tokens.rs`, `fetch.rs` and `catalog.rs`. That is the
  engine and credential resolution, not only "query documents and the raw
  model" (ADR line 26). The source schema is
  `shopify::graphql::{operations, raw}`.
- `retailer_sauce::shopify`, the bottom row, is the parent module of
  `shopify::graphql`, the row above it. The table places a parent below its own
  child.
- `retailer_sauce/src/shopify/config.rs:2` imports
  `crate::retailers::RetailerProfile`. So Storefront platform (bottom row)
  depends on Retailer catalog (above it), which is an upward edge.
- The table doesn't place these modules at all:
  - `retailer_sauce::initial_fill`, which holds the integration contract and
    the `Http` port. `shopify/graphql/{fetch,catalog,auth}.rs` import it.
  - `retailer_sauce::graphql`, which holds the schema-2 `OfferDetails`
    artifact and the response diagnostics.
  - `retailer_data_extractor`, which connects sourcing to the brands
    (`initial_fill_integration_for`, `initial_fill_process`).
- The engine, the subject of the "The engine is stateless" section, has no
  row.

**Fix:** use the exact module paths, and give the engine and the contracts their
own rows. Order the rows so the dependency rule is true. Add an architecture
test that enforces it, like ADR 61's isolation tests did. The appendix has a
six-row table that matches the current imports.

### 4. The brand row contradicts itself ✔

ADR line 24 says brand crates don't know about credentials. The 52
`StorefrontAccess { … }` blocks in `lib/retailers_*` set `acquire_token`,
`token_required`, `token_environment` and `token_market`.

**Fix:** "Brands declare the access policy as data; the engine carries it
out." Rename the row from "Brand extraction" to "Brand integration", because it
owns configuration as well as extraction.

### 5. "Sourcing does not use Shopify token types" needs a port change the ADR doesn't decide ✔

ADR line 38 makes this claim. Today, sourcing implements the port with Shopify
token types:

- `services/retailer_sourcing/src/initial_offer_fill/transport.rs:8-18`
  implements `Http::post(url, request, Option<&StorefrontToken>)` and attaches
  the Shopify header itself through `post_storefront_json`.
- `acquire_storefront_token/handler.rs:7` implements
  `configured_token() -> Option<StorefrontToken>` by reading environment
  variables.
- The paced `OfferClient` used for catalog and detail fetches doesn't implement
  `get`. The default returns "GET is unavailable", and token discovery needs
  GET.

**Fix:** decide the port. Recommended: the engine builds request headers, and
the port carries them as opaque, redacted values. Sourcing's client then builds
no Shopify headers. The port also offers GET and an environment lookup.
Specify whether discovery GETs are paced like API calls.

### 6. "The engine is stateless" and "caches the result per store" contradict each other ✔

ADR lines 33 and 50. `initial_fill_integration_for` returns static
integrations, so the obvious place for a cache is a hidden `static` in a
library. That cache would have no metrics, would leak state between tests, and
would live only inside one process.

**Fix:** sourcing supplies the cache through the port, and the engine keeps no
state. Specify:

- the key (the retailer code, since `token_market` is set per store);
- that each worker process has its own cache;
- a time-to-live, because discovered tokens can rotate;
- single-flight resolution, so that concurrent commands don't each run
  discovery.

### 7. "Authentication failure" is undefined, and errors are untyped ✔

ADR line 27 gives the platform "error classification", and line 34 says it
"resolves again once after an authentication failure". However, every port and
engine function returns `Result<_, String>`, for example
`retailer_sauce/src/shopify/graphql/fetch.rs:14` and `initial_fill.rs:10-25`.

The ADRs this one replaces were mostly about scope failures. The token was
valid but lacked `unauthenticated_read_product_pickup_locations`, so
`storeAvailability` failed (ADRs 70, 77 and 78). Resolving again returns the
same token, so "retry once" can't help.

**Fix:**

- Define an authentication failure: HTTP 401 or 403, or a GraphQL
  `UNAUTHORIZED` or `ACCESS_DENIED` error for the whole request.
- State the policy for a valid token that lacks a scope.
- Type the errors that cross the port as transient, throttled (with a
  retry-after time), authentication or permanent. Sourcing's retry policy
  depends on those classes.

## Should fix

### 8. The durable record of the access kind is lost ✔

Today, `StorefrontTokenAcquired` records whether each run used a token. After
ADR 87 that appears only in logs (ADR line 69). Logs aren't durable and can't
be joined to artifacts. The access kind changes what a response contains:
ADR 78 recorded empty pickup connections under anonymous access, and anonymous
access has a cost limit of 1,000 (ADR 60). The kind can also change in the
middle of a run when the cache expires or a credential is resolved again.

**Fix:** record the credential kind (`environment`, `discovered` or
`anonymous`) on each artifact `Page`, never the credential itself. `Page` and
`OfferDetails` don't use `deny_unknown_fields`
(`retailer_sauce/src/graphql.rs:115-127`). An optional field with a Serde
default therefore keeps schema 2, stored artifacts and older readers working.

### 9. The rollout section is underspecified ✔

- **"Deploy all workers together" (ADR line 64) is not the real constraint.**
  The `storefront_token` fields (`offer_details/command.rs:17`,
  `product_catalog_ids/command.rs:13`) are `Option`, so Serde reads a missing
  field as `None`. Workers from before ADR 87 therefore accept new commands and
  send anonymous requests, which breaks stores that require a token. The rule
  should be: no pre-87 worker may process a catalog or detail command created
  after the cutover. Stop or drain those workers before the new start API
  accepts runs.
- **"Remain readable" (ADR line 61) is ambiguous.** It should say whether
  unprocessed `StorefrontTokenAcquired` events are still handled, as ADR 74
  did for its stage. It should also say how replaying a historical token event
  stays idempotent, given ADR 69's rule against replaying token events.
- **Environment variables.** Every worker now needs the token environment
  variables, not only the one that ran acquisition.
- **Leftover code to remove.** The ADR should list
  `Step::AcquireStorefrontToken`
  (`lib/retailer_data_extractor/src/initial_fill_process.rs:24`) and the
  `all_market_contexts` and `discover_markets` stubs. ADR 74 left those stubs
  for "a future `retailer_sauce` release", and this is that release.
- **Tokens at rest.** Tokens already in queue rows stay at rest. They are
  public storefront tokens, so the risk is low, but the ADR should say so.

### 10. The supersession and context lists are inaccurate ✔

- "Per-store access settings from ADRs 70–78 remain in effect" (ADR lines 6–7)
  includes ADR 73, which is about selling plans, and ADR 74, which line 5
  supersedes.
- ADRs 70, 72 and 75–78 also contain process rules that ADR 87 supersedes:
  "new fills start with catalog commands", "queued acquisition retries skip
  discovery", and "catalog/detail requests ignore historical tokens". ADR 69
  kept the token stage, ADR 82 has the resolver called with the URL from the
  command, and ADR 83 defines handling of historical tokens. These parts are
  superseded too.
- The token-policy list in the context section (ADR line 12) leaves out ADR 76
  (Ugreen FR). The ownership list cites ADR 63, which was the monorepo import,
  and ADR 86, which says "the v2 initial-fill pipeline … unchanged". It leaves
  out ADR 57.

**Fix:** list exactly which ADRs, and which parts of them, are superseded. Say
that their per-store `acquire_token` and `token_required` values remain as
configuration.

### 11. The shared-query rule is incomplete and not enforced

ADR lines 44–45 mention query cost only. Scopes are the second constraint, as
ADRs 77 and 78 showed, and nothing in CI checks either.

**Fix:**

- A new selection must stay within the cost limit, and need no missing scope,
  for the least-privileged access kind in use.
- Add a check: a static cost estimate at the configured page sizes, or at
  least a live acceptance step.
- Add a way out. A need that one brand can't share gets a separate operation
  chosen by configuration, rather than a fork of the shared query. Without
  this, the next brand-specific field will push the code back towards
  per-brand queries, repeating the moves in ADRs 61, 64 and 83.

### 12. There are no Validation or Alternatives sections

ADRs 60, 70 and 82 include validation. The main trade-off isn't written down:
using one credential for every page of a run makes runs reproducible, while
not persisting credentials keeps policy as configuration only.

**Fix:** add both sections. The appendix has a short version of each.

## Low

- Add `Date: 2026-11-16`. ADR 86 has a date; ADR 87 doesn't.
- The "Owner" column lists modules rather than teams. Rename it to "Code".
- "Retailer catalog" is easily confused with the product-catalog step. The code
  calls these `RetailerProfile`; use "Retailer profiles".
- "Reports completeness" (ADR line 56) is ambiguous. If it means catalog
  coverage, say "does not judge catalog coverage". Sourcing still has to know
  when a run is complete.
- `offer_details_raw` is both an object storage bucket and a PostgreSQL table.
  Write "the `offer_details_raw` bucket" and name the type,
  `retailer_sauce::graphql::OfferDetails`.
- `OfferDetailsFetched` uses `MarketContext` from `retailer_sauce::shopify`, so
  the process event depends on a Shopify module type. This is acceptable if
  `shopify::markets` is listed with the shared contracts, as it is in the
  appendix.
- `docs/adr/README.md` doesn't note supersession for list entries from ADR 55
  onwards, while the table rows above them do. Consider adding "superseded in
  part by ADR 87" to the entries for ADRs 60, 69 and 74.

## What's fine

- The diagnosis is correct. Each of ADRs 70–78 had to repeat the same
  queued-command and rollout rules, and that is the cost this ADR removes.
- Commands and events no longer carry credentials, which is a real security
  improvement.
- Command identities stay unchanged, so in-flight runs keep their
  deduplication.
- The consequences section states the cost of shared queries openly instead of
  hiding it.
- The ADR is short and consistent with the style of the others. The TODO entry
  and the index entry were added as AGENTS.md requires.

## Decisions for the author

1. **Split into two ADRs?** Two principal engineers favour splitting "boundaries"
   from "engine-owned credentials", so that the next token change replaces
   only the credentials ADR. One prefers a single ADR, because the boundaries
   are the reason for the credentials move.
2. **Fall back to anonymous access automatically?** For stores where the token
   is optional, switching to anonymous access after an authentication or scope
   failure would have made most of ADRs 70–78 unnecessary. It would also hide
   broken credentials and mix access kinds within one run. Finding 8 would make
   that mix visible. The appendix keeps the current policy: the error stays
   visible, and the fix is a configuration change.
3. **Where does the cache live?** The options are a port supplied by sourcing
   (recommended, used in the appendix) or a static cache inside
   `retailer_sauce`.

## Suggested order

1. Resolve the `retailer_sauce` manifesto question (finding 2), because it
   determines whether this can happen now.
2. Make the three decisions above.
3. Correct the factual problems (findings 1, 3, 4 and 10).
4. Specify the port, cache, error classes and provenance (findings 5–8).
5. Rewrite the rollout section, and add Validation and Alternatives
   (findings 9, 11 and 12).
6. Apply the low-priority fixes, then mark the ADR Accepted.

## Appendix: proposed revision

This revision uses the recommended answers to the decisions above and keeps
one ADR.

```markdown
# 87. Initial-fill boundaries and engine-owned Storefront credentials

Date: 2026-11-16

Status: Proposed

Supersedes the Storefront token stage and its queue handling in ADRs 60, 69 and
74, and the process rules in ADRs 70–72 and 75–78: new fills start at the
catalog, queued acquisition retries skip discovery, and catalog and detail
requests ignore historical tokens. The per-store `acquire_token` and
`token_required` values those ADRs set remain in effect as configuration.
ADR 82's resolver and ADR 83's engine are unchanged, except that the engine now
calls the resolver instead of the process. Confirms ADRs 83 and 84: query
documents and the raw model stay shared.

## Context

ADR 82 moved token discovery into `retailer_sauce`, but sourcing still runs it
as the `AcquireStorefrontToken` stage and carries the token in every catalog and
detail command. As a result, each access-policy change in ADRs 70–78 also
changed process steps, queued-command handling and rollout, even though only
per-store configuration was meant to change.

Ownership of Shopify GraphQL code also moved repeatedly, in ADRs 57, 58, 61, 64
and 80–85. This ADR records the resulting boundaries so that later changes stay
within one of them.

`retailer_sauce`'s manifesto postpones redesigning the `initial_fill`
signatures. This decision needs that redesign and a breaking `retailer_sauce`
release.

## Decision

### Boundaries

| Boundary | Code | Owns | Does not know about |
| --- | --- | --- | --- |
| Initial-fill process | `retailer_sourcing::initial_offer_fill` | Runs, run and command identities, commands and events, persistence, retries, pacing, artifact storage | Shopify endpoints, credentials, query fields |
| Integration registry | `retailer_data_extractor` | Selecting a retailer's integration and process steps | Runs, transport, credentials |
| Brand integration | `retailers_<brand>` | Storefront configuration values, including access policy, and raw-to-extracted conversion | Runs, transport, credential resolution |
| Shopify engine | `retailer_sauce::initial_fill`, `retailer_sauce::shopify::graphql` except `operations` and `raw` | Integration contract, transport port, pagination, context and currency checks, credential resolution, errors | Brands, runs |
| Shopify contracts | `retailer_sauce::shopify::graphql::{operations, raw}`, `retailer_sauce::graphql`, `retailer_sauce::shopify::{config, markets}` | Query documents, raw response model, schema-2 artifact, diagnostics, access-setting and market-context types | Business meaning of fields, transport, brands |
| Retailer profiles | `retailer_sauce::retailers` | Retailers, markets, shipping countries, supported locales | HTTP, Shopify |

A boundary uses only boundaries listed below it. Nothing depends on the
initial-fill process. An architecture test rejects any other dependency.

### The engine resolves credentials

The engine resolves credentials from `StorefrontAccess` when it sends a request.
It uses an environment override, token discovery or anonymous access, as
configured. Resolution order and parsing are unchanged from ADRs 60 and 82.

- Sourcing supplies a credential cache through the transport port, so the
  engine itself keeps no state. Entries are keyed by retailer code, belong to
  one worker process, are resolved once when requested concurrently, and expire
  after a configured time-to-live.
- An authentication failure is HTTP 401 or 403, or a GraphQL `UNAUTHORIZED` or
  `ACCESS_DENIED` error for the whole request. After one, the engine drops the
  cached credential, resolves it again, and retries the request once. A second
  failure is classified as an authentication error.
- A valid credential that lacks a field's scope is not an authentication
  failure. The error stays visible, and the fix is a configuration change, as
  in ADRs 77 and 78.
- The transport port takes headers built by the engine. Sourcing's client
  builds no Shopify headers and does not use `StorefrontToken`.
- Errors that cross the port are typed as transient, throttled (with a
  retry-after time), authentication or permanent. Sourcing's retry policy uses
  these classes.

The process becomes `ProductCatalogIdentifiers → OfferDetails` for every
storefront. Commands and events carry no credentials.

### Pages record the credential kind

Each artifact `Page` records the credential kind used: `environment`,
`discovered` or `anonymous`. It never records the credential itself. The field
is optional and has a Serde default, so schema 2 and stored artifacts stay
unchanged. Anonymous responses can omit data, such as pickup availability, so
readers need to know the kind for each page.

### Queries and the raw model are shared

A selection added for one brand is sent for every brand. Before it is added, it
must:

- pass every brand fixture suite;
- stay within Shopify's query-cost limit for the least-privileged access kind
  in use, which is 1,000 for anonymous access (ADR 60);
- need no scope that any configured access kind lacks.

When one brand needs something the others cannot share, it gets a separate
operation chosen by configuration. The shared query and the raw model are not
forked.

### The engine keeps no run state

The engine fetches pages, follows pagination, and checks context and currency
consistency. Sourcing decides what to run and owns persistence, retries,
pacing and dispatch.

### The fill ends at the raw artifact

The fill's output is the schema-2 `retailer_sauce::graphql::OfferDetails`
artifact in the `offer_details_raw` bucket, together with the
`OfferDetailsFetched` event. The fill does not extract offers or judge catalog
coverage. `OfferDetailsFetched` stays terminal. Publishing it to other services
needs its own ADR.

## Compatibility and rollout

- Release a breaking `retailer_sauce` version. `InitialFillIntegration::catalog`
  and `details` lose their token parameters. `resolve_access` is removed, along
  with the `all_market_contexts` and `discover_markets` stubs that ADR 74 left
  for removal. Update the `retailer_sauce` README and manifesto in the same
  release.
- Remove `Step::AcquireStorefrontToken` from
  `retailer_data_extractor::initial_fill_process`.
- Workers still handle historical `AcquireStorefrontToken` commands and
  unprocessed `StorefrontTokenAcquired` events. Both fan out catalog commands
  as the start API does, under the existing stable command identities, and
  ignore any token.
- Catalog and detail commands still accept a `storefront_token` field and
  ignore it. Tokens already stored in queue rows stay there. They are public
  storefront tokens.
- Workers from before this ADR can read new commands, because the missing
  token deserializes as `None`. They would then send anonymous requests to
  stores that require a token. Stop or drain those workers before the new start
  API accepts runs, and retry commands that fail during the switch.
- Every worker needs the token environment variables, not only the worker that
  ran acquisition.

## Validation

- Unit tests with a fake transport cover resolution order, cache hits, a
  single new resolution after HTTP 401, no new resolution when a scope is
  missing, and redaction.
- Historical-payload tests cover queued acquisition commands, unprocessed token
  events, and catalog and detail commands that carry tokens.
- PostgreSQL workflow tests cover all 48 retailers without the token stage.
- An architecture test enforces the boundary dependencies.
- Before the TODO closes, run one live fill for each credential kind.

## Alternatives considered

- **Keep the stage and carry only the credential kind.** Policy changes would
  still change process steps and queues.
- **Resolve once per run and store the credential on the run.** Every page in a
  run would use the same credential. However, credentials would be persisted,
  and the process would still own the policy.
- **Resolve in sourcing through a port it owns.** Shopify discovery rules would
  return to sourcing, after ADR 82 moved them out.

## Consequences

Access policy changes are configuration changes, with no effect on runs or
queues. Queues and events no longer store credentials. Pages record the
credential kind, and metrics report resolutions by store and kind. Discovery
failures now show up as catalog or detail command failures, labelled as
credential-resolution errors. Shared queries make adding fields cheap, but
they couple request cost and scope across brands.
```
