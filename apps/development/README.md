# development

Developer tools for processing saved retailer sitemaps and offer pages, plus a
Poem server with a home page at `/`.

## Offline tools

Run these targets from the repository root. `DATA_DIR` defaults to `../data`;
`RETAILERS="ankerau ankerjapancom"` limits the offer stages to those
storefronts.

| Target                        | Input and output                                                                        |
| ----------------------------- | --------------------------------------------------------------------------------------- |
| `process-sitemap-sources`     | `sitemaps/sources/<retrieval id>/` to `sitemaps/processed/<retrieval id>.json`          |
| `consolidate-sitemap-sources` | Processed sitemap trees to `sitemaps/consolidate/` URL groups                           |
| `extract-offer-details`       | `offer_details/sources/*.html` to `offer_details/extracted/*.json`                      |
| `normalize-offer-details`     | Extracted offers to `offer_details/normalized/`                                         |
| `structuralize-offer-details` | Normalized offers to `offer_details/structuralized/`                                    |
| `process-offer-details`       | All offer stages through the extractor crate's CLI; its final folder is `structurized/` |

Paths are under `<DATA_DIR>/<retailer>/`. Failed offer stages remove stale
output and report errors while continuing with other pages.

`dump-sitemap-sources` downloads source files from the retailer sourcing
development API. `dump-offer-details` first lists database metadata from that
API, then downloads HTML directly from object storage. Both use
`RETAILER_SOURCING_URL` (default `http://localhost:3000`) and optionally
`RETAILER_SOURCING_TOKEN` to override the repository development token. For
`dump-offer-details`, also set `OBJECT_STORAGE_URL` (default
`http://172.28.100.43:3000`) and optionally `OBJECT_STORAGE_TOKEN` to override
the repository object-storage development token. It retrieves records from the
`offer_details_raw` bucket using each source's `storage.object_storage_id`,
validates the ID, bucket, external ID and UTF-8 byte length, and writes the
payload. Older copies are skipped before downloading; individual download or
validation failures are reported as skipped sources while other sources
continue. The offline stages operate on existing files without a running
service.

## Run and test

```sh
make fmt
make ci
PORT=3005 make serve
curl http://localhost:3005/
```

Tests cover the home page and the tools' file handling, stage failures, stale
output cleanup, and sitemap processing.

## Database dashboard

Developer tools and a server-rendered database dashboard at `/dashboard`. The
home page links to the dashboard. Reporting clients and response structs live
inside this app's server binary; service credentials never reach the browser.

### Run

```sh
PORT=3005 make serve
```

Open `http://localhost:3005/dashboard`. Each page load fetches reports from all four services
concurrently, with a five-second timeout per service. Use **Refresh reports** to
reload; there is no background polling. One service can fail without hiding the
other services’ reports.

| Environment variable      | Default                                                   |
| ------------------------- | --------------------------------------------------------- |
| `RETAILER_SOURCING_URL`   | `http://172.28.100.40:3000`                               |
| `RETAILER_SOURCING_TOKEN` | Repository development bearer token for retailer sourcing |
| `OBJECT_STORAGE_URL`      | `http://172.28.100.43:3000`                               |
| `OBJECT_STORAGE_TOKEN`    | Repository development bearer token for object storage    |
| `OFFERS_URL` | `http://172.28.100.41:3000` |
| `OFFERS_TOKEN` | `offers-development-token`; match Offers `OFFERS_REPORTING_TOKEN` |
| `CUSTOMER_FACING_URL` | `http://172.28.100.42:3000` |
| `CUSTOMER_FACING_TOKEN` | `customer-facing-development-token`; match Customer Facing `CUSTOMER_FACING_REPORTING_TOKEN` |
| `PORT`                    | `3000`                                                    |

Export overrides in the shell before starting the app. The existing CLI tools
retain their own defaults. The dashboard uses the app's existing access model
and is intended for the development environment.

### Reading the report

Each service reports ordinary tables in its connection's active PostgreSQL
schema, including migration and message bus tables. Row counts are **estimates**
from PostgreSQL statistics and can lag recent writes. A zero estimate does not
prove a table is empty. Reporting neither counts table rows nor runs ANALYZE.

Table bytes include TOAST and auxiliary storage. Index bytes cover indexes
attached to the table; total bytes are their sum. The dashboard displays IEC
units, with exact byte values available on hover and to assistive technology.
Tables are sorted alphabetically by name. Combined totals cover only available
services.

