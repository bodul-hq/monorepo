# ADR 14: Direct offer source downloads

- Status: Accepted
- Date: 2026-10-18
- Supersedes: ADR 12's compatibility requirement for offer source API responses

## Context

The development app's `dump-offer-details` currently receives HTML through
retailer sourcing, which resolves every object before returning a metadata page.
The app should obtain database information from sourcing and retrieve the data
from object storage itself.

## Decision

Return offer source metadata with the stored `storage` reference instead of
`body` from `/api/development/offer-details/sources`. Listing requires only the
sourcing database. Preserve pagination and newest-first ordering.

The development CLI uses `object_storage_client` to retrieve each needed record
from the `offer_details_raw` bucket. It validates object ID, bucket, external
ID, and UTF-8 byte length before writing HTML. Skip older copies before
retrieval. Configure each service's URL and bearer token independently.

## Consequences

- Deploy the sourcing response change and development CLI together; clients
  expecting inline `body` must switch to object retrieval.
- The CLI needs direct access to both services. Object failures skip individual
  sources without preventing retrieval of other sources in the metadata page.
- File names and HTML contents remain unchanged. Sitemap source APIs and the
  service's database export commands retain their existing behavior.
