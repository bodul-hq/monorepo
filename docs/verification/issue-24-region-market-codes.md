# Issue 24: Regional market codes verification

Base: upstream/main `567bac8552c7dca5a6ade65f864e78cd08382279`.

The regional contract is implemented across extraction, immutable export
preparation, Offers imports/event records, both projections and customer-facing
and customers read boundaries. Historical region payloads retain their exact
label encoding; current known-EU boundaries normalize to `eu`. Unmapped labels
remain readable and migrations refuse to invent mappings.

Version matrix:

| Contract | New writer | Historical support |
| --- | --- | --- |
| Source export | 9 | 1–8, including legacy-label regeneration |
| OfferCreated | 10 | 1–8 legacy content and 9 list content/label regions |
| OfferLocalizationChanged | 2 | 1 legacy-label regions |
| Customer-facing HTTP | Existing routes with region code | Legacy labels accepted by customers; optional label retained for historical values |

No Cargo dependencies or lockfiles changed; audit is not required. Projection
migrations perform preflight before writes and do not touch event/outbox stores.

## Validation

`make fmt` completed for the lib, services and apps workspaces. All affected
crate CI commands completed lint checks. `make ci` passed for retailer_sauce,
retailer_data_extractor and customer_facing (42 tests passed, 38 database tests
ignored). Offers passed 88 tests; four publisher tests failed because this
sandbox refuses TCP listener creation (`Operation not permitted`). Customers
passed 29 tests; six HTTP fixture tests failed for the same restriction.
Retailer sourcing CI stopped at its export CLI HTTP fixture (six passed, one
socket-permission failure). Its separately run export processing suite passed
all 20 tests, with 13 database tests ignored.

Regression coverage includes historical label round trips, code validation,
new-writer rejection of unmapped labels, old/new export versions, versioned
Offers events, equivalence of historical EU and code EU observations, and
customer API/client compatibility. Migration fixtures verify countries/nulls,
EU up/down conversion and atomic rejection of unknown labels.

`make test-postgres` was attempted in all three services and requires
`DATABASE_URL_TEST`, which is absent. Migration tests therefore remain
unexecuted. No operational database was used. Server startup/health checks
were attempted with `PORT=3424 make serve`; this environment cannot provide
the database configuration or TCP listener access needed for running services.
These environment-dependent checks must pass in CI before merge.
