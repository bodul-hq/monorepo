# ADR 87 review marks by Codex

- Date: 2026-11-16
- Reviewer: Codex
- Subject: [Claude's v1 review](adr_0087_initial_fill_boundaries_review_v1_by_claude-opus-5-5.md) of [ADR 87](../adr/0087-initial-fill-boundaries.md)

## Overall assessment

The review is useful and its main diagnosis is sound. Use it to revise ADR 87,
but do not adopt its appendix unchanged. It is strongest as an implementation
readiness critique; credential handling alone does not settle the initial-fill
business architecture.

These comments are recommendations, not amendments to the accepted ADR.

## Findings to adopt

- **Finding 1 — output consumers:** correct the ADR. `OfferDetailsFetched`
  explicitly has no subscriber or outbox registration in
  `services/retailer_sourcing/src/initial_offer_fill/offer_details/prelude.rs`.
  Raw acquisition is the current endpoint.
- **Findings 3 and 4 — ownership:** describe the actual engine and contracts,
  and distinguish declaring access policy from resolving credentials. Brand
  integrations currently declare that policy.
- **Finding 5 — transport:** removing tokens from commands requires changing
  the transport contract. Sourcing currently receives `StorefrontToken` and
  constructs Shopify-authenticated requests in
  `services/retailer_sourcing/src/initial_offer_fill/transport.rs`.
- **Finding 8 — provenance:** record access provenance with captured evidence,
  preferably per page. Logs alone cannot reliably explain differences between
  anonymous and authenticated captures. Do not record the credential itself.
- **Finding 9 — rollout:** make worker compatibility explicit. A payload that
  deserializes successfully can still execute incorrectly when an old worker
  interprets an omitted token as anonymous access.
- **Findings 10–12 — decision clarity:** make supersession precise and document
  validation and alternatives. Retain explicit scope and query-cost constraints.

## Findings and proposals to revise

### 1. Credential state is different from business-run state

Finding 6 correctly challenges the word “stateless,” but caching credentials
does not contradict keeping no run state. The choice is broader than a hidden
static cache versus a cache supplied through the HTTP port.

An explicitly constructed Shopify client can own an injected credential
resolver/cache. Sourcing controls its lifetime; Shopify owns its semantics.
Keep HTTP execution, environment access and credential caching as distinct
responsibilities rather than accumulating them in one `Http` interface.

Expiry and concurrent resolution behavior should be explicit regardless of
which component owns the cache.

### 2. Retailer code is not automatically the right cache key

The five Nordic retailer identities share one Shopify backend, while access
policy can differ or change for a retailer. A cache key should represent the
credential-resolution configuration: acquisition source/shop, relevant market
and access policy. Retailer identity alone does not express those dimensions.

Separate entries per retailer can be a conservative initial choice, but shared
entries require evidence that their resolution settings are equivalent.

### 3. Authentication classification needs more precision

Finding 7 rightly distinguishes invalid credentials from insufficient scope.
However, the appendix's blanket refresh rule for HTTP 403 or whole-request
`ACCESS_DENIED` risks retrying permanently forbidden requests.

Preserve enough structured evidence to distinguish refreshable authentication
failures, scope failures, throttling and other failures. Do not assume a fresh
credential repairs every denial. Keep unknown failures distinguishable instead
of assigning a misleading classification.

### 4. The proposed boundary table retains conceptual problems

The appendix still puts Shopify-specific artifacts inside generic `graphql`
and process steps inside an integration registry. Reconsider both:

- Generic GraphQL owns protocol concerns such as requests, diagnostics and
  response evidence. An artifact containing Shopify product and market
  semantics belongs with Shopify unless deliberately redesigned as a
  provider-independent capture envelope.
- Sourcing owns business orchestration. The registry selects integrations and
  exposes capabilities; it should not duplicate a process step list that
  sourcing reconstructs and validates.

A parent module containing a child is not itself a dependency violation.
Define permitted dependencies between responsibilities, then test those
specific rules. A table's vertical order or directory hierarchy is not enough
to establish the architecture.

### 5. Raw acquisition still needs a run outcome

The appendix corrects the nonexistent consumers but does not fully resolve
business completion. Distinguish two claims:

- The system cannot guarantee that the source exposed every commercial
  product.
- The system can determine whether its requested traversal finished and all
  discovered detail tasks reached an outcome.

Sourcing needs the second even if initial fill ends at raw artifacts. Define a
durable run with accepted scope and completion, partial-success and failure
outcomes. Removing token acquisition as a process stage does not solve this.

### 6. The manifesto conflict is a sequencing constraint

Finding 2 identifies a real conflict: the sauce manifesto currently preserves
initial-fill signatures and defers this redesign. Reconcile that scope and
coordinate the breaking release and consumer migration explicitly.

The manifesto does not determine whether the proposed architecture is sound.
It establishes work that must be coordinated before implementation.

## Recommended decision structure

Split the decision into two parts:

1. **Business boundaries:** what a fill accomplishes, who owns its run and
   accepted scope, and how retailer, acquisition source, collection context
   and artifact identities relate.
2. **Credential execution and migration:** resolver/cache ownership,
   transport contract, error classification, provenance and worker cutover.

Remove the duplicated process declaration as part of establishing ownership.
Then implement credential changes against those boundaries, with historical
payload handling and mixed-version behavior specified explicitly.
