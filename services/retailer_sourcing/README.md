# retailer_sourcing

Finds what retailers sell. It crawls each retailer's sitemaps, sorts the page
URLs into products, catalogs and other content, records every product URL as an
offer, and downloads the offer pages.

The code is organised as feature slices
([ADR 4](../../docs/adr/0004-feature-slices-for-retailer-sourcing.md)). Each
slice handles one command and returns events; a reaction in the next slice turns
an event into the next command. Commands and events are stored in Postgres by
the [Mulac](https://github.com/nulllabsdev/mulac) kernel.

## Workflow

```
POST /api/sitemap-retrievals
  └─▶ RetrieveSitemap ──▶ retrieve_sitemap handler ──▶ SitemapRetrieved
        └─▶ process_sitemap reaction ──▶ ProcessSitemap ──▶ process_sitemap handler ──▶ SitemapProcessed
              └─▶ consolidate_sitemap reaction ──▶ ConsolidateSitemap ──▶ consolidate_sitemap handler ──▶ SitemapConsolidated
                    └─▶ discover_offers reaction ──▶ DiscoverOffers ──▶ discover_offers handler ──▶ OfferDiscovered × N
                          └─▶ download_offer_page reaction ──▶ DownloadOfferPage × N ──▶ download_offer_page handler
                                ──▶ OfferPageWasDownloaded | OfferPageDownloadSkipped | no event (download failed)
```

| Step | Command              | Handler does                                                                                                                                                                                                    | Event                                                                                 | Reaction that follows                                                 |
| ---- | -------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------- | --------------------------------------------------------------------- |
| 1    | `RetrieveSitemap`    | Fetches the retailer's root sitemaps and every child sitemap and stores the files in `raw_sitemap_documents`. A file that can't be fetched or parsed is skipped with a warning. Fails without a sitemap config. | `SitemapRetrieved`                                                                    | `process_sitemap`: queue `ProcessSitemap`                             |
| 2    | `ProcessSitemap`     | Parses the stored files into one sitemap tree, stores it in `processed_sitemaps`.                                                                                                                               | `SitemapProcessed`                                                                    | `consolidate_sitemap`: queue `ConsolidateSitemap`                     |
| 3    | `ConsolidateSitemap` | Sorts every URL of the tree into product, catalog, content, not interested and unknown, using the retailer's link rules; stores the groups in `consolidated_sitemaps`.                                          | `SitemapConsolidated`                                                                 | `discover_offers`: queue `DiscoverOffers`, unless it found 0 products |
| 4    | `DiscoverOffers`     | Reads the product URLs and gives each its own `offer_id`. Writes nothing.                                                                                                                                       | `OfferDiscovered`, one per URL                                                        | `download_offer_page`: queue one `DownloadOfferPage` per offer        |
| 5    | `DownloadOfferPage`  | If the URL was downloaded in the last 6 hours, skips it. Otherwise downloads the page and stores it in `raw_offers` under the `offer_id`. A failed download is logged, and the command still completes.         | `OfferPageWasDownloaded` or `OfferPageDownloadSkipped`; none when the download failed | none yet                                                              |

Every row a crawl writes in steps 1 to 3 carries its `retrieval_id`. Each
step logs its outcome (see [Logging](#logging)); `command_entries` and
`event_entries` show what is queued, done or failed.

## Code layout

```
src/
  main.rs                    routes, startup; registers each context
  platform/                  plumbing that knows nothing about the domain
    bus.rs                   command and event bus over Mulac
    bus_api.rs               POST /api/commands/run and /api/events/run
    db.rs                    Postgres pool and embedded migrations
    http.rs                  blocking HTTP client
  sitemap_discovery/         context: sitemaps → sorted URLs
    retrieve_sitemap/        step 1, plus POST /api/sitemap-retrievals
    process_sitemap/         step 2
    consolidate_sitemap/     step 3
  offer_discovery/           context: product URLs → downloaded offer pages
    discover_offers/         step 4
    download_offer_page/     step 5
migrations/                  Mulac's tables, then each context's tables
```

A slice holds `command.rs`, `handler.rs`, `event.rs`, `store.rs` (its own
queries) and `reacts_to.rs` (the reaction that starts it). Each context and
slice exposes only its `prelude` module, with `register()`, any `routes()`, and
the events others may use. Contexts talk to each other only through events.

## Running it

The service needs Postgres for the workflow; without `DATABASE_URL` it serves
HTTP only and the routes below return `503`. On startup it applies pending
migrations.

```sh
make .env          # once: DATABASE_URL for the retailer_sourcing-db container
make migrate       # optional; startup also migrates
PORT=3002 make serve
```

`make migrate-redo` reverts and reapplies every migration, which deletes the
data.

When the service runs in the workspace's `compose-apps.yaml` container, rebuild
and restart it after code changes, from the workspace folder:

```sh
make services-restart
```

Events are delivered to the reactions registered in the running binary. An event
processed by an older build that had no reaction for it is marked done and isn't
delivered again.

## Driving the workflow

Nothing processes the queues in the background (the polling workers are
commented out in `Bus::start`). Queue retrievals, then run commands and events
in turns:

| Route                          | Body                          | Does                                                                      |
| ------------------------------ | ----------------------------- | ------------------------------------------------------------------------- |
| `POST /api/sitemap-retrievals` | `{"retailers": ["ugreenca"]}` | Queues a `RetrieveSitemap` per retailer, returns each `retrieval_id`      |
| `POST /api/commands/run`       | `{"limit": 10}`               | Runs up to `limit` queued commands and waits for them                     |
| `POST /api/events/run`         | `{"limit": 10}`               | Runs up to `limit` queued events; their reactions queue the next commands |

One retailer from start to finish takes alternating runs:

1. `sitemap-retrievals` with the retailer.
2. `commands/run`: retrieves the sitemaps (can take minutes; keep the limit small).
3. `events/run`, then `commands/run`: processes the sitemaps.
4. `events/run`, then `commands/run`: consolidates them.
5. `events/run`, then `commands/run`: discovers the offers.
6. `events/run`, then `commands/run` until it reports `"processed": 0`: downloads the offer pages.

Run responses look like `{"processed": 3, "errors": []}`. When something fails,
`processed` is `null` and `errors` lists the failures; failed commands are
retried on a later run.

The Bruno collection in `../../bruno/retailer_sourcing` has a request for each
route.

## Logging

Logs go to stderr and to daily files under `logs/` (set `LOG_DIR` to change the
folder; it's git-ignored):

| File                               | Contains                                                                                                                       |
| ---------------------------------- | ------------------------------------------------------------------------------------------------------------------------------ |
| `retailer_sourcing.log.YYYY-MM-DD` | The same lines as stderr: each step's outcome, skipped sitemap files, failed downloads and commands, one line per HTTP request |
| `timing.log.YYYY-MM-DD`            | How long each fetch, command and reaction took, and whether it succeeded                                                       |

`RUST_LOG` sets the level, for example `RUST_LOG=debug`; the default is `info`.
In the compose container the files are written inside the container, so mount
a volume on the log folder to keep them.

## Tests

`make ci` runs formatting, clippy and the tests. Handler tests use in-memory
fakes, so they need no database or network. The store queries are checked by
running the service against Postgres.
