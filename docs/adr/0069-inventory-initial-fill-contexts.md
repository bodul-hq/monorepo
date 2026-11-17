# 69. Inventory-backed initial-fill contexts

Status: Accepted

Supersedes live market selection in ADRs 59 and 68. Brand ownership remains as
specified in ADR 64.

## Decision

Keep token acquisition and the DiscoverStorefrontMarkets / StorefrontMarketsDiscovered
command/event stage. Its handler loads Retailer by code and snapshots the entity's
supported_locales without HTTP discovery. Entities contain only main-table locales
from docs/retailer.md; advertised-only entries remain documentation. Collapse
aliases by optional country and language, retaining declaration order. Missing
inventory, empty supported locales and region-valued countries fail explicitly.

Use a local LocaleInitialFillIntegration extension in retailers_wip for requests
with optional country and no predetermined currency. The published integration
interface remains available for historical callers. No registry patch is needed.
Do not infer country from a locale, URL, target market or shipping configuration.
Always send the selected language, and omit country when absent. Validate response
context; carry an observed default country through catalog events and commands to
check consistency without inserting it into request variables. Command identities
use the requested country/language, not the observation. Historical string country
and currency payloads still deserialize and preserve their identities/expectations.

Brand engines derive currency from product responses, reject mixed currencies and
require consistent context across pagination. Emit the existing schema-2 artifact
and terminal event with observed country/language/currency. Missing context or
currency evidence fails rather than inventing values. Historical schema-1 artifacts
and context-free queued work retain their existing behavior. Development listings,
extraction and country-language dump paths retain their resolved context format.

## Rollout and verification

No database migration or start-endpoint change is required. Drain existing fills
and deploy all workers before starting new inventory-backed runs. Persisted discovery
events replay their snapshot; do not replay old token events to change a run's policy.
Validate one explicit-country and one countryless fill before broader use. CI uses
fixtures; live acceptance requires reachable storefronts and object storage.

Tests cover inventory/table agreement, all 48 connected retailers, alias deduplication,
countryless inventory requests, explicit non-default languages, historical payloads/identities, pagination consistency,
currency validation and terminal storage without downstream work.