### Testing

```sh
make fmt
make ci
```

Rust tests cover private HTTP clients, authentication, timeouts, malformed
responses, partial failure, HTML escaping, sorting, and page routes. The home
page and dashboard share the page shell in `src/layout.rs`, which embeds
`src/style.css` into the HTML; no frontend build is required. The pages follow
the system light or dark preference, and a top-bar toggle overrides it. The
toggle is the only JavaScript: a small inline script that stores the choice in
`localStorage`, and the toggle stays hidden when scripts are off. After CSS
changes, run the app and check `/` and `/dashboard` in a browser in both themes
at desktop and narrow mobile widths. Verify that the page fits the viewport,
wide tables scroll within their cards, and Refresh remains usable.
Also stop one upstream service and check that the other services remain visible.

The dashboard also requests retailer sourcing’s, Offers’, and Customer Facing’s `/api/reporting/entry-statuses`
server-side and displays separate service sections with exact inbox, command, event and outbox counts by status, including
friendly labels for known statuses. Their numeric codes and API `status_text` labels
are available in hover-only tooltips. Unknown statuses display their API labels
and numeric codes visibly. These counts refresh on page load. Empty queues use
compact status-colored boxes with visually hidden labels and accessible zero counts.
Unavailable status reports have their own messages, and a failed status request
does not hide the database table reports.

Customer Facing is the fourth database card. Its offer projection and queue
storage contribute to combined totals only when its table report succeeds.
Its status report can fail independently of its table report. Inbox and outbox have their own status breakdowns alongside commands and events. The overview
counts database reports available out of four; queue-report failures do not
change that number. Reporting follows [ADR 13](../../docs/adr/0013-development-database-reporting.md).

Rebuild and restart customer-facing before development to enable the new reports.
For local verification, run customer-facing with `PORT=3103 make serve` against
an isolated test schema, then run this app with
`CUSTOMER_FACING_URL=http://localhost:3103 PORT=3105 make serve` and open
`http://localhost:3105/dashboard`. Configured tokens must match; they never
appear in the rendered HTML. Existing deployments need their binaries rebuilt
and containers recreated to pick up new environment settings.

All three queue status sections show entries scheduled for the current
UTC day, including later today. Each report uses its own generation time; retries
can change the daily totals. Refresh the page to fetch current counts.

Inbox, command, event and outbox counts for Retailer Sourcing, Offers, and Customer Facing
each appear in six responsive boxes: New (received)
is light green, Processing (reserved) is medium green, Completed is green, and Failed is orange,
Dead is black, and Archive is grey. Missing statuses count as zero. Zero numbers are visually hidden in every box,
including queues with other nonzero statuses, while remaining available to screen readers. Empty queues retain their status colors.
If any count is nonzero, every box in that queue keeps its status color; unknown codes
appear in additional neutral boxes. Box colors and contrasting labels work in
both themes.

The entry-status report includes `inbox_entries`, `command_entries`,
`event_entries`, and `outbox_entries` arrays, grouped by current status for
entries scheduled today in UTC. All four queues share one query snapshot.
Reports include `window_start` (inclusive) and `window_end` (exclusive), the
UTC midnight bounds used by the query. The dashboard derives its report date
from these bounds. Reports without bounds show an upgrade message for the
entire panel; their potentially all-time counts are not displayed as daily totals.
Invalid or incomplete windows are rejected. Missing inbox/outbox arrays in a
windowed report still show an upgrade message for those queues.

Queue section headings use the three service SVGs in `assets/`, embedded into
the binary and served under `/assets/` without a separate asset deployment.
Each icon sits in a side column beside its service’s four queue groups, within
one bordered panel. Images have accessible service labels. Database cards retain their text headings.
Empty queues show small status-colored boxes without an additional empty-day message.

Queue rows use the inbox, commands, events and outbox SVGs from `assets/`
as their first element, followed by the status boxes. Accessible image labels
replace the visible collection titles.

All status boxes match their queue icon height: 64px normally and 40px on narrow
screens. When every count in a queue is zero, its icon and colored boxes shrink
together to 24px (18px on narrow screens), with box labels and zeros visually
hidden but available to screen readers. The icon column retains its full width
so status columns align across empty and populated collections.
